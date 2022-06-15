use anyhow::Result;
use futures::StreamExt;
use shiplift::{BuildOptions, Container, ContainerOptions, Docker, RmContainerOptions};

const DOCKER_FILE_PATH: &str = "git-server-docker";
const DOCKER_IMAGE_NAME: &str = "git-server-docker-mock";

async fn create_image(docker: &Docker) -> Result<()> {
    let docker_path = std::env::current_dir()?
        .join(DOCKER_FILE_PATH)
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Could not convert path to string"))?
        .to_string();
    let options = BuildOptions::builder(docker_path)
        .tag(DOCKER_IMAGE_NAME)
        .build();
    let build_stream = docker.images().build(&options);

    build_stream
        .filter_map(|x| async move { x.ok().map(Ok) })
        .forward(futures::sink::drain())
        .await?;
    Ok(())
}

pub struct MockRepostioryServer {
    id: String,
    index_url: String,
    docker: Docker,
}

impl MockRepostioryServer {
    pub async fn try_new(server_port: u16, repository_path: &str) -> Result<Self> {
        let docker = Docker::new();
        create_image(&docker).await?;
        let repository_mapping = format!("{}/.git:/srv/git/index.git", repository_path);

        let container_options = ContainerOptions::builder(DOCKER_IMAGE_NAME)
            .expose(80, "tcp", server_port as u32)
            .volumes(vec![&repository_mapping])
            .build();
        let result = docker.containers().create(&container_options).await?;

        Ok(Self {
            docker,
            id: result.id,
            index_url: format!("http://localhost:{}/git/index.git", server_port),
        })
    }

    pub fn get_index_url(&self) -> &str {
        &self.index_url
    }

    pub async fn start(&self) -> Result<()> {
        let container = Container::new(&self.docker, self.id.clone());
        container.start().await?;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let container = Container::new(&self.docker, self.id.clone());
        container
            .remove(RmContainerOptions::builder().force(true).build())
            .await?;
        Ok(())
    }
}
