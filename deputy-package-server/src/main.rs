use actix_web::{web::scope, App, HttpServer};
use deputy_package_server_routes::basic::{status, version};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(scope("/").service(status).service(version)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
