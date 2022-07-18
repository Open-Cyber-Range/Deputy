use crate::client::Client;
use crate::commands::{ChecksumOptions, FetchOptions, InfoOptions, PublishOptions};
use crate::configuration::{Configuration, Registry};
use crate::constants::SMALL_PACKAGE_LIMIT;
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
    repository::{find_matching_metadata, get_or_clone_repository, pull_from_remote},
};
use git2::Repository;
use path_absolutize::Absolutize;
use std::path::Path;
use std::{collections::HashMap, env::current_dir, path::PathBuf};
use tokio::fs::rename;

pub struct Executor {
    configuration: Configuration,
    repositories: HashMap<String, Repository>,
}

impl Executor {
    fn get_or_create_registry_repositories(
        registries: HashMap<String, Registry>,
        index_repositories_root_path: String,
    ) -> Result<HashMap<String, Repository>> {
        let mut repositories = HashMap::new();
        for (registry_name, registry) in &registries {
            let repository_path = PathBuf::from(&index_repositories_root_path).join(registry_name);
            repositories.insert(
                registry_name.to_string(),
                get_or_clone_repository(&registry.index, repository_path)?,
            );
        }
        Ok(repositories)
    }

    fn try_create_client(&self, registry_name: String) -> Result<Client> {
        let api_url = if let Some(registry) = self.configuration.registries.get(&registry_name) {
            registry.api.clone()
        } else {
            return Err(anyhow::anyhow!("Registry not found in configuration"));
        };

        Client::try_new(api_url)
    }

    fn update_registry_repositories(&self) -> Result<()> {
        for repository in self.repositories.values() {
            pull_from_remote(repository)?;
        }
        Ok(())
    }

    fn get_registry(&self, registry_name: &str) -> Result<&Repository> {
        self.repositories
            .get(registry_name)
            .ok_or_else(|| anyhow::anyhow!("Registry not found"))
    }

    pub fn try_new(configuration: Configuration) -> Result<Self> {
        let repositories = Executor::get_or_create_registry_repositories(
            configuration.registries.clone(),
            configuration.package.index_path.clone(),
        )?;
        Ok(Self {
            configuration,
            repositories,
        })
    }

    pub async fn publish(&self, options: PublishOptions) -> Result<()> {
        let progress_actor = SpinnerProgressBar::new("Package published".to_string()).start();
        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::InProgress(
                "Finding toml".to_string(),
            )))
            .await??;
        let package_toml = find_toml(current_dir()?)?;
        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::InProgress(
                "Creating package".to_string(),
            )))
            .await??;
        let package = Package::from_file(package_toml, options.compression)?;
        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::InProgress(
                "Creating client".to_string(),
            )))
            .await??;
        let client = self.try_create_client(options.registry_name)?;
        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::InProgress(
                "Uploading".to_string(),
            )))
            .await??;
        if package.get_size()? <= *SMALL_PACKAGE_LIMIT {
            client
                .upload_small_package(package.try_into()?, options.timeout)
                .await?;
        } else {
            client
                .stream_large_package(package.try_into()?, options.timeout)
                .await?;
        }
        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::Done))
            .await??;
        Ok(())
    }

    pub async fn fetch(&self, options: FetchOptions) -> Result<()> {
        self.update_registry_repositories()?;
        let registry_repository = self.get_registry(&options.registry_name)?;
        let version = find_matching_metadata(
            registry_repository,
            &options.package_name,
            &options.version_requirement,
        )?
        .map(|metadata| metadata.version)
        .ok_or_else(|| anyhow::anyhow!("No version matching requirements found"))?;

        let client = self.try_create_client(options.registry_name.clone())?;
        let (temporary_package_path, temporary_directory) =
            create_temporary_package_download_path(&options.package_name, &version)?;
        client
            .download_package(&options.package_name, &version, &temporary_package_path)
            .await?;
        let unpacked_file_path =
            unpack_package_file(&temporary_package_path, &options.unpack_level)?;

        rename(
            unpacked_file_path,
            get_download_target_name(&options.unpack_level, &options.package_name, &version),
        )
        .await?;
        temporary_directory.close()?;

        Ok(())
    }

    pub fn checksum(&self, options: ChecksumOptions) -> Result<()> {
        self.update_registry_repositories()?;
        let registry = self.get_registry(&options.registry_name)?;
        let checksum = find_matching_metadata(
            registry,
            &options.package_name,
            &options.version_requirement,
        )?
        .map(|metadata| metadata.checksum)
        .ok_or_else(|| anyhow::anyhow!("No checksum matching requirements found"))?;
        println!("{checksum}");
        Ok(())
    }

    pub fn info(&self, options: InfoOptions) -> Result<()> {
        let package_toml_path = Path::new(&options.package_toml_path).absolutize()?;
        let project = create_project_from_toml_path(package_toml_path.to_path_buf())?;
        if options.pretty {
            println!("{}", serde_json::to_string_pretty(&project)?);
        } else {
            println!("{}", serde_json::to_string(&project)?);
        }
        Ok(())
    }
}
