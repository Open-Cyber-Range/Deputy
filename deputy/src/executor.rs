use crate::client::Client;
use crate::commands::{
    ChecksumOptions, FetchOptions, InspectOptions, LoginOptions, NormalizeVersionOptions,
    PublishOptions,
};
use crate::configuration::Configuration;
use crate::helpers::{
    create_temporary_package_download_path, find_toml, get_download_target_name,
    print_error_message, unpack_package_file,
};
use crate::progressbar::{AdvanceProgressBar, ProgressStatus, SpinnerProgressBar};
use actix::Actor;
use anyhow::{anyhow, Ok, Result};
use deputy_library::{package::Package, project::create_project_from_toml_path};
use dialoguer::Input;
use std::env::current_dir;
use std::path::{Path, PathBuf};
use tokio::fs::rename;

pub struct Executor {
    configuration: Configuration,
    token_file: PathBuf,
}

impl Executor {
    fn get_token_file(&self, registry_name: &str) -> Result<PathBuf> {
        let mut token_file = self.token_file.clone();
        token_file.set_file_name(format!(
            "{}-{}",
            token_file
                .file_name()
                .ok_or_else(|| {
                    anyhow!("Failed to get token file name from path: {:?}", token_file)
                })?
                .to_str()
                .ok_or_else(|| {
                    anyhow!(
                        "Failed to convert token file name to string: {:?}",
                        token_file
                    )
                })?,
            registry_name
        ));
        Ok(token_file)
    }

    fn get_token_value(&self, registry_name: &str) -> Option<String> {
        let token_file = self.get_token_file(registry_name).ok()?;
        std::fs::read_to_string(token_file).ok()
    }

    fn try_create_client(&self, registry_name: String) -> Result<Client> {
        let api_url = if let Some(registry) = self.configuration.registries.get(&registry_name) {
            registry.api.clone()
        } else {
            return Err(anyhow::anyhow!("Registry not found in configuration"));
        };
        let token = self.get_token_value(&registry_name);

        Client::try_new(api_url, token)
    }

    pub fn try_new() -> Result<Self> {
        let configuration = Configuration::get_configuration()?;
        let token_file = Configuration::get_token_file_path()?;
        Ok(Self {
            configuration,
            token_file,
        })
    }

    pub async fn publish(&self, options: PublishOptions) -> Result<()> {
        let progress_actor = SpinnerProgressBar::new("Package published".to_string()).start();
        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::InProgress(
                "Finding toml".to_string(),
            )))
            .await??;
        let toml_path = find_toml(current_dir()?)?;

        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::InProgress(
                "Creating package".to_string(),
            )))
            .await??;
        let package = Package::from_file(toml_path, options.compression).map_err(|e| {
            anyhow::anyhow!(
                "Failed to create package based on TOML file: {}",
                e.to_string()
            )
        })?;

        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::InProgress(
                "Creating client".to_string(),
            )))
            .await??;
        let client = self.try_create_client(options.registry_name)?;

        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::InProgress(
                "Validating version".to_string(),
            )))
            .await??;
        client
            .validate_version(
                package.metadata.clone().name,
                package.metadata.clone().version,
            )
            .await?;

        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::InProgress(
                "Uploading".to_string(),
            )))
            .await??;

        client
            .upload_package(package.to_stream().await?, options.timeout)
            .await?;
        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::Done))
            .await??;
        Ok(())
    }

    pub async fn fetch(&self, options: FetchOptions) -> Result<()> {
        let progress_actor = SpinnerProgressBar::new("Package fetched".to_string()).start();
        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::InProgress(
                "Updating the repositories".to_string(),
            )))
            .await??;
        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::InProgress(
                "Registering the repository".to_string(),
            )))
            .await??;
        let client = self.try_create_client(options.registry_name.clone())?;

        let version = client
            .get_latest_matching_package(&options.package_name, &options.version_requirement)
            .await?
            .version;

        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::InProgress(
                "Downloading the package".to_string(),
            )))
            .await??;
        let (temporary_package_path, temporary_parent_directory, temporary_package_directory) =
            create_temporary_package_download_path(&options.package_name, &version)?;
        client
            .download_package(&options.package_name, &version, &temporary_package_path)
            .await?;
        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::InProgress(
                "Decompressing the package".to_string(),
            )))
            .await??;
        let unpacked_file_path =
            unpack_package_file(&temporary_package_path, &options.unpack_level)?;
        let target_path = get_download_target_name(
            &options.unpack_level,
            &options.save_path,
            &options.package_name,
            &version,
        );

        rename(unpacked_file_path, target_path).await?;
        temporary_package_directory.close()?;
        temporary_parent_directory.close()?;
        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::Done))
            .await??;

        Ok(())
    }

    pub async fn checksum(&self, options: ChecksumOptions) -> Result<()> {
        let client = self.try_create_client(options.registry_name.clone())?;
        let version = client
            .get_latest_matching_package(&options.package_name, &options.version_requirement)
            .await?
            .version;
        let checksum = client
            .get_package_version(options.package_name.to_string(), version)
            .await?
            .checksum;
        println!("{checksum}");
        Ok(())
    }

    pub async fn inspect(&self, options: InspectOptions) -> Result<()> {
        let package_path: &Path = match options.package_path.trim() {
            "" => Path::new("."),
            path => Path::new(path),
        };
        let toml_path = find_toml(package_path.to_path_buf())?;
        let project = create_project_from_toml_path(&toml_path)?;
        let readme_path = package_path.join(&project.package.readme);
        if !readme_path.exists() {
            print_error_message(anyhow!("Readme not found"));
        } else if options.pretty {
            println!("{}", serde_json::to_string_pretty(&project)?);
        } else if let Some(vm) = &project.virtual_machine {
            let file_path = package_path.join(&vm.file_path);
            if !file_path.exists() {
                print_error_message(anyhow!("Virtual machine file not found"));
            } else {
                println!("{}", serde_json::to_string(&project)?);
            }
        } else if project.feature.is_some()
            || project.condition.is_some()
            || project.event.is_some()
            || project.inject.is_some()
        {
            if let Some(assets) = &project.package.assets {
                let mut asset_paths: Vec<PathBuf> = Vec::new();
                for asset in assets {
                    let asset_path = package_path.join(&asset[0]);
                    if !asset_path.exists() {
                        print_error_message(anyhow!("Asset '{}' not found", asset_path.display()));
                    } else {
                        asset_paths.push(asset_path);
                        println!("{}", serde_json::to_string(&project)?);
                    }
                }
            } else {
                print_error_message(anyhow!(
                    "Package.Assets field is required for this Package Type"
                ));
            }
        } else {
            println!("{}", serde_json::to_string(&project)?);
        }
        Ok(())
    }

    pub async fn normalize_version(&self, options: NormalizeVersionOptions) -> Result<()> {
        let client = self.try_create_client(options.registry_name.clone())?;
        let version = client
            .get_latest_matching_package(&options.package_name, &options.version_requirement)
            .await?
            .version;
        println!("{}", version);
        Ok(())
    }

    pub async fn login(&self, options: LoginOptions) -> Result<()> {
        let token_path = self.get_token_file(&options.registry_name)?;
        let token_value = Input::<String>::new()
            .with_prompt("Token")
            .interact_text()?;

        std::fs::write(token_path, token_value)?;
        Ok(())
    }
}
