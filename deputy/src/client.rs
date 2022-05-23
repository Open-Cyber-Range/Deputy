use crate::constants::endpoints::{LARGE_PACKAGE_UPLOAD_PATH, SMALL_PACKAGE_UPLOAD_PATH};
use anyhow::{anyhow, Error, Result};
use awc::Client as ActixWebClient;
use deputy_library::package::PackageStream;
use log::error;
use std::str::from_utf8;

pub struct Client {
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

    fn response_to_error(message: &str, payload: Vec<u8>) -> Result<Error> {
        let error_message = format!("{message}: {}", from_utf8(&payload)?);
        error!("{error_message}");
        Ok(anyhow!(error_message))
    }

    pub async fn stream_large_package(&self, stream: PackageStream) -> Result<()> {
        let put_uri = format!("{}{}", self.api_base_url, LARGE_PACKAGE_UPLOAD_PATH);
        let mut response = self
            .client
            .put(put_uri)
            .send_stream(stream)
            .await
            .map_err(|error| anyhow!("Failed to upload package: {}", error))?;
        if response.status().is_success() {
            return Ok(());
        }

        Err(Client::response_to_error(
            "Failed to upload package",
            response.body().await?.to_vec(),
        )?)
    }

    pub async fn upload_small_package(&self, payload: Vec<u8>) -> Result<()> {
        let put_uri = format!("{}{}", self.api_base_url, SMALL_PACKAGE_UPLOAD_PATH);
        let mut response = self
            .client
            .put(put_uri)
            .send_body(payload)
            .await
            .map_err(|error| anyhow!("Failed to upload package: {}", error))?;
        if response.status().is_success() {
            return Ok(());
        }

        Err(Client::response_to_error(
            "Failed to upload package",
            response.body().await?.to_vec(),
        )?)
    }
}
