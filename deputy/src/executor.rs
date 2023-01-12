use crate::client::Client;
use crate::commands::{
    ChecksumOptions, FetchOptions, NormalizeVersionOptions, ParseTOMLOptions, PublishOptions,
};
use crate::configuration::Configuration;
use crate::helpers::{
    create_temporary_package_download_path, find_toml, get_download_target_name,
    unpack_package_file,
};
use crate::progressbar::{AdvanceProgressBar, ProgressStatus, SpinnerProgressBar};
use actix::Actor;
use anyhow::Result;
use deputy_library::{
    package::Package,
    project::create_project_from_toml_path,
};
use path_absolutize::Absolutize;
use std::path::Path;
use std::env::current_dir;
use tokio::fs::rename;
use crate::constants::DEFAULT_REGISTRY_NAME;

pub struct Executor {
    configuration: Configuration,
}

impl Executor {
    fn try_create_client(&self, registry_name: String) -> Result<Client> {
        let api_url = if let Some(registry) = self.configuration.registries.get(&registry_name) {
            registry.api.clone()
        } else {
            return Err(anyhow::anyhow!("Registry not found in configuration"));
        };

        Client::try_new(api_url)
    }

    async fn get_version(
        &self,
        package_name: &str,
        version_requirement: &str,
    ) -> Result<String> {
        let client = self.try_create_client(DEFAULT_REGISTRY_NAME.to_string())?;
        let metadata = client.get_package_metadata(package_name.to_string(), version_requirement.to_string()).await;
        if metadata.is_ok() {
            Ok(metadata?.version)
        } else {
            Ok(client.try_get_latest_version(package_name.to_string(), version_requirement.to_string()).await?)
        }
    }

    pub fn new(configuration: Configuration) -> Self {
        Self {configuration}
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
        let project = create_project_from_toml_path(&toml_path)?;
        let package = Package::from_file(project.package.readme, toml_path, options.compression)?;

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
        client.validate_version(package.metadata.clone().name, package.metadata.clone().version).await?;
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
        let version = self.get_version(
            &options.package_name,
            &options.version_requirement,
        ).await?;

        let client = self.try_create_client(options.registry_name.clone())?;
        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::InProgress(
                "Downloading the package".to_string(),
            )))
            .await??;
        let (temporary_package_path, temporary_directory) =
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
        let target_path = get_download_target_name(&options.unpack_level, &options.save_path, &options.package_name, &version);

        rename(
            unpacked_file_path,
            target_path,
        )
        .await?;
        temporary_directory.close()?;
        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::Done))
            .await??;

        Ok(())
    }

    pub async fn checksum(&self, options: ChecksumOptions) -> Result<()> {
        let client = self.try_create_client(DEFAULT_REGISTRY_NAME.to_string())?;
        let version = self.get_version(
            &options.package_name,
            &options.version_requirement,
        ).await?;
        let checksum = client.get_package_metadata(
            options.package_name.to_string(),
            version,
        ).await?.checksum;
        println!("{checksum}");
        Ok(())
    }

    pub fn parse_toml(&self, options: ParseTOMLOptions) -> Result<()> {
        let package_toml_path = Path::new(&options.package_toml_path).absolutize()?;
        let project = create_project_from_toml_path(&package_toml_path)?;
        if options.pretty {
            println!("{}", serde_json::to_string_pretty(&project)?);
        } else {
            println!("{}", serde_json::to_string(&project)?);
        }
        Ok(())
    }

    pub async fn normalize_version(&self, options: NormalizeVersionOptions) -> Result<()> {
        let version = self.get_version(
            &options.package_name,
            &options.version_requirement,
        ).await?;
        println!("{}", version);
        Ok(())
    }
}
