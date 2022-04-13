use crate::constants::PACKAGE_TOML;
use anyhow::{anyhow, Result};
use log::{error, info};
use reqwest::StatusCode;
use std::path::PathBuf;

pub async fn upload_package(put_url: &str, package_bytes: Vec<u8>) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client.put(put_url).body(package_bytes).send().await?;
    match response.status() {
        StatusCode::OK => {
            info!("Package uploaded successfully");
            Ok(())
        }
        _ => {
            let response_text = response.text().await?;
            error!("Failed to upload package: {:?}", response_text);
            Err(anyhow!(response_text))
        }
    }
}

pub fn find_toml(mut toml_path: PathBuf) -> Result<PathBuf> {
    if toml_path.is_file() {
        Ok(toml_path)
    } else if toml_path.pop() && toml_path.pop() {
        let toml_path = toml_path.join(PACKAGE_TOML);
        Ok(find_toml(toml_path)?)
    } else {
        Err(anyhow!("Could not find package.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use tempfile::{Builder, TempDir};

    #[test]
    fn successfully_found_toml() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let package_toml = Builder::new()
            .prefix("package")
            .suffix(".toml")
            .rand_bytes(0)
            .tempfile_in(&temp_dir)?;

        assert!(find_toml(package_toml.path().to_path_buf())?.is_file());
        temp_dir.close()?;
        Ok(())
    }
}
