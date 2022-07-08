mod common;
mod repository;

#[cfg(test)]
mod tests {
    use crate::repository::MockRepositoryServer;
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

        let index_repository_mocker = MockRepositoryServer::try_new().await?;

        command.env(
            CONFIG_FILE_PATH_ENV_KEY,
            &index_repository_mocker.get_configuration_file().path(),
        );

        let temp_package = Package::from_file(toml_path, 0)?;
        let outbound_package_size = &temp_package.file.metadata().unwrap().len();
        let saved_package_path: PathBuf = [
            &index_repository_mocker.get_configuration().package_folder,
            &temp_package.metadata.name,
            &temp_package.metadata.version,
        ]
        .iter()
        .collect();

        index_repository_mocker.start().await?;

        command.assert().success();
        let saved_package_size = fs::metadata(saved_package_path)?.len();
        assert_eq!(outbound_package_size, &saved_package_size);

        temp_project.root_dir.close()?;
        fs::remove_dir_all(&index_repository_mocker.get_configuration().package_folder)?;
        fs::remove_dir_all(
            &index_repository_mocker
                .get_configuration()
                .repository
                .folder,
        )?;
        index_repository_mocker.stop().await?;

        Ok(())
    }

    #[actix_web::test]
    async fn valid_large_package_was_sent_and_received() -> Result<()> {
        let temp_project = TempArchive::builder().is_large(true).build()?;
        let toml_path = temp_project.toml_file.path().to_path_buf();
        let mut command = Command::cargo_bin("deputy")?;
        command.arg("publish");
        command.current_dir(temp_project.root_dir.path());

        let index_repository_mocker = MockRepositoryServer::try_new().await?;

        command.env(
            CONFIG_FILE_PATH_ENV_KEY,
            &index_repository_mocker.get_configuration_file().path(),
        );

        let temp_package = Package::from_file(toml_path, 0)?;
        let outbound_package_size = &temp_package.file.metadata().unwrap().len();
        let saved_package_path: PathBuf = [
            &index_repository_mocker.get_configuration().package_folder,
            &temp_package.metadata.name,
            &temp_package.metadata.version,
        ]
        .iter()
        .collect();

        index_repository_mocker.start().await?;
        command.assert().success();
        let saved_package_size = fs::metadata(saved_package_path)?.len();
        assert_eq!(outbound_package_size, &saved_package_size);

        temp_project.root_dir.close()?;
        fs::remove_dir_all(&index_repository_mocker.get_configuration().package_folder)?;
        fs::remove_dir_all(
            &index_repository_mocker
                .get_configuration()
                .repository
                .folder,
        )?;
        index_repository_mocker.stop().await?;
        Ok(())
    }

    #[actix_web::test]
    async fn error_on_missing_package_toml_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_dir = temp_dir.into_path().canonicalize()?;
        let index_repository_mocker = MockRepositoryServer::try_new().await?;

        index_repository_mocker.start().await?;
        let mut command = Command::cargo_bin("deputy")?;
        command.arg("publish");
        command.current_dir(temp_dir);
        command.env(
            CONFIG_FILE_PATH_ENV_KEY,
            &index_repository_mocker.get_configuration_file().path(),
        );
        command.assert().failure().stderr(predicate::str::contains(
            "Error: Could not find package.toml",
        ));
        index_repository_mocker.stop().await?;
        Ok(())
    }

    #[actix_web::test]
    async fn error_on_missing_package_toml_content() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let index_repository_mocker = MockRepositoryServer::try_new().await?;

        let _package_toml = Builder::new()
            .prefix("package")
            .suffix(".toml")
            .rand_bytes(0)
            .tempfile_in(&temp_dir)?;
        let temp_dir = temp_dir.into_path().canonicalize()?;

        index_repository_mocker.start().await?;
        let mut command = Command::cargo_bin("deputy")?;
        command.arg("publish");
        command.current_dir(temp_dir);
        command.env(
            CONFIG_FILE_PATH_ENV_KEY,
            &index_repository_mocker.get_configuration_file().path(),
        );
        command
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error: missing field `package`"));
        index_repository_mocker.stop().await?;
        Ok(())
    }

    #[actix_web::test]
    async fn rejects_invalid_small_package() -> Result<()> {
        let invalid_package_bytes: Vec<u8> = vec![124, 0, 0, 0, 123, 34, 110, 97, 109, 101, 34, 58];
        let (configuration, server_address) = generate_server_test_configuration()?;
        start_test_server(configuration).await?;
        let client = Client::try_new(server_address)?;
        let response = client.upload_small_package(invalid_package_bytes, 60).await;

        assert!(response.is_err());
        Ok(())
    }

    #[actix_web::test]
    async fn accepts_valid_small_package() -> Result<()> {
        let package_bytes = TEST_PACKAGE_BYTES.clone();
        let (configuration, server_address) = generate_server_test_configuration()?;
        start_test_server(configuration.clone()).await?;

        let client = Client::try_new(server_address.to_string())?;
        let response = client.upload_small_package(package_bytes, 60).await;

        assert!(response.is_ok());
        fs::remove_dir_all(&configuration.package_folder)?;
        fs::remove_dir_all(&configuration.repository.folder)?;
        Ok(())
    }
}
