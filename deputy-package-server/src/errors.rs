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
    #[error("Failed to parse version value")]
    VersionParse,
    #[error("Package version on the server is either same or later")]
    VersionConflict,
}

#[derive(Debug)]
pub struct ServerResponseError(pub(crate) Error);

impl Display for ServerResponseError {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        write!(formatter, "{:?}", self.0)
    }
}

impl ResponseError for ServerResponseError {
    fn status_code(&self) -> actix_http::StatusCode {
        if let Some(package_server_error) = self.0.root_cause().downcast_ref::<PackageServerError>()
        {
            return match package_server_error {
                PackageServerError::MetadataParse => StatusCode::BAD_REQUEST,
                PackageServerError::FileSave => StatusCode::INTERNAL_SERVER_ERROR,
                PackageServerError::PackageSave => StatusCode::INTERNAL_SERVER_ERROR,
                PackageServerError::VersionParse => StatusCode::BAD_REQUEST,
                PackageServerError::VersionConflict => StatusCode::CONFLICT,
            };
        }
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::with_body(self.status_code(), format!("{}", self.0)).map_into_boxed_body()
    }
}

impl From<anyhow::Error> for ServerResponseError {
    fn from(error: anyhow::Error) -> ServerResponseError {
        ServerResponseError(error)
    }
}
