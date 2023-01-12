use crate::{
    constants::endpoints::PACKAGE_UPLOAD_PATH,
    helpers::create_file_from_stream,
};
use anyhow::{anyhow, Error, Result};
use awc::Client as ActixWebClient;
use deputy_library::package::{PackageMetadata, PackageStream};
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

    pub async fn upload_package(&self, stream: PackageStream, timeout: u64) -> Result<()> {
        let put_uri = self.api_base_url.join(PACKAGE_UPLOAD_PATH)?;
        let mut response = self
            .client
            .put(put_uri.to_string())
            .timeout(std::time::Duration::from_secs(timeout))
            .send_stream(stream)
            .await
            .map_err(|error| anyhow!("Failed to upload package: {:?}", error))?;
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
            .map_err(|error| anyhow!("Failed to download package: {:?}", error))?;
        if response.status().is_success() {
            create_file_from_stream(&mut response, file_path).await?;
            return Ok(());
        }

        Err(Client::response_to_error(
            "Failed to download package",
            response.body().await?.to_vec(),
        )?)
    }

    pub async fn get_package_metadata(&self, name: String, version: String) -> Result<PackageMetadata> {
        let get_uri = self
            .api_base_url
            .join("api/v1/package/")?
            .join(&format!("{name}/"))?
            .join(&format!("{version}/"))?
            .join("metadata")?;
        let mut response = self
            .client
            .get(get_uri.to_string())
            .send()
            .await
            .map_err(|error| anyhow!("Failed to fetch package metadata: {:?}", error))?;
        if response.status().is_success() {
            let metadata = PackageMetadata::try_from(&*response.body().await?.to_vec())?;
            return Ok(metadata);
        }

        Err(Client::response_to_error(
            "Failed to fetch package metadata",
            response.body().await?.to_vec(),
        )?)
    }

    pub async fn try_get_latest_version(&self, name: String, version: String) -> Result<String> {
        let get_uri = self
            .api_base_url
            .join("api/v1/package/")?
            .join(&format!("{name}/"))?
            .join(&format!("{version}/"))?
            .join("try_get_latest")?;
        let mut response = self
            .client
            .get(get_uri.to_string())
            .send()
            .await
            .map_err(|error| anyhow!("Failed to check if latest version: {:?}", error))?;
        if response.status().is_success() {
            let version = String::from_utf8(Vec::from(response.body().await?))?;
            return Ok(version);
        }

        Err(Client::response_to_error(
            "Failed to check if latest version",
            response.body().await?.to_vec(),
        )?)
    }

    pub async fn validate_version(&self, name: String, version: String) -> Result<()> {
        let get_uri = self
            .api_base_url
            .join("api/v1/package/")?
            .join(&format!("{name}/"))?
            .join(&format!("{version}/"))?
            .join("exists")?;
        let mut response = self.client
            .get(get_uri.to_string())
            .send()
            .await
            .map_err(|error| anyhow!("Failed to validate package version: {:?}", error))?;
        if response.status().is_success() {
            return Ok(());
        };
        Err(Client::response_to_error(
            "Package version error",
            response.body().await?.to_vec(),
        )?)
    }
}
