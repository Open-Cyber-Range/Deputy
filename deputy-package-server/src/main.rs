mod routes;

use crate::routes::basic::{status, version};
use actix_web::{web::scope, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(scope("/").service(status).service(version)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
