use actix_web::{
    web::{scope, Data},
    App, HttpServer,
};
use anyhow::{anyhow, Error, Result};
use deputy_library::repository::{get_or_create_repository, RepositoryConfiguration};
use rand::Rng;
use std::path::PathBuf;

use crate::{
    configuration::Configuration,
    routes::package::{add_package, download_package},
    AppState,
};
use futures::lock::Mutex;
use futures::TryFutureExt;
use lazy_static::lazy_static;
use std::sync::Arc;
use tokio::{
    sync::oneshot::{channel, Sender},
    time::timeout,
    try_join,
};

lazy_static! {
    pub static ref CONFIGURATION: Configuration = Configuration {
        host: "127.0.0.1".to_string(),
        port: 9090,
        repository: RepositoryConfiguration {
            folder: "/tmp/test-repo".to_string(),
            username: "some-username".to_string(),
            email: "some@email.com".to_string(),
        },
        package_folder: "/tmp/test-packages".to_string(),
    };
}

pub fn generate_random_string(length: usize) -> Result<String> {
    let random_bytes = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(length)
        .collect::<Vec<u8>>();
    Ok(String::from_utf8(random_bytes)?)
}

pub fn generate_server_test_configuration(port: u16) -> Result<(Configuration, String)> {
    let mut configuration = CONFIGURATION.clone();
    configuration.port = port;
    configuration.repository.folder = format!("/tmp/test-repo-{}", generate_random_string(10)?);
    configuration.package_folder = format!("/tmp/test-packages-{}", generate_random_string(10)?);
    let server_address = format!("http://{}:{}", configuration.host, configuration.port);
    Ok((configuration, server_address))
}

pub fn get_predictable_temporary_folders(randomizer: String) -> Result<(String, String)> {
    let temporary_directory = std::env::temp_dir();
    let package_folder: PathBuf =
        temporary_directory.join(format!("test-package-folder-{}", randomizer));
    let repository_folder: PathBuf =
        temporary_directory.join(format!("test-repository-folder-{}", randomizer));
    Ok((
        package_folder.to_str().unwrap().to_string(),
        repository_folder.to_str().unwrap().to_string(),
    ))
}

pub fn create_predictable_temporary_folders(randomizer: String) -> Result<(String, String)> {
    let (package_string, repository_string) = get_predictable_temporary_folders(randomizer)?;

    std::fs::create_dir_all(&PathBuf::from(package_string.clone()))?;
    std::fs::create_dir_all(&PathBuf::from(repository_string.clone()))?;
    Ok((package_string, repository_string))
}

pub fn create_test_app_state(randomizer: String) -> Result<Data<AppState>> {
    let temporary_directory = std::env::temp_dir();
    let package_folder: PathBuf =
        temporary_directory.join(format!("test-package-folder-{}", randomizer));
    std::fs::create_dir_all(&package_folder)?;
    let repository_folder: PathBuf =
        temporary_directory.join(format!("test-repository-folder-{}", randomizer));
    std::fs::create_dir_all(&repository_folder)?;

    let repository_configuration = RepositoryConfiguration {
        username: String::from("test-username"),
        email: String::from("test@email.com"),
        folder: repository_folder.to_str().unwrap().to_string(),
    };
    let repository = get_or_create_repository(&repository_configuration)?;

    Ok(Data::new(AppState {
        repository: Arc::new(Mutex::new(repository)),
        package_folder: package_folder.to_str().unwrap().to_string(),
    }))
}

async fn initialize_test_server(configuration: Configuration, tx: Sender<()>) -> Result<()> {
    let package_folder = configuration.package_folder.clone();
    if let Result::Ok(repository) = get_or_create_repository(&configuration.repository) {
        let app_data = AppState {
            repository: Arc::new(Mutex::new(repository)),
            package_folder,
        };
        try_join!(
            HttpServer::new(move || {
                let app_data = Data::new(app_data.clone());
                App::new().app_data(app_data).service(
                    scope("/api/v1")
                        .service(add_package)
                        .service(download_package),
                )
            })
            .bind((configuration.host, configuration.port))?
            .workers(1)
            .run()
            .map_err(|error| anyhow!("Failed to start the server: {:?}", error)),
            async move {
                tx.send(())
                    .map_err(|error| anyhow!("Failed to send message: {:?}", error))?;
                Ok::<(), Error>(())
            }
        )?;
        return Ok(());
    }

    Err(anyhow!("Failed to create the repository"))
}

pub async fn start_test_server(configuration: Configuration) -> Result<()> {
    let (tx, rx) = channel::<()>();
    tokio::spawn(async move { initialize_test_server(configuration, tx).await });
    timeout(std::time::Duration::from_millis(1000), rx).await??;

    Ok(())
}
