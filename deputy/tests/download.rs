mod common;
mod repository;

#[cfg(test)]
mod tests {
    use crate::common::create_temp_configuration_file;
    use crate::repository::MockRepostioryServer;
    use anyhow::Result;
    use assert_cmd::Command;
    use deputy::{client::Client, constants::CONFIG_FILE_PATH_ENV_KEY};
    use deputy_library::test::TEST_PACKAGE_BYTES;
    use deputy_package_server::test::{generate_server_test_configuration, start_test_server};
    use std::fs;
    use tempfile::TempDir;

    #[actix_web::test]
    async fn downloads_package() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let package_bytes = TEST_PACKAGE_BYTES.clone();
        let (configuration, server_address) = generate_server_test_configuration(9097)?;
        let index_repository_mocker =
            MockRepostioryServer::try_new(11015, &configuration.repository.folder).await?;
        let (configuration_directory, configuration_file) = create_temp_configuration_file(
            &server_address,
            index_repository_mocker.get_index_url(),
        )?;
        start_test_server(configuration.clone()).await?;
        index_repository_mocker.start().await?;
        let client = Client::try_new(server_address.to_string())?;
        let response = client.upload_small_package(package_bytes.clone(), 60).await;
        assert!(response.is_ok());

        let mut command = Command::cargo_bin("deputy")?;
        command.arg("fetch").arg("some-package-name");
        command.current_dir(&temp_dir);
        command.env(CONFIG_FILE_PATH_ENV_KEY, &configuration_file.path());
        command.assert().success();
        configuration_directory.close()?;
        index_repository_mocker.stop().await?;

        assert!(&temp_dir.path().join("some-package-name-0.1.0").exists());
        fs::remove_dir_all(&configuration.package_folder)?;
        fs::remove_dir_all(&configuration.repository.folder)?;
        Ok(())
    }
}
