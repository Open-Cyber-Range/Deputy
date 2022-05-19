mod common;

#[cfg(test)]
mod tests {
    use crate::common::create_temp_configuration_file;
    use anyhow::Result;
    use assert_cmd::Command;
    use deputy::{client::Client, constants::CONFIG_FILE_PATH_ENV_KEY};
    use deputy_library::{
        package::Package,
        test::{create_temp_project, TEST_PACKAGE_BYTES},
    };
    use deputy_package_server::test::{generate_server_test_configuration, start_test_server};
    use predicates::prelude::predicate;
    use std::{fs, path::PathBuf};
    use tempfile::{Builder, TempDir};

    #[actix_web::test]
    async fn valid_package_was_sent_and_received() -> Result<()> {
        let temp_project = create_temp_project()?;
        let toml_path = temp_project.toml_file.path().to_path_buf();
        let temp_package = Package::from_file(toml_path)?;
        let outbound_package_size = &temp_package.file.metadata().unwrap().len();
        let (configuration, server_address) = generate_server_test_configuration(9090)?;
        let saved_package_path: PathBuf = [
            &configuration.package_folder,
            &temp_package.metadata.name,
            &temp_package.metadata.version,
        ]
        .iter()
        .collect();

        start_test_server(configuration.clone()).await?;
        let client = Client::new(server_address);
        let response = client.upload_small_package(temp_package.try_into()?).await;
        let saved_package_size = fs::metadata(saved_package_path)?.len();
        assert!(response.is_ok());
        assert_eq!(outbound_package_size, &saved_package_size);

        temp_project.root_dir.close()?;
        fs::remove_dir_all(&configuration.package_folder)?;
        fs::remove_dir_all(&configuration.repository.folder)?;
        Ok(())
    }

    #[test]
    fn error_on_missing_package_toml_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_dir = temp_dir.into_path().canonicalize()?;
        let (configuration_directory, configuration_file) = create_temp_configuration_file()?;
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
        let (configuration_directory, configuration_file) = create_temp_configuration_file()?;
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
        let (configuration_directory, configuration_file) = create_temp_configuration_file()?;
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
    async fn rejected_put_request() -> Result<()> {
        let invalid_package_bytes: Vec<u8> = vec![124, 0, 0, 0, 123, 34, 110, 97, 109, 101, 34, 58];
        let (configuration, server_address) = generate_server_test_configuration(9091)?;
        start_test_server(configuration).await?;
        let client = Client::new(server_address);
        let response = client.upload_small_package(invalid_package_bytes).await;

        assert!(response.is_err());
        Ok(())
    }

    #[actix_web::test]
    async fn accepted_put_request() -> Result<()> {
        let package_bytes = TEST_PACKAGE_BYTES.clone();
        let (configuration, server_address) = generate_server_test_configuration(9092)?;
        start_test_server(configuration.clone()).await?;

        let client = Client::new(server_address.to_string());
        let response = client.upload_small_package(package_bytes).await;
        assert!(response.is_ok());
        fs::remove_dir_all(&configuration.package_folder)?;
        fs::remove_dir_all(&configuration.repository.folder)?;
        Ok(())
    }
}
