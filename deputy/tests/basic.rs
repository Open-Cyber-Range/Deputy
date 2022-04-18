#[cfg(test)]
mod tests {
    use anyhow::Result;
    use assert_cmd::prelude::*;
    use deputy_library::{
        client::upload_package, constants::CONFIG_FILE_PATH_ENV_KEY, test::TEST_PACKAGE_BYTES,
    };
    use deputy_package_server::test::{start_test_server, CONFIGURATION};
    use predicates::prelude::*;
    use std::{env, fs, io::Write, path::PathBuf, process::Command};
    use tempfile::{tempdir, Builder, NamedTempFile, TempDir};

    fn create_temp_configuration_file() -> Result<(TempDir, NamedTempFile)> {
        let configuration_file_contents = br#"    
                [repository]
                repositories = [{ dl = "dllink", api = "apilink" }]"#;
        let configuration_directory = tempdir()?;
        let mut configuration_file = Builder::new()
            .prefix("configuration")
            .suffix(".toml")
            .rand_bytes(0)
            .tempfile_in(&configuration_directory)?;
        configuration_file.write_all(configuration_file_contents)?;
        Ok((configuration_directory, configuration_file))
    }

    #[test]
    fn test_version() -> Result<()> {
        let mut command = Command::cargo_bin("deputy")?;
        let (configuration_directory, configuration_file) = create_temp_configuration_file()?;
        command.arg("version");
        command.env(CONFIG_FILE_PATH_ENV_KEY, &configuration_file.path());
        command
            .assert()
            .success()
            .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
        configuration_directory.close()?;
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
        start_test_server(CONFIGURATION.to_owned());
        let client = reqwest::Client::new();
        let result = upload_package(
            "http://localhost:9090/api/v1/package",
            invalid_package_bytes,
            client,
        )
        .await;
        assert!(result.is_err());
        Ok(())
    }

    #[actix_web::test]
    async fn accepted_put_request() -> Result<()> {
        let package_bytes = TEST_PACKAGE_BYTES.clone();
        start_test_server(CONFIGURATION.to_owned());

        let client = reqwest::Client::new();
        let response = client
            .put("http://localhost:9090/api/v1/package")
            .body(package_bytes)
            .send()
            .await?;

        assert!(response.status().is_success());
        fs::remove_dir_all(&CONFIGURATION.package_folder)?;
        fs::remove_dir_all(&CONFIGURATION.repository.folder)?;
        Ok(())
    }
}
