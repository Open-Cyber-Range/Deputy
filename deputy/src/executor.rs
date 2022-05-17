use crate::client::{stream_large_package, upload_small_package};
use crate::configuration::Configuration;
use crate::constants::SMALL_PACKAGE_LIMIT;
use crate::helpers::find_toml;
use anyhow::Result;
use awc::Client;
use deputy_library::package::Package;
use std::env::current_dir;

#[derive(Default)]
pub struct Executor {
    configuration: Configuration,
    client: Client,
}

impl Executor {
    pub fn new(configuration: Configuration, client: Client) -> Self {
        Self {
            configuration,
            client,
        }
    }

    fn get_base_url(&self) -> Result<&str> {
        Ok(&self
            .configuration
            .repository
            .repositories
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("No repositories found in configuration"))?
            .api)
    }

    pub async fn publish(&self) -> Result<()> {
        let package_toml = find_toml(current_dir()?)?;
        let package = Package::from_file(package_toml)?;
        if package.get_size()? >= *SMALL_PACKAGE_LIMIT {
            upload_small_package(self.get_base_url()?, &self.client, package.try_into()?).await?;
        } else {
            stream_large_package(self.get_base_url()?, &self.client, package.try_into()?).await?;
        }
        Ok(())
    }
}
