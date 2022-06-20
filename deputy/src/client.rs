use crate::{
    constants::endpoints::{LARGE_PACKAGE_UPLOAD_PATH, SMALL_PACKAGE_UPLOAD_PATH},
    helpers::create_file_from_stream,
};
use anyhow::{anyhow, Error, Result};
use awc::Client as ActixWebClient;
use deputy_library::package::PackageStream;
use log::error;
use std::str::from_utf8;
use url::Url;

pub struct Client {
    client: ActixWebClient,
    api_base_url: Url,
}

impl Client {
    pub fn try_new(api_base_url: String) -> Result<Self> {
        Ok(Self {
            client: ActixWebClient::new(),
            api_base_url: Url::parse(&api_base_url)?,
        })
    }

    fn response_to_error(message: &str, payload: Vec<u8>) -> Result<Error> {
        let error_message = format!("{message}: {}", from_utf8(&payload)?);
        error!("{error_message}");
        Ok(anyhow!(error_message))
    }

    pub async fn stream_large_package(&self, stream: PackageStream) -> Result<()> {
        let put_uri = self.api_base_url.join(LARGE_PACKAGE_UPLOAD_PATH)?;
        let mut response = self
            .client
            .put(put_uri.to_string())
            .timeout(std::time::Duration::from_secs(60))
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
        let put_uri = self.api_base_url.join(SMALL_PACKAGE_UPLOAD_PATH)?;
        let mut response = self
            .client
            .put(put_uri.to_string())
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

    pub async fn download_package(&self, name: &str, version: &str, file_path: &str) -> Result<()> {
        let get_uri = self
            .api_base_url
            .join("api/v1/package/")?
            .join(&format!("{name}/"))?
            .join(&format!("{version}/"))?
            .join("download")?;
        let mut response = self
            .client
            .get(get_uri.to_string())
            .send()
            .await
            .map_err(|error| anyhow!("Failed to download package: {}", error))?;
        if response.status().is_success() {
            create_file_from_stream(&mut response, file_path).await?;
            return Ok(());
        }

        Err(Client::response_to_error(
            "Failed to download package",
            response.body().await?.to_vec(),
        )?)
    }
}
