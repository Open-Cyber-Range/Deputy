use crate::repository::TestRepositoryServer;
use anyhow::Result;
use deputy::constants::DEFAULT_REGISTRY_NAME;
use deputy_package_server::{configuration::Configuration, test::TestPackageServer};
use std::io::Write;
use tempfile::{tempdir, Builder, NamedTempFile, TempDir};

pub struct TestBackEnd {
    pub test_repository_server: TestRepositoryServer,
    pub configuration: Configuration,
    pub configuration_directory: TempDir,
    pub configuration_file: NamedTempFile,
    pub server_address: String,
}

#[allow(dead_code)]
impl TestBackEnd {
    pub async fn setup_test_backend() -> Result<TestBackEnd> {
        let (configuration, server_address) = TestPackageServer::setup_test_server().await?;
        let (test_repository_server, index_url) =
            TestRepositoryServer::try_new(&configuration.repository.folder).await?;
        let (configuration_directory, configuration_file) =
            Self::create_temp_configuration_file(&server_address, &index_url)?;

        let test_backend = Self::new(
            test_repository_server,
            configuration,
            configuration_directory,
            configuration_file,
            server_address,
        );
        test_backend.test_repository_server.start().await?;
        Ok(test_backend)
    }

    pub fn new(
        test_repository_server: TestRepositoryServer,
        configuration: Configuration,
        configuration_directory: TempDir,
        configuration_file: NamedTempFile,
        server_address: String,
    ) -> Self {
        Self {
            test_repository_server,
            configuration,
            configuration_directory,
            configuration_file,
            server_address,
        }
    }

    pub fn create_temp_configuration_file(
        api_address: &str,
        index_repository: &str,
    ) -> Result<(TempDir, NamedTempFile)> {
        let configuration_file_contents = format!(
            "[registries]\n{DEFAULT_REGISTRY_NAME} = {{ index = \"{index_repository}\", api = \"{api_address}\" }}\n[package]\nindex_path = \"./index\"\ndownload_path = \"./download\"",
        );

        let configuration_directory = tempdir()?;
        let mut configuration_file = Builder::new()
            .prefix("configuration")
            .suffix(".toml")
            .rand_bytes(0)
            .tempfile_in(&configuration_directory)?;
        configuration_file.write_all(configuration_file_contents.as_bytes())?;
        Ok((configuration_directory, configuration_file))
    }
}
