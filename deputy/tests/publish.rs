mod common;

#[cfg(test)]
mod tests {
    use crate::common::create_temp_configuration_file;
    use anyhow::Result;
    use assert_cmd::Command;
    use deputy::{client::Client, constants::CONFIG_FILE_PATH_ENV_KEY};
    use deputy_library::{
        package::Package,
        test::{TempArchive, TEST_PACKAGE_BYTES},
    };
    use deputy_package_server::test::{generate_server_test_configuration, start_test_server};
    use predicates::prelude::predicate;
    use std::{fs, path::PathBuf};
    use tempfile::{Builder, TempDir};

    #[actix_web::test]
    async fn valid_small_package_was_sent_and_received() -> Result<()> {
        let temp_project = TempArchive::builder().build()?;
        let toml_path = temp_project.toml_file.path().to_path_buf();
        let mut command = Command::cargo_bin("deputy")?;
        command.arg("publish");
        command.current_dir(temp_project.root_dir.path());

        let (server_configuration, server_address) = generate_server_test_configuration(9090)?;
        let (configuration_directory, configuration_file) =
            create_temp_configuration_file(&server_address)?;
        command.env(CONFIG_FILE_PATH_ENV_KEY, &configuration_file.path());

        let temp_package = Package::from_file(toml_path)?;
        let outbound_package_size = &temp_package.file.metadata().unwrap().len();
        let saved_package_path: PathBuf = [
            &server_configuration.package_folder,
            &temp_package.metadata.name,
            &temp_package.metadata.version,
        ]
        .iter()
        .collect();

        start_test_server(server_configuration.clone()).await?;
        command.assert().success();
        let saved_package_size = fs::metadata(saved_package_path)?.len();
        assert_eq!(outbound_package_size, &saved_package_size);

        temp_project.root_dir.close()?;
        fs::remove_dir_all(&server_configuration.package_folder)?;
        fs::remove_dir_all(&server_configuration.repository.folder)?;
        configuration_directory.close()?;
        Ok(())
    }

    #[actix_web::test]
    async fn valid_large_package_was_sent_and_received() -> Result<()> {
        let temp_project = TempArchive::builder().is_large(true).build()?;
        let toml_path = temp_project.toml_file.path().to_path_buf();
        let mut command = Command::cargo_bin("deputy")?;
        command.arg("publish");
        command.current_dir(temp_project.root_dir.path());
        println!("asddsasda");
        let (server_configuration, server_address) = generate_server_test_configuration(9091)?;
        let (configuration_directory, configuration_file) =
            create_temp_configuration_file(&server_address)?;
        command.env(CONFIG_FILE_PATH_ENV_KEY, &configuration_file.path());

        let temp_package = Package::from_file(toml_path)?;
        let outbound_package_size = &temp_package.file.metadata().unwrap().len();
        let saved_package_path: PathBuf = [
            &server_configuration.package_folder,
            &temp_package.metadata.name,
            &temp_package.metadata.version,
        ]
        .iter()
        .collect();

        start_test_server(server_configuration.clone()).await?;
        command.assert().success();
        let saved_package_size = fs::metadata(saved_package_path)?.len();
        assert_eq!(outbound_package_size, &saved_package_size);

        temp_project.root_dir.close()?;
        fs::remove_dir_all(&server_configuration.package_folder)?;
        fs::remove_dir_all(&server_configuration.repository.folder)?;
        configuration_directory.close()?;
        Ok(())
    }

    #[test]
    fn error_on_missing_package_toml_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_dir = temp_dir.into_path().canonicalize()?;
        let (configuration_directory, configuration_file) =
            create_temp_configuration_file("does-not-matter")?;
        let mut command = Command::cargo_bin("deputy")?;
        command.arg("publish");
        command.current_dir(temp_dir);
        command.env(CONFIG_FILE_PATH_ENV_KEY, &configuration_file.path());
        command.assert().failure().stderr(predicate::str::contains(
            "Error: Could not find package.toml",
        ));
        configuration_directory.close()?;
        Ok(())
    }

    #[test]
    fn error_on_missing_package_toml_content() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let (configuration_directory, configuration_file) =
            create_temp_configuration_file("does-not-matter")?;
        let _package_toml = Builder::new()
            .prefix("package")
            .suffix(".toml")
            .rand_bytes(0)
            .tempfile_in(&temp_dir)?;
        let temp_dir = temp_dir.into_path().canonicalize()?;

        let mut command = Command::cargo_bin("deputy")?;
        command.arg("publish");
        command.current_dir(temp_dir);
        command.env(CONFIG_FILE_PATH_ENV_KEY, &configuration_file.path());
        command
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error: missing field `package`"));
        configuration_directory.close()?;
        Ok(())
    }

    #[test]
    fn finds_invalid_package_toml_from_parent_folder() -> Result<()> {
        let root_temp_dir = TempDir::new()?;
        let (configuration_directory, configuration_file) =
            create_temp_configuration_file("does-not-matter")?;
        let deep_path: PathBuf = ["some", "where", "many", "layers", "deep"].iter().collect();
        let deep_path = root_temp_dir.path().join(deep_path);
        std::fs::create_dir_all(&deep_path)?;

        let _package_toml = Builder::new()
            .prefix("package")
            .suffix(".toml")
            .rand_bytes(0)
            .tempfile_in(&root_temp_dir)?;

        let mut command = Command::cargo_bin("deputy")?;
        command.arg("publish");
        command.current_dir(deep_path.as_path());
        command.env(CONFIG_FILE_PATH_ENV_KEY, &configuration_file.path());
        command
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error: missing field `package`"));
        configuration_directory.close()?;
        Ok(())
    }

    #[actix_web::test]
    async fn rejects_invalid_small_package() -> Result<()> {
        let invalid_package_bytes: Vec<u8> = vec![124, 0, 0, 0, 123, 34, 110, 97, 109, 101, 34, 58];
        let (configuration, server_address) = generate_server_test_configuration(9092)?;
        start_test_server(configuration).await?;
        let client = Client::new(server_address);
        let response = client.upload_small_package(invalid_package_bytes).await;

        assert!(response.is_err());
        Ok(())
    }

    #[actix_web::test]
    async fn accepts_valid_small_package() -> Result<()> {
        let package_bytes = TEST_PACKAGE_BYTES.clone();
        let (configuration, server_address) = generate_server_test_configuration(9093)?;
        start_test_server(configuration.clone()).await?;

        let client = Client::new(server_address.to_string());
        let response = client.upload_small_package(package_bytes).await;
        assert!(response.is_ok());
        fs::remove_dir_all(&configuration.package_folder)?;
        fs::remove_dir_all(&configuration.repository.folder)?;
        Ok(())
    }
}
