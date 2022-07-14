mod common;
mod repository;
mod test_backend;

#[cfg(test)]
mod tests {
    use crate::common::create_temp_configuration_file;
    use crate::test_backend::TestBackEnd;
    use anyhow::Result;
    use assert_cmd::Command;
    use deputy::{client::Client, constants::CONFIG_FILE_PATH_ENV_KEY};
    use deputy_library::test::TEST_PACKAGE_BYTES;
    use tempfile::TempDir;

    #[actix_web::test]
    async fn downloads_package() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let package_bytes = TEST_PACKAGE_BYTES.clone();
        let (_configuration, server_address, index_url, test_backend) =
            TestBackEnd::setup_test_backend().await?;
        let (configuration_directory, configuration_file) =
            create_temp_configuration_file(&server_address, &index_url)?;

        let client = Client::try_new(server_address.to_string())?;
        let response = client.upload_small_package(package_bytes.clone(), 60).await;
        assert!(response.is_ok());

        let mut command = Command::cargo_bin("deputy")?;
        command.arg("fetch").arg("some-package-name");
        command.current_dir(&temp_dir);
        command.env(CONFIG_FILE_PATH_ENV_KEY, &configuration_file.path());
        command.assert().success();

        configuration_directory.close()?;
        test_backend.test_repository_server.stop().await?;

        assert!(&temp_dir.path().join("some-package-name-0.1.0").exists());
        Ok(())
    }
}
