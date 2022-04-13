#[cfg(test)]
mod tests {
    use anyhow::Result;
    use deputy_library::package::create_package_from_toml;
    use deputy_library::test::create_temp_project;
    use deputy_package_server::test::{start_test_server, CONFIGURATION};
    use std::{fs, path::PathBuf};

    #[actix_web::test]
    async fn valid_package_was_sent_and_received() -> Result<()> {
        let temp_project = create_temp_project()?;
        let toml_path = temp_project.toml_file.path().to_path_buf();
        let temp_package = create_package_from_toml(toml_path)?;
        let outbound_package_size = &temp_package.file.metadata().unwrap().len();
        let saved_package_path: PathBuf = [
            &CONFIGURATION.package_folder,
            &temp_package.metadata.name,
            &temp_package.metadata.version,
        ]
        .iter()
        .collect();

        let package_bytes = Vec::try_from(temp_package)?;
        start_test_server(CONFIGURATION.to_owned());
        let client = reqwest::Client::new();
        let response = client
            .put("http://localhost:9090/api/v1/package")
            .body(package_bytes)
            .send()
            .await?;
        let saved_package_size = fs::metadata(saved_package_path)?.len();
        assert!(response.status().is_success());
        assert_eq!(outbound_package_size, &saved_package_size);

        temp_project.root_dir.close()?;
        fs::remove_dir_all(&CONFIGURATION.package_folder)?;
        fs::remove_dir_all(&CONFIGURATION.repository.folder)?;
        Ok(())
    }
}
