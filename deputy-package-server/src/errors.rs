use actix_http::{body::BoxBody, StatusCode};
use actix_web::{error::ResponseError, HttpResponse};
use anyhow::Error;
use log::error;
use std::fmt::{Display, Formatter, Result};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum PackageServerError {
    #[error("Failed to parse metadata")]
    MetadataParse,
    #[error("Failed to save the file")]
    FileSave,
    #[error("Failed to save the package")]
    PackageSave,
    #[error("Failed to validate the package")]
    PackageValidation,
    #[error("Failed to validate the package version")]
    PackageVersionValidation,
    #[error("Failed to validate the package version requirement")]
    PackageVersionRequirementValidation,
    #[error("Failed to validate the package name")]
    PackageNameValidation,
    #[error("Failed to parse version value")]
    VersionParse,
    #[error("Package version on the server is either same or later: {0}")]
    VersionConflict(String),
    #[error("Failed to paginate packages")]
    Pagination,
    #[error("Actix mailbox full")]
    MailboxError,
    #[error("File not found")]
    FileNotFound,
    #[error("Not found")]
    DatabaseRecordNotFound,
    #[error("Keycloak validation failed")]
    KeycloakValidationFailed,
    #[error("Token missing")]
    TokenMissing,
    #[error("App state missing")]
    AppStateMissing,
    #[error("Database query failed")]
    DatabaseQueryFailed,
}

#[derive(Debug)]
pub struct ServerResponseError(pub(crate) Error);

impl Display for ServerResponseError {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        write!(formatter, "{:?}", self.0)
    }
}

impl ResponseError for ServerResponseError {
    fn status_code(&self) -> StatusCode {
        if let Some(package_server_error) = self.0.root_cause().downcast_ref::<PackageServerError>()
        {
            return match package_server_error {
                PackageServerError::MetadataParse => StatusCode::BAD_REQUEST,
                PackageServerError::VersionParse => StatusCode::BAD_REQUEST,
                PackageServerError::PackageValidation => StatusCode::BAD_REQUEST,
                PackageServerError::PackageVersionValidation => StatusCode::BAD_REQUEST,
                PackageServerError::PackageVersionRequirementValidation => StatusCode::BAD_REQUEST,
                PackageServerError::PackageNameValidation => StatusCode::BAD_REQUEST,
                PackageServerError::VersionConflict(_) => StatusCode::CONFLICT,
                PackageServerError::FileNotFound => StatusCode::NOT_FOUND,
                PackageServerError::DatabaseRecordNotFound => StatusCode::NOT_FOUND,
                PackageServerError::TokenMissing => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
        }
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::with_body(self.status_code(), format!("{}", self.0)).map_into_boxed_body()
    }
}

impl From<Error> for ServerResponseError {
    fn from(error: Error) -> ServerResponseError {
        ServerResponseError(error)
    }
}
