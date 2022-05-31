use crate::client::Client;
use crate::commands::FetchOptions;
use crate::configuration::Configuration;
use crate::constants::{DEFAULT_REGISTRY_NAME, SMALL_PACKAGE_LIMIT};
use crate::helpers::{find_toml, print_success_message};
use anyhow::Result;
use deputy_library::package::Package;
use std::env::current_dir;

pub struct Executor {
    configuration: Configuration,
}

impl Executor {
    pub fn try_new(configuration: Configuration) -> Result<Self> {
        Ok(Self { configuration })
    }

    pub fn try_create_client(&self, registry_name_option: Option<String>) -> Result<Client> {
        let api_url = if let Some(overriding_registry_name) = registry_name_option {
            if let Some(registry) = self.configuration.registries.get(&overriding_registry_name) {
                registry.api.clone()
            } else {
                return Err(anyhow::anyhow!(
                    "Registry {} not found in configuration",
                    overriding_registry_name
                ));
            }
        } else if let Some(registry) = self.configuration.registries.get(DEFAULT_REGISTRY_NAME) {
            registry.api.clone()
        } else {
            return Err(anyhow::anyhow!(
                "Default registry not found in configuration"
            ));
        };

        Ok(Client::new(api_url))
    }

    pub async fn publish(&self) -> Result<()> {
        let package_toml = find_toml(current_dir()?)?;
        let package = Package::from_file(package_toml)?;
        let client = self.try_create_client(None)?;
        if package.get_size()? <= *SMALL_PACKAGE_LIMIT {
            client.upload_small_package(package.try_into()?).await?;
        } else {
            client.stream_large_package(package.try_into()?).await?;
        }
        print_success_message("Package published");
        Ok(())
    }

    pub async fn fetch(&self, options: FetchOptions) -> Result<()> {
        println!("Fetch options: {:?}", options);
        Ok(())
    }
}
