use crate::repository::TestRepositoryServer;
use anyhow::Result;
use deputy_package_server::{configuration::Configuration, test::TestPackageServer};

pub struct TestBackEnd {
    pub test_repository_server: TestRepositoryServer,
}

impl TestBackEnd {
    pub async fn setup_test_backend() -> Result<(Configuration, String, String, TestBackEnd)> {
        let (configuration, server_address) = TestPackageServer::setup_test_server().await?;
        let (test_repository_server, index_url) =
            TestRepositoryServer::try_new(&configuration.repository.folder).await?;
        let test_backend = Self::new(test_repository_server);
        test_backend.test_repository_server.start().await?;
        Ok((
            configuration.clone(),
            server_address,
            index_url,
            test_backend,
        ))
    }

    pub fn new(test_repository_server: TestRepositoryServer) -> Self {
        Self {
            test_repository_server,
        }
    }
}
