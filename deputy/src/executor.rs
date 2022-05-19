use crate::client::Client;
use crate::configuration::Configuration;
use crate::constants::SMALL_PACKAGE_LIMIT;
use crate::helpers::find_toml;
use anyhow::Result;
use deputy_library::package::Package;
use std::env::current_dir;

pub struct Executor {
    client: Client,
}

impl Executor {
    pub fn try_new(configuration: Configuration) -> Result<Self> {
        Ok(Self {
            client: Client::new(
                configuration
                    .repository
                    .repositories
                    .get(0)
                    .ok_or_else(|| anyhow::anyhow!("No repositories found in configuration"))?
                    .api
                    .clone(),
            ),
        })
    }

    pub async fn publish(&self) -> Result<()> {
        let package_toml = find_toml(current_dir()?)?;
        let package = Package::from_file(package_toml)?;
        if package.get_size()? <= *SMALL_PACKAGE_LIMIT {
            self.client
                .upload_small_package(package.try_into()?)
                .await?;
        } else {
            self.client
                .stream_large_package(package.try_into()?)
                .await?;
        }
        Ok(())
    }
}
