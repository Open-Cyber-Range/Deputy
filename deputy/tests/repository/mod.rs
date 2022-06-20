use anyhow::Result;
use bollard::{
    container::{Config, CreateContainerOptions, RemoveContainerOptions},
    image::BuildImageOptions,
    models::{HostConfig, PortBinding},
    Docker,
};
use deputy_library::test::generate_random_string;
use futures::StreamExt;
use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

const DOCKER_IMAGE_NAME: &str = "git-server-docker-mock";

async fn create_image(docker: &Docker) -> Result<()> {
    let build_options = BuildImageOptions {
        t: DOCKER_IMAGE_NAME.to_string(),
        ..Default::default()
    };
    let file_path = vec!["assets", "git-server-docker.tar.xz"]
        .iter()
        .collect::<PathBuf>();
    let mut file = File::open(file_path).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    let build_stream = docker.build_image(build_options, None, Some(contents.into()));

    build_stream
        .filter_map(|x| async move {
            {
                x.ok().map(Ok)
            }
        })
        .forward(futures::sink::drain())
        .await?;
    Ok(())
}

pub struct MockRepostioryServer {
    name: String,
    index_url: String,
    docker: Docker,
}

impl MockRepostioryServer {
    pub async fn try_new(server_port: u16, repository_path: &str) -> Result<Self> {
        let docker = Docker::connect_with_unix_defaults()?;
        create_image(&docker).await?;
        let repository_mapping = format!("{}/.git:/srv/git/index.git", repository_path);
        let mut ports: HashMap<String, Option<Vec<PortBinding>>> = HashMap::new();
        ports.insert(
            "80/tcp".to_string(),
            Some(vec![PortBinding {
                host_port: Some(format!("{}", server_port)),
                host_ip: Some("0.0.0.0".to_string()),
            }]),
        );
        let container_configuration = Config {
            image: Some(DOCKER_IMAGE_NAME.to_string()),
            host_config: Some(HostConfig {
                binds: Some(vec![repository_mapping]),
                port_bindings: Some(ports),
                ..Default::default()
            }),
            ..Default::default()
        };
        let name = generate_random_string(24)?;
        docker
            .create_container::<String, String>(
                Some(CreateContainerOptions { name: name.clone() }),
                container_configuration,
            )
            .await?;

        Ok(Self {
            docker,
            name,
            index_url: format!("http://localhost:{}/git/index.git", server_port),
        })
    }

    pub fn get_index_url(&self) -> &str {
        &self.index_url
    }

    pub async fn start(&self) -> Result<()> {
        self.docker
            .start_container::<String>(&self.name, None)
            .await?;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        self.docker
            .remove_container(
                &self.name,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await?;

        Ok(())
    }
}
