use crate::{constants::endpoints::PACKAGE_UPLOAD_PATH, helpers::create_file_from_stream};
use anyhow::{anyhow, Error, Ok, Result};
use awc::{http::header, Client as ActixWebClient};
use deputy_library::{package::PackageStream, rest::VersionRest};
use log::error;
use qstring::QString;
use std::str::from_utf8;
use url::Url;

pub struct Client {
    client: ActixWebClient,
    api_base_url: Url,
    token: Option<String>,
}

impl Client {
    pub fn try_new(api_base_url: String, token: Option<String>) -> Result<Self> {
        let token = token.map(|token| format!("Bearer {}", token));
        Ok(Self {
            client: ActixWebClient::new(),
            api_base_url: Url::parse(&api_base_url)?,
            token,
        })
    }

    fn response_to_error(message: &str, payload: Vec<u8>) -> Result<Error> {
        let error_message = format!("{message}: {}", from_utf8(&payload)?);
        error!("{error_message}");
        Ok(anyhow!(error_message))
    }

    pub async fn upload_package(&self, stream: PackageStream, timeout: u64) -> Result<()> {
        let put_uri = self.api_base_url.join(PACKAGE_UPLOAD_PATH)?;
        let mut client_request = self.client.put(put_uri.to_string());
        let token = self
            .token
            .clone()
            .ok_or_else(|| anyhow!("No login token found"))?;

        let headers = client_request.headers_mut();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&token)?,
        );

        let mut response = client_request
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

    pub async fn get_package_version(&self, name: String, version: String) -> Result<VersionRest> {
        let get_uri = self
            .api_base_url
            .join("api/v1/package/")?
            .join(&format!("{name}/"))?
            .join(&version)?;
        let mut response = self
            .client
            .get(get_uri.to_string())
            .send()
            .await
            .map_err(|error| anyhow!("Failed to fetch package metadata: {:?}", error))?;
        if response.status().is_success() {
            let body = response.body().await?;
            let version_rest: VersionRest = serde_json::from_slice(&body)?;
            return Ok(version_rest);
        }

        Err(Client::response_to_error(
            "Failed to fetch package metadata",
            response.body().await?.to_vec(),
        )?)
    }

    pub async fn get_latest_matching_package(
        &self,
        name: &str,
        version_requirement: &str,
    ) -> Result<VersionRest> {
        let mut get_uri = self.api_base_url.join("api/v1/package/")?.join(name)?;
        get_uri.set_query(Some(
            QString::new(vec![("version_requirement", version_requirement)])
                .to_string()
                .as_str(),
        ));

        let mut response = self
            .client
            .get(get_uri.to_string())
            .send()
            .await
            .map_err(|error| anyhow!("Failed to fetch packages: {:?}", error))?;
        if response.status().is_success() {
            let body = response.body().await?;
            let packages: Vec<VersionRest> = serde_json::from_slice(&body)?;
            if let Some(matching_package) = VersionRest::get_latest_package(packages) {
                return Ok(matching_package);
            }
            return Err(anyhow!(
                "No packages with {name} found matching version requirement {version_requirement}",
            ));
        }

        Err(Client::response_to_error(
            "Failed to fetch packages",
            response.body().await?.to_vec(),
        )?)
    }

    pub async fn validate_version(&self, name: String, version: String) -> Result<()> {
        let get_uri = self
            .api_base_url
            .join("api/v1/package/")?
            .join(format!("{name}/").as_str())?;

        let mut response = self
            .client
            .get(get_uri.to_string())
            .timeout(std::time::Duration::from_secs(100))
            .send()
            .await
            .map_err(|error| anyhow!("Failed to validate package version: {:?}", error))?;

        match response.status() {
            awc::http::StatusCode::NOT_FOUND => Ok(()),
            awc::http::StatusCode::OK => {
                let body = response.body().await?;
                let packages: Vec<VersionRest> = serde_json::from_slice(&body)?;
                if let Some(existing) = VersionRest::is_latest_version(&version, packages)? {
                    return Err(anyhow!(
                        "Package version {version} already exists. Latest version is {existing}",
                    ));
                }
                Ok(())
            }
            _ => Err(Client::response_to_error(
                "Package version error",
                response.body().await?.to_vec(),
            )?),
        }
    }
}
