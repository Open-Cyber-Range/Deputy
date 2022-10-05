mod repository;
mod test_backend;

#[cfg(test)]
mod tests {
    use crate::test_backend::TestBackEnd;
    use anyhow::Result;
    use assert_cmd::Command;
    use deputy::client::Client;
    use deputy_library::{constants::CONFIGURATION_FOLDER_PATH_ENV_KEY, test::create_test_package};
    use tempfile::TempDir;

    #[actix_web::test]
    async fn downloads_package() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let package = create_test_package()?;
        let test_backend = TestBackEnd::builder().build().await?;

        let client = Client::try_new(test_backend.server_address.to_string())?;
        let response = client.upload_package(package.to_stream().await?, 60).await;
        assert!(response.is_ok());

        let mut command = Command::cargo_bin("deputy")?;
        command.arg("fetch").arg("some-package-name");
        command.current_dir(&temp_dir);
        command.env(
            CONFIGURATION_FOLDER_PATH_ENV_KEY,
            &test_backend.configuration_directory.path(),
        );
        command.assert().success();

        test_backend.configuration_file.close()?;
        test_backend.configuration_directory.close()?;
        test_backend.test_repository_server.stop().await?;

        assert!(&temp_dir.path().join("some-package-name-0.1.0").exists());
        Ok(())
    }
}
