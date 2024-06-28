use crate::commands::network::container::shared::{
    connect_to_docker, Error as ConnectionError, Network, DOCKER_HOST_HELP,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to stop container: {0}")]
    StopConnectionContainerError(#[from] ConnectionError),

    #[error("Container {container_name} not found")]
    ContainerNotFoundError {
        container_name: String,
        #[source]
        source: bollard::errors::Error,
    },

    #[error("Failed to stop container: {0}")]
    StopContainerError(#[from] bollard::errors::Error),
}

#[derive(Debug, clap::Parser, Clone)]
pub struct Cmd {
    /// Network to stop
    pub network: Network,

    #[arg(short = 'd', long, help = DOCKER_HOST_HELP, env = "DOCKER_HOST")]
    pub docker_host: Option<String>,
}

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        let container_name = format!("stellar-{}", self.network);
        let docker = connect_to_docker(&self.docker_host).await?;
        println!("ℹ️  Stopping container: {container_name}");
        docker
            .stop_container(&container_name, None)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("No such container") {
                    Error::ContainerNotFoundError {
                        container_name: container_name.clone(),
                        source: e,
                    }
                } else {
                    Error::StopContainerError(e)
                }
            })?;
        println!("✅ Container stopped: {container_name}");
        Ok(())
    }
}
