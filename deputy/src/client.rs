use crate::constants::endpoints::{LARGE_PACKAGE_UPLOAD_PATH, SMALL_PACKAGE_UPLOAD_PATH};
use anyhow::{anyhow, Result};
use awc::Client as ActixWebClient;
use deputy_library::package::PackageStream;
use log::error;

pub(crate) struct Client {
    client: ActixWebClient,
    api_base_url: String,
}

impl Client {
    pub fn new(api_base_url: String) -> Self {
        Self {
            client: ActixWebClient::new(),
            api_base_url,
        }
    }

    pub async fn stream_large_package(&self, stream: PackageStream) -> Result<()> {
        let put_uri = format!("{}{}", self.api_base_url, LARGE_PACKAGE_UPLOAD_PATH);
        let response = self
            .client
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

    pub async fn upload_small_package(&self, payload: Vec<u8>) -> Result<()> {
        let put_uri = format!("{}{}", self.api_base_url, SMALL_PACKAGE_UPLOAD_PATH);
        let response = self
            .client
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
}
