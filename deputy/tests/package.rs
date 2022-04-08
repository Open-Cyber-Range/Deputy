#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;
    use deputy_library::client::create_package_from_toml;
    use deputy_library::test::create_temp_project;
    use deputy_package_server::test::{start_test_server, CONFIGURATION};

    #[actix_web::test]
    async fn get_test_server_version() -> Result<()> {
        start_test_server(CONFIGURATION.to_owned());
        let version_response = reqwest::get("http://localhost:9090/version")
            .await?
            .text()
            .await?;
        println!("{:?}", version_response);

        Ok(())
    }

    #[actix_web::test]
    async fn valid_package_was_received() -> Result<()> {
        let temp_project = create_temp_project()?;
        let toml_path = temp_project.toml_file.path().to_path_buf();
        let temp_package = create_package_from_toml(toml_path)?;
        let package_bytes = Vec::try_from(temp_package)?;

        start_test_server(CONFIGURATION.to_owned());

        let client = reqwest::Client::new();
        let response = client
            .put("http://localhost:9090/api/v1/package")
            .body(package_bytes)
            .send()
            .await?;

        assert!(response.status().is_success()); //ADD ACTUAL FILE CHECKING
        temp_project.root_dir.close()?;
        fs::remove_dir_all("/tmp/test-packages")?; //?
        fs::remove_dir_all("/tmp/test-repo")?;
        Ok(())
    }
}
