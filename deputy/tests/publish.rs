mod test_backend;

#[cfg(test)]
mod tests {
    use crate::test_backend::TestBackEnd;
    use anyhow::Result;
    use assert_cmd::Command;
    use deputy::client::Client;
    use deputy_library::{
        constants::CONFIGURATION_FOLDER_PATH_ENV_KEY,
        test::{create_test_package},
    };
    use deputy_package_server::test::TestPackageServer;
    use predicates::prelude::predicate;
    use tempfile::{Builder, TempDir};

    /* TODO Fix tests timeouting
    These tests fail because of a timeout caused by httpserver setup in
    deputy-package-server/src/test.rs, lines 107-125.
    */

    /*#[actix_web::test]
    async fn valid_small_package_was_sent_and_received() -> Result<()> {
        println!("1");
        let temp_project = TempArchive::builder().build()?;
        let toml_path = temp_project.toml_file.path().to_path_buf();
        let mut command = Command::cargo_bin("deputy")?;
        command.arg("publish");
        command.current_dir(temp_project.root_dir.path());
        println!("Path: {:?}", temp_project.root_dir.path());
        println!("2");
        let test_backend = TestBackEnd::builder().build().await?;

        println!("3");
        command.env(
            CONFIGURATION_FOLDER_PATH_ENV_KEY,
            &test_backend.configuration_directory.path(),
        );

        let temp_package = Package::from_file(None, toml_path, 0)?;
        let outbound_package_size = &temp_package.file.metadata().unwrap().len();
        println!("4");
        let saved_package_path: PathBuf = [
            &test_backend.configuration.storage_folders.package_folder,
            &temp_package.index_info.name,
            &temp_package.index_info.version,
        ]
        .iter()
        .collect();
        println!("5");
        println!("Command: {:?}", command);

        command.assert().success();
        let saved_package_size = fs::metadata(saved_package_path)?.len();
        assert_eq!(outbound_package_size, &saved_package_size);

        println!("6");
        temp_project.root_dir.close()?;
        test_backend.configuration_file.close()?;
        test_backend.configuration_directory.close()?;
        test_backend.test_repository_server.stop().await?;
        println!("7");

        Ok(())
    }
    */

    /*#[actix_web::test]
    async fn valid_large_package_was_sent_and_received() -> Result<()> {
        let temp_project = TempArchive::builder().is_large(true).build()?;
        let toml_path = temp_project.toml_file.path().to_path_buf();
        let mut command = Command::cargo_bin("deputy")?;
        command.arg("publish");
        command.current_dir(temp_project.root_dir.path());

        let test_backend = TestBackEnd::builder().build().await?;

        command.env(
            CONFIGURATION_FOLDER_PATH_ENV_KEY,
            &test_backend.configuration_directory.path(),
        );

        let temp_package = Package::from_file(None, toml_path, 0)?;
        let outbound_package_size = &temp_package.file.metadata().unwrap().len();
        let saved_package_path: PathBuf = [
            &test_backend.configuration.storage_folders.package_folder,
            &temp_package.index_info.name,
            &temp_package.index_info.version,
        ]
        .iter()
        .collect();

        command.assert().success();
        let saved_package_size = fs::metadata(saved_package_path)?.len();
        assert_eq!(outbound_package_size, &saved_package_size);

        temp_project.root_dir.close()?;
        test_backend.configuration_file.close()?;
        test_backend.configuration_directory.close()?;
        test_backend.test_repository_server.stop().await?;

        Ok(())
    }
    */

    #[actix_web::test]
    async fn error_on_missing_package_toml_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_dir = temp_dir.into_path().canonicalize()?;

        let test_backend = TestBackEnd::builder().build().await?;

        let mut command = Command::cargo_bin("deputy")?;
        command.arg("publish");
        command.current_dir(temp_dir);
        command.env(
            CONFIGURATION_FOLDER_PATH_ENV_KEY,
            &test_backend.configuration_directory.path(),
        );
        command.assert().failure().stderr(predicate::str::contains(
            "Error: Could not find package.toml",
        ));

        test_backend.configuration_file.close()?;
        test_backend.configuration_directory.close()?;

        Ok(())
    }

    /*
    #[actix_web::test]
    async fn error_on_missing_package_toml_content() -> Result<()> {
        let temp_dir = TempDir::new()?;

        let test_backend = TestBackEnd::builder().build().await?;

        let _package_toml = Builder::new()
            .prefix("package")
            .suffix(".toml")
            .rand_bytes(0)
            .tempfile_in(&temp_dir)?;
        let temp_dir = temp_dir.into_path().canonicalize()?;

        let mut command = Command::cargo_bin("deputy")?;
        command.arg("publish");
        command.current_dir(temp_dir);
        command.env(
            CONFIGURATION_FOLDER_PATH_ENV_KEY,
            &test_backend.configuration_directory.path(),
        );
        command
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error: missing field `package`"));

        test_backend.configuration_file.close()?;
        test_backend.configuration_directory.close()?;

        Ok(())
    }

    #[actix_web::test]
    async fn accepts_valid_small_package() -> Result<()> {
        let package = create_test_package()?;
        let (_configuration, server_address) = TestPackageServer::setup_test_server().await?;

        let client = Client::try_new(server_address.to_string())?;
        let response = client.upload_package(package.to_stream().await?, 60).await;

        assert!(response.is_ok());
        Ok(())
    }
    */

    /*#[actix_web::test]
    async fn valid_small_package_was_sent_and_received_with_non_default_registry() -> Result<()> {
        let temp_project = TempArchive::builder().build()?;
        let toml_path = temp_project.toml_file.path().to_path_buf();
        let registry_name = String::from("other-registry");
        let mut command = Command::cargo_bin("deputy")?;
        command
            .arg("publish")
            .arg("--registry-name")
            .arg(&registry_name);
        command.current_dir(temp_project.root_dir.path());

        let test_backend = TestBackEnd::builder()
            .change_registry_name(registry_name)
            .build()
            .await?;

        command.env(
            CONFIGURATION_FOLDER_PATH_ENV_KEY,
            &test_backend.configuration_directory.path(),
        );

        let temp_package = Package::from_file(None, toml_path, 0)?;
        let outbound_package_size = &temp_package.file.metadata().unwrap().len();
        let saved_package_path: PathBuf = [
            &test_backend.configuration.storage_folders.package_folder,
            &temp_package.index_info.name,
            &temp_package.index_info.version,
        ]
        .iter()
        .collect();

        command.assert().success();
        let saved_package_size = fs::metadata(saved_package_path)?.len();
        assert_eq!(outbound_package_size, &saved_package_size);

        temp_project.root_dir.close()?;
        test_backend.configuration_file.close()?;
        test_backend.configuration_directory.close()?;
        test_backend.test_repository_server.stop().await?;

        Ok(())
    }
    */
}
