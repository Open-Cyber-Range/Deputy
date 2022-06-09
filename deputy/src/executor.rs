use crate::client::Client;
use crate::commands::FetchOptions;
use crate::configuration::{Configuration, Registry};
use crate::constants::{DEFAULT_REGISTRY_NAME, SMALL_PACKAGE_LIMIT};
use crate::helpers::{
    create_temporary_package_download_path, find_toml, get_download_target_name,
    print_success_message, unpack_package_file,
};
use anyhow::Result;
use deputy_library::repository::find_largest_matching_version;
use deputy_library::{
    package::Package,
    repository::{get_or_clone_repository, pull_from_remote},
};
use git2::Repository;
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
            return Err(anyhow::anyhow!(
                "Default registry not found in configuration"
            ));
        };

        Client::try_new(api_url)
    }

    fn update_registry_repositories(&self) -> Result<()> {
        for repository in self.repositories.values() {
            pull_from_remote(repository)?;
        }
        Ok(())
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

    pub async fn publish(&self) -> Result<()> {
        let package_toml = find_toml(current_dir()?)?;
        let package = Package::from_file(package_toml)?;
        let client = self.try_create_client(DEFAULT_REGISTRY_NAME.to_string())?;
        if package.get_size()? <= *SMALL_PACKAGE_LIMIT {
            client.upload_small_package(package.try_into()?).await?;
        } else {
            client.stream_large_package(package.try_into()?).await?;
        }
        print_success_message("Package published");
        Ok(())
    }

    pub async fn fetch(&self, options: FetchOptions) -> Result<()> {
        self.update_registry_repositories()?;
        let registry_repository = self
            .repositories
            .get(&options.registry_name)
            .ok_or_else(|| anyhow::anyhow!("Registry not found"))?;
        let version = find_largest_matching_version(
            registry_repository,
            &options.package_name,
            &options.version_requirement,
        )?
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
}
