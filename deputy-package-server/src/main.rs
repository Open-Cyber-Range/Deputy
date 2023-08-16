use actix::Actor;
use actix_web::{
    web::{get, post, put, scope, Data},
    App, HttpServer,
};
use anyhow::{Ok, Result};
use deputy_package_server::routes::package::search_packages;
use deputy_package_server::{
    configuration::read_configuration,
    middleware::authentication::{
        jwt::AuthenticationMiddlewareFactory,
        local_token::LocalTokenAuthenticationMiddlewareFactory,
    },
    routes::{
        apitoken::{create_api_token, get_all_api_tokens},
        basic::{status, version},
        package::{
            add_package, download_file, download_package, get_all_packages, get_all_versions,
            get_package_version,
        },
    },
    services::database::Database,
    AppState,
};

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
        package_folder: configuration.package_folder,
        database_address: database,
    };

    HttpServer::new(move || {
        let auth_middleware =
            AuthenticationMiddlewareFactory(configuration.keycloak.pem_content.clone());
        let app_data = Data::new(app_state.clone());
        App::new()
            .app_data(app_data)
            .service(status)
            .service(version)
            .service(
                scope("/api").service(
                    scope("/v1")
                        .service(
                            scope("/package")
                                .service(
                                    scope("/{package_name}")
                                        .route("", get().to(get_all_versions::<Database>))
                                        .service(
                                            scope("/{version}")
                                                .route(
                                                    "/download",
                                                    get().to(download_package::<Database>),
                                                )
                                                .route(
                                                    "/path/{tail:.*}",
                                                    get().to(download_file::<Database>),
                                                )
                                                .route(
                                                    "",
                                                    get().to(get_package_version::<Database>),
                                                ),
                                        ),
                                )
                                .service(
                                    scope("")
                                        .service(
                                            scope("").route("", put().to(add_package::<Database>)),
                                        )
                                        .wrap(LocalTokenAuthenticationMiddlewareFactory),
                                )
                                .route("", get().to(get_all_packages::<Database>)),
                        )
                        .service(scope("/search").route("", get().to(search_packages::<Database>))),
                        .service(
                            scope("/token")
                                .service(
                                    scope("")
                                        .route("", post().to(create_api_token::<Database>))
                                        .route("", get().to(get_all_api_tokens::<Database>)),
                                )
                                .wrap(auth_middleware),
                        ),
                ),
            )
    })
    .bind(configuration.hostname)?
    .run()
    .await?;
    Ok(())
}

#[actix_web::main]
async fn main() {
    if let Err(error) = real_main().await {
        panic!("Failed to start the app due to: {error}");
    };
}
