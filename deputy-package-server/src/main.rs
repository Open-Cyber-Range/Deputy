use actix::Actor;
use actix_web::{
    web::{scope, Data},
    App, HttpServer,
};
use anyhow::{Ok, Result};
use deputy_package_server::{
    configuration::read_configuration,
    routes::{
        basic::{status, version},
        package::{add_package, download_package, download_file_by_path, get_all_packages,
                  get_all_latest_packages, get_all_versions, get_metadata, get_readme,
                  try_get_latest_version, version_exists},
    },
    AppState,
};
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

    let app_state = AppState {
        storage_folders: configuration.storage_folders,
        database_address: database,
    };

    HttpServer::new(move || {
        let app_data = Data::new(app_state.clone());
        App::new().app_data(app_data).service(
            scope("/api").service(status).service(version).service(
                scope("/v1")
                    .service(get_all_packages)
                    .service(get_all_latest_packages)
                    .service(get_all_versions)
                    .service(add_package)
                    .service(download_package)
                    .service(download_file_by_path)
                    .service(get_readme)
                    .service(get_metadata)
                    .service(try_get_latest_version)
                    .service(version_exists),
            ),
        )
    })
    .bind((configuration.host, configuration.port))?
    .run()
    .await?;
    Ok(())
}

#[actix_web::main]
async fn main() {
    if let Err(error) = real_main().await {
        panic!("Failed to start the app due to: {:}", error);
    };
}
