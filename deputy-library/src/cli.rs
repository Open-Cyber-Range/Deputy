use crate::{
    archiver,
    package::{Package, PackageFile},
};
use anyhow::{anyhow, Result};
use reqwest::StatusCode;
use std::{env, fs::File, path::PathBuf};
async fn publishing_put_request(package_bytes: Vec<u8>) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .put("http://localhost:8080/api/v1/package")
        .body(package_bytes)
        .send()
        .await?;
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
    if toml_path.exists() {
        Ok(toml_path)
    } else if toml_path.pop() && toml_path.pop() {
        let toml_path = toml_path.join("package.toml");
        Ok(find_toml(toml_path)?)
    } else {
        Err(anyhow!("Could not find package.toml"))
    }
}

pub async fn create_and_send_package_file() -> Result<()> {
    let current_path = env::current_dir()?;
    let package_toml = PathBuf::from("package.toml");
    let toml_path = [&current_path, &package_toml].iter().collect();
    let toml_path = find_toml(toml_path)?;
    if let Some(package_root) = toml_path.parent() {
        let archive_path = archiver::create_package(package_root.to_path_buf())?;
        let metadata = Package::parse_metadata(toml_path, &archive_path)?;
        let file = File::open(&archive_path)?;
        let package = Package {
            metadata,
            file: PackageFile(file),
        };
        let package_bytes = Vec::try_from(package)?;
        publishing_put_request(package_bytes).await?;
        Ok(())
    } else {
        Err(anyhow!("Directory error"))
    }
}
