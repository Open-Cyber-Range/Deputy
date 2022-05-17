use crate::constants::endpoints::{LARGE_PACKAGE_UPLOAD_PATH, SMALL_PACKAGE_UPLOAD_PATH};
use anyhow::{anyhow, Result};
use awc::Client;
use deputy_library::package::PackageStream;
use log::error;

pub async fn stream_large_package(
    base_api_url: &str,
    client: &Client,
    stream: PackageStream,
) -> Result<()> {
    let put_uri = format!("{}{}", base_api_url, LARGE_PACKAGE_UPLOAD_PATH);
    let response = client
        .put(put_uri)
        .send_stream(stream)
        .await
        .map_err(|error| anyhow!("Failed to upload package: {}", error))?;
    if response.status().is_success() {
        return Ok(());
    }

    error!("Failed to upload package");
    Err(anyhow!("Failed to upload package"))
}

pub async fn upload_small_package(
    base_api_url: &str,
    client: &Client,
    payload: Vec<u8>,
) -> Result<()> {
    let put_uri = format!("{}{}", base_api_url, SMALL_PACKAGE_UPLOAD_PATH);
    let response = client
        .put(put_uri)
        .send_body(payload)
        .await
        .map_err(|error| anyhow!("Failed to upload package: {}", error))?;
    if response.status().is_success() {
        return Ok(());
    }

    error!("Failed to upload package");
    Err(anyhow!("Failed to upload package"))
}
