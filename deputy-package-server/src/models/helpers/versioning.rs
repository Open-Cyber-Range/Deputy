use actix_web::{Error, HttpResponse, web::Data};
use anyhow::Result;
use log::error;
use semver::Version;
use crate::AppState;
use crate::errors::{PackageServerError, ServerResponseError};
use crate::models::Package;
use crate::services::database::package::{GetPackageByNameAndVersion, GetPackagesByName};

pub async fn get_package_by_name_and_version(name: String, version: String, app_state: Data<AppState>) -> Result<Package, Error> {
    Ok(app_state
        .database_address
        .send(GetPackageByNameAndVersion {
            name,
            version,
        })
        .await
        .map_err(|error| {
            error!("Failed to get package: {error}");
            ServerResponseError(PackageServerError::Pagination.into())
        })?
        .map_err(|error| {
            error!("Failed to get package: {error}");
            ServerResponseError(PackageServerError::DatabaseRecordNotFound.into())
        })?
    )
}

pub async fn get_packages_by_name(name: String, app_state: Data<AppState>) -> Result<Vec<Package>, Error> {
    Ok(app_state
        .database_address
        .send(GetPackagesByName { name })
        .await
        .map_err(|error| {
            error!("Failed to get package: {error}");
            ServerResponseError(PackageServerError::Pagination.into())
        })?
        .map_err(|error| {
            error!("Failed to get package: {error}");
            ServerResponseError(PackageServerError::DatabaseRecordNotFound.into())
        })?
    )
}

fn get_latest_package(
    packages: Vec<Package>
) -> Option<Package> {
    packages
        .iter()
        .max_by(|a, b| a.version.cmp(&b.version))
        .cloned()
}

fn is_latest_version (
    uploadable_version: &str,
    packages: Vec<Package>
) -> Result<bool> {
    let latest_package = get_latest_package(packages);
    match latest_package {
        Some(package) => Ok(uploadable_version.parse::<Version>()? > package.version.parse::<Version>()?),
        None => Ok(true)
    }
}

pub fn validate_version (uploadable_version: &str, packages: Vec<Package>) -> Result<HttpResponse, Error> {
    if let Ok(is_valid) = is_latest_version(uploadable_version, packages) {
        if !is_valid {
            error!("Package version on the server is either same or later");
            return Err(ServerResponseError(PackageServerError::VersionConflict.into()).into());
        }
    } else {
        error!("Failed to validate versioning");
        return Err(ServerResponseError(PackageServerError::VersionParse.into()).into());
    }
    Ok(HttpResponse::Ok().body("OK"))
}
