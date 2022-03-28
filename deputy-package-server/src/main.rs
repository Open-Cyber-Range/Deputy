use actix_web::{
    web::{scope, Data},
    App, HttpServer,
};
use anyhow::{Ok, Result};
use deputy_package_server::{
    configuration::read_configuration,
    routes::{
        basic::{status, version},
        package::add_package,
    },
    AppState,
};

async fn real_main() -> Result<()> {
    env_logger::init();
    let configuration = read_configuration(std::env::args().collect())?;
    let port = configuration.port;
    let hostname = configuration.host.clone();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState {
                configuration: configuration.clone(),
            }))
            .service(scope("/").service(status).service(version))
            .service(scope("/api/v1/").service(add_package))
    })
    .bind((hostname, port))?
    .run()
    .await?;
    Ok(())
}

#[actix_web::main]
async fn main() {
    if let Err(error) = real_main().await {
        println!("Failed to start the app due to: {:}", error);
    };
}
