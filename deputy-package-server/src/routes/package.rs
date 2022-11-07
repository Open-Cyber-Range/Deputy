use crate::{
    constants::{default_limit, default_page, PACKAGE_TOML},
    errors::{PackageServerError, ServerResponseError},
    utils::get_file_content_by_path,
    AppState,
};
use actix_files::NamedFile;
use actix_http::error::PayloadError;
use actix_web::{
    get, put,
    web::{Bytes, Data, Json, Path, Payload, Query},
    Error, HttpResponse, Responder,
};
use anyhow::Result;
use deputy_library::{
    constants::PAYLOAD_CHUNK_SIZE,
    package::{FromBytes, Package, PackageFile, IndexInfo},
    project::{Body, Project},
    validation::{validate_name, validate_version, Validate},
};
use divrem::DivCeil;
use futures::{Stream, StreamExt};
use git2::Repository;
use log::error;
use paginate::Pages;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use crate::services::database::package::GetPackages;

fn check_for_version_error(
    package_metadata: &IndexInfo,
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
    mut body: Payload,
    app_state: Data<AppState>,
) -> Result<HttpResponse, Error> {
    let metadata = if let Some(Ok(metadata_bytes)) = body.next().await {
        let metadata_vector = metadata_bytes.to_vec();
        let result = IndexInfo::try_from(metadata_vector.as_slice()).map_err(|error| {
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

    let repository = &app_state.repository.lock().await;
    if let Err(error) = check_for_version_error(&metadata, repository) {
        drain_stream(body).await?;
        return Err(error);
    }

    let toml_size = if let Some(Ok(bytes)) = body.next().await {
        u64::from_bytes(bytes).map_err(|error| {
            error!("Failed to parse package metadata: {error}");
            ServerResponseError(PackageServerError::MetadataParse.into())
        })?
    } else {
        error!("Invalid stream chunk: invalid toml length");
        return Ok(
            HttpResponse::UnprocessableEntity().body("Invalid stream chunk: invalid toml length")
        );
    };

    if toml_size > PAYLOAD_CHUNK_SIZE {
        error!("Invalid package.toml: abnormally large package.toml");
        drain_stream(body).await?;
        return Ok(HttpResponse::UnprocessableEntity()
            .body("Invalid package.toml: abnormally large package.toml"));
    }

    let toml_file = if let Some(Ok(toml_bytes)) = body.next().await {
        let toml_bytes_vector = toml_bytes.to_vec();
        let result = PackageFile::try_from(toml_bytes_vector.as_slice()).map_err(|error| {
            error!("Failed to parse package toml: {error}");
            ServerResponseError(PackageServerError::MetadataParse.into())
        });
        if let Err(error) = result {
            drain_stream(body).await?;
            return Err(error.into());
        }
        result?
    } else {
        error!("Invalid stream chunk: No metadata");
        drain_stream(body).await?;
        return Ok(HttpResponse::UnprocessableEntity().body("Invalid stream chunk: No metadata"));
    };

    let readme_size = if let Some(Ok(bytes)) = body.next().await {
        u64::from_bytes(bytes).map_err(|error| {
            error!("Failed to parse package metadata: {error}");
            ServerResponseError(PackageServerError::MetadataParse.into())
        })?
    } else {
        error!("Invalid stream chunk: invalid readme length");
        drain_stream(body).await?;
        return Ok(
            HttpResponse::UnprocessableEntity().body("Invalid stream chunk: invalid readme length")
        );
    };

    let optional_readme: Option<PackageFile> = if readme_size > 0 {
        let mut vector_bytes: Vec<u8> = Vec::new();
        let readme_chunk = DivCeil::div_ceil(readme_size, PAYLOAD_CHUNK_SIZE);

        for _ in 0..readme_chunk {
            if let Some(Ok(readme_bytes)) = body.next().await {
                vector_bytes.extend(readme_bytes.to_vec());
            }
        }
        let result = PackageFile::try_from(vector_bytes.as_slice()).map_err(|error| {
            error!("Failed to parse package readme: {error}");
            ServerResponseError(PackageServerError::MetadataParse.into())
        });
        if let Err(error) = result {
            drain_stream(body).await?;
            return Err(error.into());
        }
        Some(result?)
    } else {
        None
    };

    let archive_file: PackageFile =
        PackageFile::from_stream(body.skip(1), true)
            .await
            .map_err(|error| {
                error!("Failed to save the file: {error}");
                ServerResponseError(PackageServerError::FileSave.into())
            })?;

    let mut package = Package::new(metadata, toml_file, optional_readme, archive_file);
    package.validate().map_err(|error| {
        error!("Failed to validate the package: {error}");
        ServerResponseError(PackageServerError::PackageValidation.into())
    })?;

    package
        .save(&app_state.storage_folders, repository)
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
    let package_folder = &app_state.storage_folders.package_folder;
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
) -> Result<Json<Vec<crate::models::Package>>, Error> {
    let _page = query.into_inner().page;
    let packages = app_state
        .database_address
        .send(GetPackages)
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

#[derive(Debug, Deserialize)]
pub enum FileType {
    #[serde(rename = "archive")]
    Archive,
    #[serde(rename = "readme")]
    Readme,
    #[serde(rename = "toml")]
    Toml,
}

#[get("package/{package_name}/{package_version}/{file_type}")]
pub async fn download_file(
    path_variables: Path<(String, String, FileType)>,
    app_state: Data<AppState>,
) -> Result<NamedFile, Error> {
    let package_name = &path_variables.0;
    let package_version = &path_variables.1;
    let file_type = &path_variables.2;

    validate_name(package_name.to_string()).map_err(|error| {
        error!("Failed to validate the package name: {error}");
        ServerResponseError(PackageServerError::PackageNameValidation.into())
    })?;

    validate_version(package_version.to_string()).map_err(|error| {
        error!("Failed to validate the package version: {error}");
        ServerResponseError(PackageServerError::PackageVersionValidation.into())
    })?;

    let file_path = match file_type {
        FileType::Archive => PathBuf::from(&app_state.storage_folders.package_folder)
            .join(package_name)
            .join(package_version),
        FileType::Readme => PathBuf::from(&app_state.storage_folders.readme_folder)
            .join(package_name)
            .join(package_version),
        FileType::Toml => PathBuf::from(&app_state.storage_folders.toml_folder)
            .join(package_name)
            .join(package_version),
    };

    NamedFile::open(file_path).map_err(|error| {
        error!("Failed to open the file: {error}");
        Error::from(error)
    })
}

#[get("package/{package_name}/{package_version}/metadata")]
pub async fn get_metadata(
    path_variables: Path<(String, String)>,
    app_state: Data<AppState>,
) -> Result<Json<IndexInfo>, Error> {
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
    let repository = &app_state.repository.lock().await;
    let metadata = IndexInfo::get_latest_index_info(package_name.as_str(), repository).map_err(|error| {
        error!("Failed to get latest metadata: {error}");
        ServerResponseError(PackageServerError::MetadataParse.into())
    })?;
    match metadata {
        Some(inner) => Ok(Json(inner)),
        None => {
            error!("Failed to get latest metadata");
            Err(ServerResponseError(PackageServerError::MetadataParse.into()).into())
        }
    }
}
