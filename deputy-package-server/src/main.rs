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
        package::add_package,
        package::download_package,
    },
    AppState,
};
use log::error;

async fn real_main() -> Result<()> {
    env_logger::init();
    let configuration = read_configuration(std::env::args().collect())?;

    HttpServer::new(move || {
        let package_folder = configuration.package_folder.clone();
        if let Result::Ok(repository) = get_or_create_repository(&configuration.repository) {
            let app_data = Data::new(AppState {
                repository,
                package_folder,
            });
            return App::new()
                .app_data(app_data)
                .service(status)
                .service(version)
                .service(scope("/api/v1").service(add_package))
                .service(scope("/api/v1").service(download_package));
        }
        error!("Failed to get the repository for keeping the index");
        panic!("Failed to get the repository for keeping the index");
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
