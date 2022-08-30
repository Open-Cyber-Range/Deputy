use crate::{
    configuration::Configuration,
    routes::package::{add_package, download_package},
    AppState,
};
use actix_web::{
    web::{scope, Data},
    App, HttpServer,
};
use anyhow::{anyhow, Error, Result};
use deputy_library::{
    repository::{get_or_create_repository, RepositoryConfiguration},
    test::{generate_random_string, get_free_port},
    StorageFolders,
};
use futures::lock::Mutex;
use futures::TryFutureExt;
use lazy_static::lazy_static;
use std::fs;
use std::path::{Path, PathBuf};
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
        storage_folders: StorageFolders {
            package_folder: "/tmp/packages".to_string(),
            toml_folder: "/tmp/package-tomls".to_string(),
            readme_folder: "/tmp/readmes".to_string(),
        },
    };
}

pub struct TestPackageServer {
    configuration: Configuration,
    server_address: String,
}

impl TestPackageServer {
    pub async fn setup_test_server() -> Result<(Configuration, String)> {
        let test_server = Self::try_new()?;
        let (configuration, server_address) = test_server.get_configuration_and_server_address();
        test_server.start().await?;
        Ok((configuration, server_address))
    }

    pub fn try_new() -> Result<Self> {
        let (configuration, server_address) = Self::create_configuration()?;
        Ok(Self {
            configuration,
            server_address,
        })
    }

    fn create_configuration() -> Result<(Configuration, String)> {
        let mut configuration = CONFIGURATION.clone();
        configuration.port = get_free_port()?;
        configuration.repository.folder = format!(
            "{}-{}",
            configuration.repository.folder,
            generate_random_string(10)?
        );
        configuration.storage_folders.package_folder = format!(
            "{}-{}",
            configuration.storage_folders.package_folder,
            generate_random_string(10)?
        );
        let server_address = format!("http://{}:{}", configuration.host, configuration.port);
        Ok((configuration, server_address))
    }

    async fn initialize(&self, tx: Sender<()>) -> Result<()> {
        let configuration = self.configuration.clone();
        let package_folder = configuration.storage_folders.package_folder;
        let toml_folder = configuration.storage_folders.toml_folder;
        let readme_folder = configuration.storage_folders.readme_folder;
        if let Ok(repository) = get_or_create_repository(&configuration.repository) {
            let app_data = AppState {
                repository: Arc::new(Mutex::new(repository)),
                storage_folders: StorageFolders {
                    package_folder,
                    toml_folder,
                    readme_folder,
                },
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

    pub async fn start(self) -> Result<()> {
        let (tx, rx) = channel::<()>();
        tokio::spawn(async move { self.initialize(tx).await });
        timeout(std::time::Duration::from_millis(1000), rx).await??;

        Ok(())
    }

    pub fn get_configuration_and_server_address(&self) -> (Configuration, String) {
        (self.configuration.clone(), self.server_address.clone())
    }
}

impl Drop for TestPackageServer {
    fn drop(&mut self) {
        if Path::new(&self.configuration.storage_folders.package_folder).is_dir() {
            fs::remove_dir_all(&self.configuration.storage_folders.package_folder).unwrap();
        }
        if Path::new(&self.configuration.repository.folder).is_dir() {
            fs::remove_dir_all(&self.configuration.repository.folder).unwrap();
        }
    }
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
    let toml_folder: PathBuf = temporary_directory.join(format!("test-toml-folder-{}", randomizer));
    std::fs::create_dir_all(&toml_folder)?;
    let readme_folder: PathBuf =
        temporary_directory.join(format!("test-readme-folder-{}", randomizer));
    std::fs::create_dir_all(&readme_folder)?;

    let repository_configuration = RepositoryConfiguration {
        username: String::from("test-username"),
        email: String::from("test@email.com"),
        folder: repository_folder.to_str().unwrap().to_string(),
    };
    let repository = get_or_create_repository(&repository_configuration)?;

    Ok(Data::new(AppState {
        repository: Arc::new(Mutex::new(repository)),
        storage_folders: StorageFolders {
            package_folder: package_folder.to_str().unwrap().to_string(),
            toml_folder: toml_folder.to_str().unwrap().to_string(),
            readme_folder: readme_folder.to_str().unwrap().to_string(),
        },
    }))
}
