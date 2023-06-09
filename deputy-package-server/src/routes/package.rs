use crate::models::helpers::versioning::{
    get_package_by_name_and_version, get_packages_by_name, validate_version,
};
use crate::services::database::package::{
    CreatePackage, GetPackageByNameAndVersion, GetPackages, GetVersionsByPackageName,
};
use crate::{
    constants::{default_limit, default_page},
    errors::{PackageServerError, ServerResponseError},
    AppState,
};
use actix::Actor;
use actix_files::NamedFile;
use actix_http::error::PayloadError;
use actix_web::{
    web::{Bytes, Data, Json, Path, Payload, Query},
    Error, HttpResponse,
};
use anyhow::Result;
use deputy_library::archiver::ArchiveStreamer;
use deputy_library::rest::VersionRest;
use deputy_library::{
    package::{Package, PackageFile, PackageMetadata},
    validation::{validate_name, validate_version_semantic},
};
use futures::{Stream, StreamExt};
use log::error;
use semver::{Version, VersionReq};
use serde::Deserialize;
use std::path::PathBuf;

async fn drain_stream(
    stream: impl Stream<Item = Result<Bytes, PayloadError>> + Unpin + 'static,
) -> Result<(), Error> {
    stream
        .filter_map(|x| async move { x.ok().map(Ok) })
        .forward(futures::sink::drain())
        .await?;
    Ok(())
}

pub async fn add_package<T>(
    body: Payload,
    app_state: Data<AppState<T>>,
) -> Result<HttpResponse, Error>
where
    T: Actor
        + actix::Handler<CreatePackage>
        + actix::Handler<GetVersionsByPackageName>
        + actix::Handler<GetPackageByNameAndVersion>
        + actix::Handler<GetPackages>,
    <T as actix::Actor>::Context: actix::dev::ToEnvelope<T, CreatePackage>,
    <T as actix::Actor>::Context: actix::dev::ToEnvelope<T, GetVersionsByPackageName>,
    <T as actix::Actor>::Context: actix::dev::ToEnvelope<T, GetPackageByNameAndVersion>,
    <T as actix::Actor>::Context: actix::dev::ToEnvelope<T, GetPackages>,
{
    let (package_metadata, body) = PackageMetadata::from_stream(body).await.map_err(|error| {
        error!("Failed to parse package metadata: {error}");
        ServerResponseError(PackageServerError::MetadataParse.into())
    })?;

    let versions: Vec<crate::models::Version> =
        get_packages_by_name(package_metadata.clone().name, app_state.clone()).await?;
    if let Err(error) = validate_version(package_metadata.version.as_str(), versions) {
        drain_stream(body).await?;
        return Err(error);
    }

    let archive_file: PackageFile = PackageFile::from_stream(body).await.map_err(|error| {
        error!("Failed to save the file: {error}");
        ServerResponseError(PackageServerError::FileSave.into())
    })?;

    let mut package = Package::new(package_metadata.clone(), archive_file);
    package.validate_checksum().map_err(|error| {
        error!("Failed to validate the package: {error}");
        ServerResponseError(PackageServerError::PackageValidation.into())
    })?;
    package.save(&app_state.package_folder).map_err(|error| {
        error!("Failed to save the package: {error}");
        ServerResponseError(PackageServerError::PackageSave.into())
    })?;

    let package_metadata = package.metadata.clone();
    let readme_html = package_metadata
        .readme_html(app_state.package_folder.clone().into())
        .await
        .map_err(|error| {
            error!("Failed to generate the readme html: {error}");
            ServerResponseError(PackageServerError::PackageSave.into())
        })?
        .unwrap_or_default();
    app_state
        .database_address
        .send(CreatePackage((package_metadata, readme_html).into()))
        .await
        .map_err(|error| {
            error!("Failed to add package: {error}");
            ServerResponseError(PackageServerError::PackageSave.into())
        })?
        .map_err(|error| {
            error!("Failed to add package: {error}");
            ServerResponseError(PackageServerError::PackageSave.into())
        })?;

    Ok(HttpResponse::Ok().body("OK"))
}

pub async fn download_package<T>(
    path_variables: Path<(String, String)>,
    app_state: Data<AppState<T>>,
) -> Result<NamedFile, Error>
where
    T: Actor,
{
    let package_name = &path_variables.0;
    let package_version = &path_variables.1;

    validate_name(package_name.to_string()).map_err(|error| {
        error!("Failed to validate the package name: {error}");
        ServerResponseError(PackageServerError::PackageNameValidation.into())
    })?;
    validate_version_semantic(package_version.to_string()).map_err(|error| {
        error!("Failed to validate the package version: {error}");
        ServerResponseError(PackageServerError::PackageVersionValidation.into())
    })?;

    let package_path = PathBuf::from(app_state.package_folder.clone())
        .join(package_name)
        .join(package_version);
    NamedFile::open(package_path).map_err(|error| {
        error!("Failed to open the package: {error}");
        Error::from(error)
    })
}

#[derive(Deserialize, Debug, Default)]
pub struct PackageQuery {
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_limit")]
    limit: u32,
}

pub async fn get_all_packages<T>(
    app_state: Data<AppState<T>>,
    query: Query<PackageQuery>,
) -> Result<Json<Vec<crate::models::Package>>, Error>
where
    T: Actor + actix::Handler<GetPackages>,
    <T as actix::Actor>::Context: actix::dev::ToEnvelope<T, GetPackages>,
{
    let packages = app_state
        .database_address
        .send(GetPackages {
            page: query.page as i64,
            per_page: query.limit as i64,
        })
        .await
        .map_err(|error| {
            error!("Failed to get all packages: {error}");
            ServerResponseError(PackageServerError::Pagination.into())
        })?
        .map_err(|error| {
            error!("Failed to get all packages: {error}");
            ServerResponseError(PackageServerError::Pagination.into())
        })?;
    Ok(Json(packages))
}

#[derive(Deserialize, Debug)]
pub struct VersionQuery {
    pub version_requirement: Option<String>,
}

pub async fn get_all_versions<T>(
    path_variable: Path<String>,
    app_state: Data<AppState<T>>,
    query: Query<VersionQuery>,
) -> Result<Json<Vec<VersionRest>>, Error>
where
    T: Actor + actix::Handler<GetVersionsByPackageName>,
    <T as actix::Actor>::Context: actix::dev::ToEnvelope<T, GetVersionsByPackageName>,
{
    let package_name = path_variable.into_inner();
    validate_name(package_name.to_string()).map_err(|error| {
        error!("Failed to validate the package name: {error}");
        ServerResponseError(PackageServerError::PackageNameValidation.into())
    })?;
    let version_requirement = match &query.version_requirement {
        Some(version_requirement) => match VersionReq::parse(version_requirement) {
            Ok(version_requirement) => Some(version_requirement),
            Err(error) => {
                error!("Failed to validate the version requirement: {error}");
                return Err(ServerResponseError(
                    PackageServerError::PackageVersionRequirementValidation.into(),
                )
                .into());
            }
        },
        None => None,
    };

    let packages: Vec<VersionRest> = get_packages_by_name(package_name.to_string(), app_state)
        .await?
        .into_iter()
        .filter_map(|package| match &version_requirement {
            Some(version_requirement) => {
                if let Ok(version) = Version::parse(&package.version) {
                    if version_requirement.matches(&version) {
                        return Some(package);
                    }
                }
                None
            }
            None => Some(package),
        })
        .map(|package| package.into())
        .collect();
    Ok(Json(packages))
}

pub async fn download_file<T>(
    path_variables: Path<(String, String, String)>,
    app_state: Data<AppState<T>>,
) -> Result<HttpResponse, Error>
where
    T: Actor,
{
    let package_name = &path_variables.0;
    let package_version = &path_variables.1;
    let file_path_in_package = &path_variables.2;

    let package_path = PathBuf::from(&app_state.package_folder)
        .join(package_name)
        .join(package_version);

    let archive_stream = ArchiveStreamer::try_new(package_path, file_path_in_package.into())
        .map_err(|error| {
            error!("Failed to open the package: {error}");
            ServerResponseError(PackageServerError::FileNotFound.into())
        })?
        .ok_or_else(|| {
            error!("File not found from the archive");
            ServerResponseError(PackageServerError::FileNotFound.into())
        })?;

    Ok(HttpResponse::Ok().streaming(archive_stream))
}

pub async fn get_package_version<T>(
    path_variables: Path<(String, String)>,
    app_state: Data<AppState<T>>,
) -> Result<Json<VersionRest>, Error>
where
    T: Actor + actix::Handler<GetPackageByNameAndVersion>,
    <T as actix::Actor>::Context: actix::dev::ToEnvelope<T, GetPackageByNameAndVersion>,
{
    let package_name = &path_variables.0;
    let package_version = &path_variables.1;

    let package_version = get_package_by_name_and_version(
        package_name.to_string(),
        package_version.to_string(),
        app_state,
    )
    .await?;

    Ok(Json(package_version.into()))
}
