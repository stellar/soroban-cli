use std::collections::HashMap;

use bollard::{
    container::{Config, CreateContainerOptions, StartContainerOptions},
    image::CreateImageOptions,
    service::{HostConfig, PortBinding},
};
use futures_util::TryStreamExt;

use crate::commands::network::container::shared::{
    connect_to_docker, Error as ConnectionError, Network,
};

use super::shared::ContainerArgs;

const DEFAULT_PORT_MAPPING: &str = "8000:8000";
const DOCKER_IMAGE: &str = "docker.io/stellar/quickstart";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("⛔ ️Failed to connect to docker: {0}")]
    ConnectionError(#[from] ConnectionError),

    #[error("⛔ ️Failed to create container: {0}")]
    BollardErr(#[from] bollard::errors::Error),
}

#[derive(Debug, clap::Parser, Clone)]
pub struct Cmd {
    #[command(flatten)]
    pub container_args: ContainerArgs,

    /// Network to start
    pub network: Network,

    /// Optional argument to specify the limits for the local network only
    #[arg(short = 'l', long)]
    pub limits: Option<String>,

    /// Argument to specify the HOST_PORT:CONTAINER_PORT mapping
    #[arg(short = 'p', long, num_args = 1.., default_value = DEFAULT_PORT_MAPPING)]
    pub ports_mapping: Vec<String>,

    /// Optional argument to override the default docker image tag for the given network
    #[arg(short = 't', long)]
    pub image_tag_override: Option<String>,

    /// Optional argument to specify the protocol version for the local network only
    #[arg(short = 'v', long)]
    pub protocol_version: Option<String>,
}

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        println!("ℹ️  Starting {} network", &self.network);
        run_docker_command(self).await
    }
}

async fn run_docker_command(cmd: &Cmd) -> Result<(), Error> {
    let docker = connect_to_docker(&cmd.container_args.docker_host).await?;

    let image = get_image_name(cmd);
    docker
        .create_image(
            Some(CreateImageOptions {
                from_image: image.clone(),
                ..Default::default()
            }),
            None,
            None,
        )
        .try_collect::<Vec<_>>()
        .await?;

    let container_args = get_container_args(cmd);
    let port_mapping = get_port_mapping(cmd);

    let config = Config {
        image: Some(image),
        cmd: Some(container_args),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        host_config: Some(HostConfig {
            auto_remove: Some(true),
            port_bindings: Some(port_mapping),
            ..Default::default()
        }),
        ..Default::default()
    };

    let container_name = format!("stellar-{}", cmd.network);
    let create_container_response = docker
        .create_container(
            Some(CreateContainerOptions {
                name: container_name.clone(),
                ..Default::default()
            }),
            config,
        )
        .await?;

    docker
        .start_container(
            &create_container_response.id,
            None::<StartContainerOptions<String>>,
        )
        .await?;
    println!("✅ Container started: {container_name}");
    let stop_message = format!(
        "ℹ️  To stop this container run: stellar network container stop {network} {additional_flags}",
        network = &cmd.network,
        additional_flags = if cmd.container_args.docker_host.is_some() {
            format!("--docker-host {}", cmd.container_args.docker_host.as_ref().unwrap())
        } else {
            String::new()
        }
    );

    println!("{stop_message}");
    Ok(())
}

fn get_container_args(cmd: &Cmd) -> Vec<String> {
    [
        format!("--{}", cmd.network),
        "--enable rpc,horizon".to_string(),
        get_protocol_version_arg(cmd),
        get_limits_arg(cmd),
    ]
    .iter()
    .filter(|&s| !s.is_empty())
    .cloned()
    .collect()
}

fn get_image_name(cmd: &Cmd) -> String {
    // this can be overriden with the `-t` flag
    let mut image_tag = match cmd.network {
        Network::Pubnet => "latest",
        Network::Futurenet => "future",
        _ => "testing", // default to testing for local and testnet
    };

    if let Some(image_override) = &cmd.image_tag_override {
        println!("Overriding docker image tag to use '{image_override}' instead of '{image_tag}'");
        image_tag = image_override;
    }

    format!("{DOCKER_IMAGE}:{image_tag}")
}

// The port mapping in the bollard crate is formatted differently than the docker CLI. In the docker CLI, we usually specify exposed ports as `-p  HOST_PORT:CONTAINER_PORT`. But with the bollard crate, it is expecting the port mapping to be a map of the container port (with the protocol) to the host port.
fn get_port_mapping(cmd: &Cmd) -> HashMap<String, Option<Vec<PortBinding>>> {
    let mut port_mapping_hash = HashMap::new();
    for port_mapping in &cmd.ports_mapping {
        let ports_vec: Vec<&str> = port_mapping.split(':').collect();
        let from_port = ports_vec[0];
        let to_port = ports_vec[1];

        port_mapping_hash.insert(
            format!("{to_port}/tcp"),
            Some(vec![PortBinding {
                host_ip: None,
                host_port: Some(from_port.to_string()),
            }]),
        );
    }

    port_mapping_hash
}

fn get_protocol_version_arg(cmd: &Cmd) -> String {
    if cmd.network == Network::Local && cmd.protocol_version.is_some() {
        let version = cmd.protocol_version.as_ref().unwrap();
        format!("--protocol-version {version}")
    } else {
        String::new()
    }
}

fn get_limits_arg(cmd: &Cmd) -> String {
    if cmd.network == Network::Local && cmd.limits.is_some() {
        let limits = cmd.limits.as_ref().unwrap();
        format!("--limits {limits}")
    } else {
        String::new()
    }
}
