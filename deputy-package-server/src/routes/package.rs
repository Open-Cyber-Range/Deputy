use crate::{
    errors::{PackageServerError, ServerResponseError},
    AppState,
};
use actix_files::NamedFile;
use actix_web::{
    get, put,
    web::{Bytes, Data, Path, Payload},
    Error, HttpResponse, Responder,
};
use deputy_library::{
    package::{Package, PackageFile, PackageMetadata},
    validation::{validate_name, validate_version, Validate},
};
use futures::StreamExt;
use log::error;
use std::path::PathBuf;

#[put("package")]
pub async fn add_package(package_bytes: Bytes, app_state: Data<AppState>) -> HttpResponse {
    let package_vector = package_bytes.to_vec();

    match Package::try_from(&package_vector as &[u8]) {
        Ok(mut package) => {
            let folder = &app_state.package_folder;
            let repository = &app_state.repository;
            match package.validate() {
                Ok(_) => match package.metadata.is_latest_version(repository) {
                    Ok(true) => match package.save(folder.to_string(), repository) {
                        Ok(_) => HttpResponse::Ok().finish(),
                        Err(error) => {
                            error!("Failed to save the package: {:}", error);
                            HttpResponse::InternalServerError().finish()
                        }
                    },
                    Ok(false) => {
                        error!("Package version on the server is either same or later");
                        HttpResponse::BadRequest()
                            .body("Package version on the server is either same or later")
                    }
                    Err(error) => {
                        error!("Failed to validate versioning: {:}", error);
                        HttpResponse::InternalServerError().finish()
                    }
                },
                Err(error) => {
                    error!("Failed to validate the package: {:}", error);
                    HttpResponse::BadRequest()
                        .body(format!("Failed to validate the package: {:}", error))
                }
            }
        }
        Err(error) => {
            error!("Failed to parse package body: {:?}", error);
            HttpResponse::UnprocessableEntity().body("Failed to parse package bytes")
        }
    }
}

#[put("/package/stream")]
pub async fn add_package_streaming(
    mut body: Payload,
    app_state: Data<AppState>,
) -> Result<HttpResponse, Error> {
    let metadata = if let Some(Ok(metadata_bytes)) = body.next().await {
        let metadata_vector = metadata_bytes.to_vec();
        PackageMetadata::try_from(metadata_vector.as_slice()).map_err(|error| {
            error!("Failed to parse package metadata: {error}");
            ServerResponseError(PackageServerError::MetadataParse.into())
        })?
    } else {
        error!("Invalid stream chunk: No metadata");
        return Ok(HttpResponse::UnprocessableEntity().body("Invalid stream chunk: No metadata"));
    };

    let folder = &app_state.package_folder;
    let repository = &app_state.repository;
    if let Ok(is_valid) = metadata.is_latest_version(repository) {
        if !is_valid {
            error!("Package version on the server is either same or later");
            return Err(ServerResponseError(PackageServerError::VersionConflict.into()).into());
        }
    } else {
        error!("Failed to validate versioning");
        return Err(ServerResponseError(PackageServerError::VersionParse.into()).into());
    }

    let package_file: PackageFile = PackageFile::from_stream(body).await.map_err(|error| {
        error!("Failed to save the file: {error}");
        ServerResponseError(PackageServerError::FileSave.into())
    })?;
    let mut package = Package::new(metadata, package_file);
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
    match validate_name(package_name.to_string()) {
        Ok(_) => (),
        Err(error) => {
            error!("Failed to validate package name: {error}");
        }
    }
    match validate_version(package_version.to_string()) {
        Ok(_) => (),
        Err(error) => {
            error!("Failed to validate package version: {error}");
        }
    }

    let package_path = PathBuf::from(package_folder)
        .join(package_name)
        .join(package_version);
    NamedFile::open(package_path).map_err(|error| {
        error!("Failed to open the package: {error}");
        Error::from(error)
    })
}
