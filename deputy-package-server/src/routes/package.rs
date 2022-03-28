use actix_web::{
    put,
    web::{Bytes, Data},
    HttpResponse,
};
use deputy_library::package::Package;
use log::error;

use crate::AppState;

#[put("package")]
pub async fn add_package(package_bytes: Bytes, app_state: Data<AppState>) -> HttpResponse {
    let package_vector = package_bytes.to_vec();

    match Package::try_from(&package_vector as &[u8]) {
        Ok(mut package) => {
            let folder = &app_state.package_folder;
            let repository = &app_state.repository;
            match package.save(folder.clone(), repository) {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(error) => {
                    error!("Failed to save package: {:?}", error);
                    HttpResponse::UnprocessableEntity().body("Failed to save the package")
                }
            }
        }
        Err(error) => {
            error!("Failed to parse package body: {:?}", error);
            HttpResponse::UnprocessableEntity().body("Failed to parse bytes")
        }
    }
}
