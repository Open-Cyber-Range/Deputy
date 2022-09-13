use crate::{
    constants::{default_limit, default_page, PACKAGE_TOML},
    errors::{PackageServerError, ServerResponseError},
    AppState,
};
use actix_files::NamedFile;
use actix_http::{body::MessageBody, error::PayloadError, StatusCode};
use actix_web::{
    get, put,
    web::{Bytes, Data, Path, Payload, Query},
    Error, HttpResponse, Responder,
};
use anyhow::Result;
use deputy_library::{
    package::{Package, PackageFile, PackageMetadata},
    project::Project,
    validation::{validate_name, validate_version, Validate},
};
use flate2::read::MultiGzDecoder;
use futures::{Stream, StreamExt};
use git2::Repository;
use log::error;
use paginate::Pages;
use serde::Deserialize;
use serde_json;
use std::fs;
use std::path::PathBuf;
use std::{fs::File, io::Read};
use tar::Archive;

fn check_for_version_error(
    package_metadata: &PackageMetadata,
    repository: &Repository,
) -> Result<(), Error> {
    if let Ok(is_valid) = package_metadata.is_latest_version(repository) {
        if !is_valid {
            error!("Package version on the server is either same or later");
            return Err(ServerResponseError(PackageServerError::VersionConflict.into()).into());
        }
    } else {
        error!("Failed to validate versioning");
        return Err(ServerResponseError(PackageServerError::VersionParse.into()).into());
    }

    Ok(())
}

async fn drain_stream(
    stream: impl Stream<Item = Result<Bytes, PayloadError>> + Unpin + 'static,
) -> Result<(), Error> {
    stream
        .filter_map(|x| async move { x.ok().map(Ok) })
        .forward(futures::sink::drain())
        .await?;
    Ok(())
}

#[put("package")]
pub async fn add_package(
    package_bytes: Bytes,
    app_state: Data<AppState>,
) -> Result<HttpResponse, Error> {
    let package_vector = package_bytes.to_vec();

    let mut package = Package::try_from(&package_vector as &[u8]).map_err(|error| {
        error!("Failed to validate the package: {error}");
        ServerResponseError(PackageServerError::PackageParse.into())
    })?;
    let folder = &app_state.package_folder;
    let repository = &app_state.repository.lock().await;

    package.validate().map_err(|error| {
        error!("Failed to validate the package: {error}");
        ServerResponseError(PackageServerError::PackageValidation.into())
    })?;
    check_for_version_error(&package.metadata, repository)?;

    package
        .save(folder.to_string(), repository)
        .map_err(|error| {
            error!("Failed to save the package: {error}");
            ServerResponseError(PackageServerError::PackageSave.into())
        })?;
    Ok(HttpResponse::Ok().finish())
}

#[put("/package/stream")]
pub async fn add_package_streaming(
    mut body: Payload,
    app_state: Data<AppState>,
) -> Result<HttpResponse, Error> {
    let metadata = if let Some(Ok(metadata_bytes)) = body.next().await {
        let metadata_vector = metadata_bytes.to_vec();
        let result = PackageMetadata::try_from(metadata_vector.as_slice()).map_err(|error| {
            error!("Failed to parse package metadata: {error}");
            ServerResponseError(PackageServerError::MetadataParse.into())
        });
        if let Err(error) = result {
            drain_stream(body).await?;
            return Err(error.into());
        }
        result?
    } else {
        error!("Invalid stream chunk: No metadata");
        return Ok(HttpResponse::UnprocessableEntity().body("Invalid stream chunk: No metadata"));
    };

    let folder = &app_state.package_folder;
    let repository = &app_state.repository.lock().await;
    if let Err(error) = check_for_version_error(&metadata, repository) {
        drain_stream(body).await?;
        return Err(error);
    }

    let package_file: PackageFile = PackageFile::from_stream(body).await.map_err(|error| {
        error!("Failed to save the file: {error}");
        ServerResponseError(PackageServerError::FileSave.into())
    })?;

    let mut package = Package::new(metadata, package_file);
    package.validate().map_err(|error| {
        error!("Failed to validate the package: {error}");
        ServerResponseError(PackageServerError::PackageValidation.into())
    })?;
    package
        .save(folder.to_string(), repository)
        .map_err(|error| {
            error!("Failed to save the package: {error}");
            ServerResponseError(PackageServerError::PackageSave.into())
        })?;

    Ok(HttpResponse::Ok().body("OK"))
}

#[get("package/{package_name}/{package_version}/download")]
pub async fn download_package(
    path_variables: Path<(String, String)>,
    app_state: Data<AppState>,
) -> impl Responder {
    let package_folder = &app_state.package_folder;
    let package_name = &path_variables.0;
    let package_version = &path_variables.1;

    validate_name(package_name.to_string()).map_err(|error| {
        error!("Failed to validate the package name: {error}");
        ServerResponseError(PackageServerError::PackageNameValidation.into())
    })?;
    validate_version(package_version.to_string()).map_err(|error| {
        error!("Failed to validate the package version: {error}");
        ServerResponseError(PackageServerError::PackageVersionValidation.into())
    })?;

    let package_path = PathBuf::from(package_folder)
        .join(package_name)
        .join(package_version);
    NamedFile::open(package_path).map_err(|error| {
        error!("Failed to open the package: {error}");
        Error::from(error)
    })
}

fn iterate_and_parse_packages(package_path: &PathBuf) -> Result<Vec<Project>, Error> {
    let paths = fs::read_dir(package_path).unwrap();
    let mut result_vec: Vec<Project> = Vec::new();

    for package in paths {
        let versions = fs::read_dir(package?.path()).unwrap();
        for version in versions {
            let file = File::open(version?.path())?;
            let tarfile = MultiGzDecoder::new(file);
            let mut archive = Archive::new(tarfile);
            for entry in archive.entries()? {
                let mut entry = entry?;
                if entry.path()?.to_str() == Some(PACKAGE_TOML) {
                    let mut buffer = String::new();
                    entry.read_to_string(&mut buffer)?;
                    let value: Project = toml::from_str(&buffer).unwrap();
                    result_vec.push(value);
                }
            }
        }
    }
    Ok(result_vec)
}

fn paginate_json(result: Vec<Project>, query: PackageQuery) -> Vec<Project> {
    let projects: Vec<Project> = result;
    let pages = Pages::new(
        projects.len() + 1,
        usize::try_from(query.limit + 1).unwrap(),
    );
    let page = pages.with_offset(usize::try_from(query.page).unwrap());
    projects[page.start..page.end].to_vec()
}

#[derive(Deserialize, Debug)]
pub struct PackageQuery {
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_limit")]
    limit: u32,
}

#[get("package")]
pub async fn get_all_packages(
    app_state: Data<AppState>,
    query: Query<PackageQuery>,
) -> Result<HttpResponse, Error> {
    let package_path = PathBuf::from(&app_state.package_folder);
    let iteration_result = iterate_and_parse_packages(&package_path).map_err(|error| {
        error!("Failed to iterate over all packages: {error}");
        ServerResponseError(PackageServerError::Pagination.into())
    })?;
    let paginated_result = paginate_json(iteration_result, query.into_inner());
    Ok(HttpResponse::new(StatusCode::OK)
        .set_body(serde_json::to_string(&paginated_result)?.boxed()))
}
