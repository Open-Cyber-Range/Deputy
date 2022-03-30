use actix_web::{
    put,
    web::{Bytes, Data},
    HttpResponse,
};
use deputy_library::{package::Package, validation::Validate};
use log::error;

use crate::AppState;

#[put("package")]
pub async fn add_package(package_bytes: Bytes, app_state: Data<AppState>) -> HttpResponse {
    let package_vector = package_bytes.to_vec();

    match Package::try_from(&package_vector as &[u8]) {
        Ok(mut package) => {
            let folder = &app_state.package_folder;
            let repository = &app_state.repository;
            match package.metadata.validate() {
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
