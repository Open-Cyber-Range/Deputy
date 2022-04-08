use actix_web::{
    web::{scope, Data},
    App, HttpServer,
};
use anyhow::Result;
use deputy_library::repository::{get_or_create_repository, RepositoryConfiguration};
use rand::Rng;
use std::path::PathBuf;

use crate::{
    configuration::Configuration, routes::basic::version, routes::package::add_package, AppState,
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CONFIGURATION: Configuration = Configuration {
        host: "localhost".to_string(),
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
        repository,
        package_folder: package_folder.to_str().unwrap().to_string(),
    }))
}
#[actix_web::main]
async fn real_main(configuration: Configuration) -> Result<()> {
    HttpServer::new(move || {
        let package_folder = configuration.package_folder.clone();

        if let Result::Ok(repository) = get_or_create_repository(&configuration.repository) {
            let app_data = Data::new(AppState {
                repository,
                package_folder,
            });
            return App::new()
                .app_data(app_data)
                .service(scope("/api/v1").service(add_package))
                .service(version);
        }
        panic!("Failed to get the repository for keeping the index");
    })
    .bind((configuration.host, configuration.port))?
    .run()
    .await?;
    Ok(())
}
pub fn start_test_server(configuration: Configuration) {
    std::thread::spawn(|| real_main(configuration));
    std::thread::sleep(std::time::Duration::from_millis(1000));
}
