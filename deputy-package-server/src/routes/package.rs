use actix_web::{put, web::Bytes, HttpResponse};
use deputy_library::package::Package;
use log::error;

#[put("package")]
pub async fn add_package(package_bytes: Bytes) -> HttpResponse {
    let package_vector = package_bytes.to_vec();

    match Package::try_from(&package_vector as &[u8]) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(error) => {
            error!("Failed to parse package body: {:?}", error);
            HttpResponse::UnprocessableEntity().body("Failed to parse bytes")
        }
    }
}
