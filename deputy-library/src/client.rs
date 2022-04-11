use crate::constants::{PACKAGE_PUT_URL, PACKAGE_TOML};
use crate::package::PackageMetadata;
use crate::{
    archiver,
    package::{Package, PackageFile},
};
use anyhow::{anyhow, Result};
use reqwest::StatusCode;
use std::{fs::File, path::PathBuf};

pub async fn publishing_put_request(put_url: &str, package_bytes: Vec<u8>) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client.put(put_url).body(package_bytes).send().await?;
    match response.status() {
        StatusCode::OK => {
            println!("Package uploaded successfully");
        }
        _ => {
            println!("{}", response.text().await?)
        }
    }
    Ok(())
}

fn find_toml(mut toml_path: PathBuf) -> Result<PathBuf> {
    if toml_path.is_file() {
        Ok(toml_path)
    } else if toml_path.pop() && toml_path.pop() {
        let toml_path = toml_path.join(PACKAGE_TOML);
        Ok(find_toml(toml_path)?)
    } else {
        Err(anyhow!("Could not find package.toml"))
    }
}

pub fn create_package_from_toml(toml_path: PathBuf) -> Result<Package> {
    let package_root = toml_path
        .parent()
        .ok_or_else(|| anyhow!("Directory error"))?;
    let archive_path = archiver::create_package(package_root.to_path_buf())?;
    let metadata = PackageMetadata::gather_metadata(toml_path, &archive_path)?;
    let file = File::open(&archive_path)?;
    let package = Package {
        metadata,
        file: PackageFile(file),
    };
    Ok(package)
}

pub async fn create_and_send_package_file(execution_directory: PathBuf) -> Result<()> {
    let package_toml = PathBuf::from(PACKAGE_TOML);
    let toml_path = [&execution_directory, &package_toml].iter().collect();
    let toml_path = find_toml(toml_path)?;
    let package = create_package_from_toml(toml_path)?;
    let package_bytes = Vec::try_from(package)?;
    publishing_put_request(PACKAGE_PUT_URL, package_bytes).await?;
    Ok(())
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
