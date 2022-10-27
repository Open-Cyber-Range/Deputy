use crate::client::Client;
use crate::commands::{
    ChecksumOptions, FetchOptions, NormalizeVersionOptions, ParseTOMLOptions, PublishOptions,
};
use crate::configuration::{Configuration, Registry};
use crate::helpers::{
    create_temporary_package_download_path, find_toml, get_download_target_name,
    unpack_package_file,
};
use crate::progressbar::{AdvanceProgressBar, ProgressStatus, SpinnerProgressBar};
use actix::Actor;
use anyhow::Result;
use deputy_library::{
    package::{Package, IndexMetadata},
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

    fn get_version(
        &self,
        registry_name: &str,
        package_name: &str,
        version_requirement: &str,
    ) -> Result<String> {
        self.update_registry_repositories()?;
        let registry_repository = self.get_registry(registry_name)?;
        let version =
            find_matching_metadata(registry_repository, package_name, version_requirement)?
                .map(|metadata| metadata.version)
                .ok_or_else(|| anyhow::anyhow!("No version matching requirements found"))?;
        Ok(version)
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
        let toml_path = find_toml(current_dir()?)?;
        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::InProgress(
                "Validating version".to_string(),
            )))
            .await??;
        self.update_registry_repositories()?;
        let registry_repository = self.get_registry(&options.registry_name)?;
        IndexMetadata::validate_version(&toml_path, registry_repository)?;

        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::InProgress(
                "Creating package".to_string(),
            )))
            .await??;
        let project = create_project_from_toml_path(&toml_path)?;
        let optional_readme_path: Option<PathBuf> = match project.virtual_machine {
            Some(vm) => vm.readme_path.map(PathBuf::from),
            None => None,
        };

        let package = Package::from_file(optional_readme_path, toml_path, options.compression)?;
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
        self.update_registry_repositories()?;
        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::InProgress(
                "Registering the repository".to_string(),
            )))
            .await??;
        let registry_repository = self.get_registry(&options.registry_name)?;
        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::InProgress(
                "Fetching the version".to_string(),
            )))
            .await??;
        let _version = find_matching_metadata(
            registry_repository,
            &options.package_name,
            &options.version_requirement,
        )?
        .map(|metadata| metadata.version)
        .ok_or_else(|| anyhow::anyhow!("No version matching requirements found"))?;
        progress_actor
            .send(AdvanceProgressBar(ProgressStatus::InProgress(
                "Creating client".to_string(),
            )))
            .await??;
        let version = self.get_version(
            &options.registry_name,
            &options.package_name,
            &options.version_requirement,
        )?;

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

    pub fn normalize_version(&self, options: NormalizeVersionOptions) -> Result<()> {
        let version = self.get_version(
            &options.registry_name,
            &options.package_name,
            &options.version_requirement,
        )?;
        println!("{}", version);
        Ok(())
    }
}
