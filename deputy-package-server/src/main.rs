use actix::Actor;
use actix_web::{
    web::{scope, Data},
    App, HttpServer,
};
use anyhow::{Ok, Result};
use deputy_library::repository::get_or_create_repository;
use deputy_package_server::{
    configuration::read_configuration,
    routes::{
        basic::{status, version},
        package::{add_package, download_package, get_all_packages},
    },
    AppState,
};
use futures::lock::Mutex;
use log::error;
use std::sync::Arc;
use deputy_package_server::services::database::Database;

async fn real_main() -> Result<()> {
    env_logger::init();
    let configuration = read_configuration(std::env::args().collect())?;
    let database = Database::try_new(&configuration.database_url)
        .unwrap_or_else(|error| {
            panic!(
                "Failed to create database connection to {} due to: {error}",
                &configuration.database_url
            )
        })
        .start();

    if let Result::Ok(repository) = get_or_create_repository(&configuration.repository) {
        let app_state = AppState {
            repository: Arc::new(Mutex::new(repository)),
            storage_folders: configuration.storage_folders,
            database_address: database,
        };

        HttpServer::new(move || {
            let app_data = Data::new(app_state.clone());
            App::new().app_data(app_data).service(
                scope("/api").service(status).service(version).service(
                    scope("/v1")
                        .service(get_all_packages)
                        .service(add_package)
                        .service(download_package),
                ),
            )
        })
        .bind((configuration.host, configuration.port))?
        .run()
        .await?;
        return Ok(());
    }
    error!("Failed to get the repository for keeping the index");
    panic!("Failed to get the repository for keeping the index");
}

#[actix_web::main]
async fn main() {
    if let Err(error) = real_main().await {
        panic!("Failed to start the app due to: {:}", error);
    };
}
