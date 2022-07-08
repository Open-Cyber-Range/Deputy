mod common;
mod repository;

#[cfg(test)]
mod tests {
    use crate::repository::MockRepositoryServer;
    use anyhow::Result;
    use assert_cmd::Command;
    use deputy::{client::Client, constants::CONFIG_FILE_PATH_ENV_KEY};
    use deputy_library::test::TEST_PACKAGE_BYTES;
    use std::fs;
    use tempfile::TempDir;

    #[actix_web::test]
    async fn get_package_checksum() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let package_bytes = TEST_PACKAGE_BYTES.clone();
        let index_repository_mocker = MockRepositoryServer::try_new().await?;
        index_repository_mocker.start().await?;
        let client = Client::try_new(index_repository_mocker.get_server_address().to_string())?;
        let response = client.upload_small_package(package_bytes.clone(), 60).await;
        assert!(response.is_ok());

        let mut command = Command::cargo_bin("deputy")?;
        command.arg("checksum").arg("some-package-name");
        command.current_dir(&temp_dir);
        command.env(
            CONFIG_FILE_PATH_ENV_KEY,
            &index_repository_mocker.get_configuration_file().path(),
        );
        command.assert().success();
        command
            .assert()
            .stdout("aa30b1cc05c10ac8a1f309e3de09de484c6de1dc7c226e2cf8e1a518369b1d73\n");
        index_repository_mocker.stop().await?;

        fs::remove_dir_all(&index_repository_mocker.get_configuration().package_folder)?;
        fs::remove_dir_all(
            &index_repository_mocker
                .get_configuration()
                .repository
                .folder,
        )?;
        Ok(())
    }
}
