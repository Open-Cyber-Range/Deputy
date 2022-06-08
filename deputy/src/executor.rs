use crate::client::Client;
use crate::commands::FetchOptions;
use crate::configuration::Configuration;
use crate::constants::{DEFAULT_REGISTRY_NAME, SMALL_PACKAGE_LIMIT};
use crate::helpers::{AdvanceProgressBar, find_toml, SpinnerProgressBar, ProgressStatus};
use anyhow::Result;
use deputy_library::package::Package;
use std::env::current_dir;
use actix::Actor;

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
        let progress_actor = SpinnerProgressBar::new("Package published".to_string()).start();
        progress_actor.send(AdvanceProgressBar(ProgressStatus::InProgress("Finding toml".to_string()))).await??;
        let package_toml = find_toml(current_dir()?)?;
        progress_actor.send(AdvanceProgressBar(ProgressStatus::InProgress("Creating package".to_string()))).await??;
        let package = Package::from_file(package_toml)?;
        progress_actor.send(AdvanceProgressBar(ProgressStatus::InProgress("Creating client".to_string()))).await??;
        let client = self.try_create_client(None)?;
        progress_actor.send(AdvanceProgressBar(ProgressStatus::InProgress("Uploading".to_string()))).await??;
        if package.get_size()? <= *SMALL_PACKAGE_LIMIT {
            client.upload_small_package(package.try_into()?).await?;
        } else {
            client.stream_large_package(package.try_into()?).await?;
        }
        progress_actor.send(AdvanceProgressBar(ProgressStatus::Done)).await??;
        Ok(())
    }

    pub async fn fetch(&self, options: FetchOptions) -> Result<()> {
        println!("Fetch options: {:?}", options);
        Ok(())
    }
}
