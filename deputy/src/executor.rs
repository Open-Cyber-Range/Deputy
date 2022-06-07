use crate::client::Client;
use crate::commands::FetchOptions;
use crate::configuration::Configuration;
use crate::constants::{DEFAULT_REGISTRY_NAME, SMALL_PACKAGE_LIMIT};
use crate::helpers::{find_toml, print_success_message};
use anyhow::Result;
use deputy_library::package::Package;
use std::env::current_dir;
use std::{thread};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle};

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
        let (sender, receiver): (Sender<String>, Receiver<String>) = mpsc::channel();

        let bar = ProgressBar::new(1);
        bar.set_style(ProgressStyle::default_spinner()
                .template("[{elapsed_precise}] {spinner} {msg}"));
        thread::spawn(move || {
            loop {
                if let Ok(received) = receiver.try_recv() {
                    let received_clone = received.clone();
                    bar.set_message(received_clone);
                    if received == "Done" {
                        bar.finish();
                        break;
                    }
                }
                bar.inc(1);
                // Sleep so that loading bar is more smooth
                thread::sleep(Duration::from_millis(75));
            }
        });

        sender.send(String::from("Finding toml")).unwrap();
        let package_toml = find_toml(current_dir()?)?;
        sender.send(String::from("Creating package")).unwrap();
        let package = Package::from_file(package_toml)?;
        sender.send(String::from("Creating client")).unwrap();
        let client = self.try_create_client(None)?;
        sender.send(String::from("Uploading")).unwrap();

        if package.get_size()? <= *SMALL_PACKAGE_LIMIT {
            client.upload_small_package(package.try_into()?).await?;
        } else {
            client.stream_large_package(package.try_into()?).await?;
        }
        sender.send(String::from("Done")).unwrap();
        print_success_message("Package published");
        Ok(())
    }

    pub async fn fetch(&self, options: FetchOptions) -> Result<()> {
        println!("Fetch options: {:?}", options);
        Ok(())
    }
}
