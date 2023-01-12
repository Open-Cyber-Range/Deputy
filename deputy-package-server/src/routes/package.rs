use crate::models::helpers::uuid::Uuid;
use crate::services::database::package::{CreatePackage, GetPackages};
use crate::{
    constants::{default_limit, default_page},
    errors::{PackageServerError, ServerResponseError},
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
    package::{FromBytes, Package, PackageFile, PackageMetadata},
    validation::{validate_name, validate_version_semantic, Validate},
};
use divrem::DivCeil;
use futures::{Stream, StreamExt};
use log::error;
use serde::Deserialize;
use std::path::PathBuf;
use crate::models::helpers::versioning::{get_latest_version, get_package_by_name_and_version, get_packages_by_name, validate_version};

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
    let package_metadata = if let Some(Ok(metadata_bytes)) = body.next().await {
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
        error!("Invalid stream chunk: No package metadata");
        return Ok(HttpResponse::UnprocessableEntity().body("Invalid stream chunk: No package metadata"));
    };

    let same_name_packages: Vec<crate::models::Package> = get_packages_by_name(package_metadata.clone().name, app_state.clone()).await?;
    if let Err(error) = validate_version(package_metadata.version.as_str(), same_name_packages) {
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

    let readme: PackageFile = if readme_size > 0 {
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
        result?
    } else {
        error!("Invalid stream chunk: No readme");
        drain_stream(body).await?;
        return Ok(HttpResponse::UnprocessableEntity().body("Invalid stream chunk: No readme"));
    };

    let archive_file: PackageFile =
        PackageFile::from_stream(body.skip(1), true)
            .await
            .map_err(|error| {
                error!("Failed to save the file: {error}");
                ServerResponseError(PackageServerError::FileSave.into())
            })?;

    let (readme, readme_string) = PackageFile::content_to_string(readme);
    let readme_html: String = PackageFile::markdown_to_html(&readme_string);

    let metadata = PackageMetadata {
        name: package_metadata.clone().name,
        version: package_metadata.clone().version,
        license: package_metadata.clone().license,
        readme: readme_string,
        readme_html,
        checksum: package_metadata.clone().checksum,
    };
    let mut package = Package::new(metadata, toml_file, readme, archive_file);
    package.validate().map_err(|error| {
        error!("Failed to validate the package: {error}");
        ServerResponseError(PackageServerError::PackageValidation.into())
    })?;

    app_state
        .database_address
        .send(CreatePackage(crate::models::NewPackage {
            id: Uuid::random(),
            name: package.metadata.clone().name,
            version: package.metadata.clone().version,
            license: package.metadata.clone().license,
            readme: package.metadata.clone().readme,
            readme_html: package.metadata.clone().readme_html,
            checksum: package.metadata.clone().checksum,
        }))
        .await
        .map_err(|error| {
            error!("Failed to add package: {error}");
            ServerResponseError(PackageServerError::PackageSave.into())
        })?
        .map_err(|error| {
            error!("Failed to add package: {error}");
            ServerResponseError(PackageServerError::PackageSave.into())
        })?;

    package
        .save(&app_state.storage_folders)
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
    validate_version_semantic(package_version.to_string()).map_err(|error| {
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

    validate_version_semantic(package_version.to_string()).map_err(|error| {
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
) -> Result<Json<crate::models::Package>, Error> {
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
    let package: crate::models::Package = get_package_by_name_and_version(
        package_name.to_string(),
        package_version.to_string(),
        app_state,
    ).await.map_err(|error| {
        error!("Failed: {error}");
        ServerResponseError(PackageServerError::PackageValidation.into())
    })?;
    Ok(Json(package))
}

#[get("package/{package_name}/{package_version}/try_get_latest")]
pub async fn try_get_latest_version(
    path_variables: Path<(String, String)>,
    app_state: Data<AppState>,
) -> Result<HttpResponse, Error> {
    /*
    Check if wanted version is latest.
    If yes, then return it
    If no, query db and return latest
     */
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
    let same_name_packages: Vec<crate::models::Package> = get_packages_by_name(package_name.to_string(), app_state).await?;
    // TODO think about how to make it not print errors in server logs
    if validate_version(package_version, same_name_packages.clone()).is_ok() {
        return Ok(HttpResponse::Ok().body(package_version.to_string()));
    }

    let latest_version = get_latest_version(same_name_packages)?;
    Ok(HttpResponse::Ok().body(latest_version))
}

#[get("package/{package_name}/{package_version}/readme")]
pub async fn get_readme(
    path_variables: Path<(String, String)>,
    app_state: Data<AppState>,
) -> Result<String, Error> {
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

    let package: crate::models::Package = get_package_by_name_and_version(
        package_name.to_string(),
        package_version.to_string(),
        app_state,
    ).await?;
    Ok(package.readme)
}

#[get("package/{package_name}/{package_version}/exists")]
pub async fn version_exists(
    path_variables: Path<(String, String)>,
    app_state: Data<AppState>,
) -> Result<HttpResponse, Error> {
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
    let same_name_packages: Vec<crate::models::Package> = get_packages_by_name(package_name.to_string(), app_state).await?;
    validate_version(package_version, same_name_packages)?;
    Ok(HttpResponse::Ok().body("OK"))
}
