mod routes;

use crate::routes::basic::{status, version};
use actix_web::{web::scope, App, HttpServer};
use routes::package::add_package;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(scope("/").service(status).service(version))
            .service(scope("/api/v1/").service(add_package))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
