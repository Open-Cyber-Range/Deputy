mod repository;
mod test_backend;

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        thread::{self, JoinHandle},
    };

    use crate::test_backend::TestBackEnd;
    use anyhow::Result;
    use assert_cmd::Command;
    use deputy::client::Client;
    use deputy_library::{constants::CONFIGURATION_FOLDER_PATH_ENV_KEY, test::create_test_package};
    use futures::future::join_all;
    use tempfile::TempDir;

    async fn spawn_checksum_request(
        config_path: PathBuf,
        temp_dir: PathBuf,
    ) -> JoinHandle<Result<(), anyhow::Error>> {
        thread::spawn(move || {
            let mut command = Command::cargo_bin("deputy")?;
            command.arg("checksum").arg("some-package-name");
            command.current_dir(temp_dir);
            command.env(CONFIGURATION_FOLDER_PATH_ENV_KEY, config_path);
            command.assert().success();
            command
                .assert()
                .stdout("aa30b1cc05c10ac8a1f309e3de09de484c6de1dc7c226e2cf8e1a518369b1d73\n");
            Ok::<_, anyhow::Error>(())
        })
    }

    #[actix_web::test]
    async fn create_concurrent_checksum_requests() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let package = create_test_package()?;
        let pacakge_bytes: Vec<u8> = package.try_into()?;
        let test_backend = TestBackEnd::builder().build().await?;

        let client = Client::try_new(test_backend.server_address.to_string())?;
        let response = client.upload_small_package(pacakge_bytes.clone(), 60).await;
        assert!(response.is_ok());

        let config_path = test_backend.configuration_directory.path().to_owned();
        let temp_dir = temp_dir.into_path().clone();

        const CONCURRENT_REQUESTS: usize = 10;
        let requests = vec![0; CONCURRENT_REQUESTS];

        let join_handles = join_all(requests.iter().map(|_| async {
            spawn_checksum_request(config_path.clone(), temp_dir.clone()).await
        }))
        .await;

        for handle in join_handles {
            handle.join().unwrap()?;
        }
        test_backend.configuration_file.close()?;
        test_backend.configuration_directory.close()?;
        test_backend.test_repository_server.stop().await?;
        Ok(())
    }

    #[actix_web::test]
    async fn get_package_checksum() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_backend = TestBackEnd::builder().build().await?;

        let package = create_test_package()?;
        let client = Client::try_new(test_backend.server_address.to_string())?;
        let response = client.upload_small_package(package.try_into()?, 60).await;
        assert!(response.is_ok());

        let mut command = Command::cargo_bin("deputy")?;
        command.arg("checksum").arg("some-package-name");
        command.current_dir(&temp_dir);
        command.env(
            CONFIGURATION_FOLDER_PATH_ENV_KEY,
            &test_backend.configuration_directory.path(),
        );
        command.assert().success();
        command
            .assert()
            .stdout("aa30b1cc05c10ac8a1f309e3de09de484c6de1dc7c226e2cf8e1a518369b1d73\n");

        test_backend.configuration_file.close()?;
        test_backend.configuration_directory.close()?;
        test_backend.test_repository_server.stop().await?;
        Ok(())
    }
}
