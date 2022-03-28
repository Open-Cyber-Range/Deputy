use actix_web::web::Data;
use anyhow::Result;
use deputy_library::repository::{get_or_create_repository, RepositoryConfiguration};
use rand::Rng;
use std::path::PathBuf;

use crate::AppState;

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
