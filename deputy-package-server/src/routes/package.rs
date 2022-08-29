use crate::{
    errors::{PackageServerError, ServerResponseError},
    AppState,
};
use actix_files::NamedFile;
use actix_http::error::PayloadError;
use actix_web::{
    get, put,
    web::{Bytes, Data, Path, Payload},
    Error, HttpResponse, Responder,
};
use deputy_library::{
    package::{Package, PackageFile, PackageMetadata},
    validation::{validate_name, validate_version, Validate},
};
use futures::{Stream, StreamExt};
use git2::Repository;
use log::error;
use std::path::PathBuf;

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
    println!("length server side {}", package_vector.len());

    let mut package = Package::try_from(&package_vector as &[u8]).map_err(|error| {
        error!("Failed to validate the package: {error}");
        ServerResponseError(PackageServerError::PackageParse.into())
    })?;
    let storage_folders = &app_state.storage_folders;
    let repository = &app_state.repository.lock().await;

    package.validate().map_err(|error| {
        error!("Failed to validate the package: {error}");
        ServerResponseError(PackageServerError::PackageValidation.into())
    })?;
    check_for_version_error(&package.metadata, repository)?;

    package.save(storage_folders, repository).map_err(|error| {
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

    let repository = &app_state.repository.lock().await;
    if let Err(error) = check_for_version_error(&metadata, repository) {
        drain_stream(body).await?;
        return Err(error);
    }

    let toml_file = if let Some(Ok(toml_bytes)) = body.next().await {
        let toml_bytes_vector = toml_bytes.to_vec();
        let result = PackageFile::try_from(toml_bytes_vector.as_slice()).map_err(|error| {
            error!("Failed to parse package metadata: {error}");
            ServerResponseError(PackageServerError::MetadataParse.into())
        });

        println!("toml length {}", toml_bytes_vector.len());

        if let Err(error) = result {
            drain_stream(body).await?;
            return Err(error.into());
        }
        result?
    } else {
        error!("Invalid stream chunk: No metadata");
        return Ok(HttpResponse::UnprocessableEntity().body("Invalid stream chunk: No metadata"));
    };

    let package_file: PackageFile =
        PackageFile::from_stream(body, true)
            .await
            .map_err(|error| {
                error!("Failed to save the file: {error}");
                ServerResponseError(PackageServerError::FileSave.into())
            })?;


    let mut package = Package::new(metadata, toml_file, None, package_file);
    package.validate().map_err(|error| {
        error!("Failed to validate the package: {error}");
        ServerResponseError(PackageServerError::PackageValidation.into())
    })?;

    let folder = &app_state.storage_folders;
    package.save(folder, repository).map_err(|error| {
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
