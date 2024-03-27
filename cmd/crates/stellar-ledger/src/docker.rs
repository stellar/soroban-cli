use std::{collections::HashMap, path::PathBuf};

use bollard::{
    container::{Config, CreateContainerOptions, LogsOptions, StartContainerOptions},
    image::CreateImageOptions,
    service::{HostConfig, PortBinding},
    ClientVersion, Docker,
};
use futures_util::TryStreamExt;

#[allow(unused_imports)]
// Need to add this for windows, since we are only using this crate for the unix fn try_docker_desktop_socket
use home::home_dir;

pub const DOCKER_HOST_HELP: &str = "Optional argument to override the default docker host. This is useful when you are using a non-standard docker host path for your Docker-compatible container runtime, e.g. Docker Desktop defaults to $HOME/.docker/run/docker.sock instead of /var/run/docker.sock";

// DEFAULT_DOCKER_HOST is from the bollard crate on the main branch, which has not been released yet: https://github.com/fussybeaver/bollard/blob/0972b1aac0ad5c08798e100319ddd0d2ee010365/src/docker.rs#L64
#[cfg(unix)]
pub const DEFAULT_DOCKER_HOST: &str = "unix:///var/run/docker.sock";

#[cfg(windows)]
pub const DEFAULT_DOCKER_HOST: &str = "npipe:////./pipe/docker_engine";

// DEFAULT_TIMEOUT and API_DEFAULT_VERSION are from the bollard crate
const DEFAULT_TIMEOUT: u64 = 120;
const API_DEFAULT_VERSION: &ClientVersion = &ClientVersion {
    major_version: 1,
    minor_version: 40,
};

const BOLOS_SDK: &str = "/project/deps/nanos-secure-sdk";
const DEFAULT_APP_PATH: &str = "/project/app/bin";
const BOLOS_ENV: &str = "/opt/bolos";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("⛔ ️Failed to start container: {0}")]
    BollardErr(#[from] bollard::errors::Error),

    #[error("URI scheme is not supported: {uri}")]
    UnsupportedURISchemeError { uri: String },
}

pub struct DockerConnection {
    docker: Docker,
}

impl DockerConnection {
    pub async fn new() -> Self {
        DockerConnection {
            docker: connect_to_docker(&Some(DEFAULT_DOCKER_HOST.to_owned()))
                .await
                .unwrap(),
        }
    }

    pub async fn get_image_with_defaults(&self, image_name: &str) -> Result<(), Error> {
        self.docker
            .create_image(
                Some(CreateImageOptions {
                    from_image: image_name.to_string(),
                    ..Default::default()
                }),
                None,
                None,
            )
            .try_collect::<Vec<_>>()
            .await?;

        Ok(())
    }

    //     docker run --rm -it -v "$(pwd)"/apps:/speculos/apps \
    // -p 1234:1234 -p 5001:5000 -p 40000:40000 -p 41000:41000 ghcr.io/ledgerhq/speculos:latest \
    // --model nanos ./apps/btc.elf --sdk 2.0 --seed "secret" --display headless --apdu-port 40000 \
    //  --vnc-port 41000

    // docker run --rm -it -v $(pwd)/apps:/speculos/apps \
    // -p 5001:5000 --publish 41000:41000 speculos  \
    // --model nanos ./apps/btc.elf --display headless --vnc-port 41000

    // docker run --rm -it -p 5001:5000 --publish 41000:41000 zondax/builder-zemu  \
    // --model nanos ./apps/btc.elf --display headless --vnc-port 41000

    // docker run --rm -it -v $(pwd)/apps:/speculos/apps -p 5001:5000 --publish 41000:41000 speculos --display headless --vnc-port 41000 --model nanos ./apps/btc.elf

    pub async fn get_container_with_defaults(&self, image_name: &str) -> Result<String, Error> {
        let default_port_mappings = vec!["5001:5000", "9998:9998", "41000:41000"];
        // The port mapping in the bollard crate is formatted differently than the docker CLI. In the docker CLI, we usually specify exposed ports as `-p  HOST_PORT:CONTAINER_PORT`. But with the bollard crate, it is expecting the port mapping to be a map of the container port (with the protocol) to the host port.
        let mut port_mapping_hash = HashMap::new();
        for port_mapping in default_port_mappings {
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

        // const displaySetting = "--display headless";
        // const command = `/home/zondax/speculos/speculos.py --log-level speculos:DEBUG --color JADE_GREEN ${displaySetting} ${customOptions} -m ${modelOptions} ${DEFAULT_APP_PATH}/${appFilename} ${libArgs}`;

        let container_elf_path = format!("{DEFAULT_APP_PATH}/demoAppS.elf");
        let command_string = format!("/home/zondax/speculos/speculos.py --log-level speculos:DEBUG --color JADE_GREEN --display headless -s \"other base behind follow wet put glad muscle unlock sell income october\" -m nanos {container_elf_path}");
        let command_args = vec![command_string.as_str()];

        let apps_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("apps");
        // let volume_bind_string = format!("{}:/speculos/apps", apps_dir.display());
        let volume_bind_string = format!("{}:/project/app/bin", apps_dir.display());
        println!("volume_bind_string: {volume_bind_string}");

        let env_vars = vec![
            "BOLOS_SDK=/project/deps/nanos-secure-sdk",
            "BOLOS_ENV=/opt/bolos",
            "DISPLAY=host.docker.internal:0",
        ];

        let config = Config {
            image: Some(image_name),
            tty: Some(true),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            env: Some(env_vars),
            host_config: Some(HostConfig {
                auto_remove: Some(true),
                port_bindings: Some(port_mapping_hash),
                binds: Some(vec![volume_bind_string]),
                ..Default::default()
            }),
            cmd: Some(command_args),
            ..Default::default()
        };

        let create_container_response = self
            .docker
            .create_container(
                Some(CreateContainerOptions {
                    name: "FIX_ME",
                    ..Default::default()
                }),
                config,
            )
            .await?;

        Ok(create_container_response.id)
    }

    pub async fn start_container_with_defaults(
        &self,
        container_response_id: &str,
    ) -> Result<(), bollard::errors::Error> {
        // deal with this error
        self.docker
            .start_container(container_response_id, None::<StartContainerOptions<String>>)
            .await
    }

    pub async fn stream_logs(&self, container_response_id: &str) {
        let log_options = Some(LogsOptions::<String> {
            follow: true,
            stdout: true,
            stderr: true,
            ..Default::default()
        });

        let logs = self.docker.logs(container_response_id, log_options);
        let logs = logs.try_collect::<Vec<_>>().await;
        println!("{logs:?}");
    }
}

pub async fn connect_to_docker(docker_host: &Option<String>) -> Result<Docker, Error> {
    // if no docker_host is provided, use the default docker host:
    // "unix:///var/run/docker.sock" on unix machines
    // "npipe:////./pipe/docker_engine" on windows machines

    let host = docker_host
        .clone()
        .unwrap_or(DEFAULT_DOCKER_HOST.to_string());

    // this is based on the `connect_with_defaults` method which has not yet been released in the bollard crate
    // https://github.com/fussybeaver/bollard/blob/0972b1aac0ad5c08798e100319ddd0d2ee010365/src/docker.rs#L660
    let connection = match host.clone() {
        // if tcp or http, use connect_with_http_defaults
        // if unix and host starts with "unix://" use connect_with_unix
        // if windows and host starts with "npipe://", use connect_with_named_pipe
        // else default to connect_with_unix
        h if h.starts_with("tcp://") || h.starts_with("http://") => {
            Docker::connect_with_http_defaults()
        }
        #[cfg(unix)]
        h if h.starts_with("unix://") => {
            Docker::connect_with_unix(&h, DEFAULT_TIMEOUT, API_DEFAULT_VERSION)
        }
        #[cfg(windows)]
        h if h.starts_with("npipe://") => {
            Docker::connect_with_named_pipe(&h, DEFAULT_TIMEOUT, API_DEFAULT_VERSION)
        }
        _ => {
            return Err(Error::UnsupportedURISchemeError {
                uri: host.to_string(),
            });
        }
    }?;

    match check_docker_connection(&connection).await {
        Ok(()) => Ok(connection),
        // If we aren't able to connect with the defaults, or with the provided docker_host
        // try to connect with the default docker desktop socket since that is a common use case for devs
        #[allow(unused_variables)]
        Err(e) => {
            // if on unix, try to connect to the default docker desktop socket
            #[cfg(unix)]
            {
                let docker_desktop_connection = try_docker_desktop_socket(&host)?;
                match check_docker_connection(&docker_desktop_connection).await {
                    Ok(()) => Ok(docker_desktop_connection),
                    Err(err) => Err(err)?,
                }
            }

            #[cfg(windows)]
            {
                Err(e)?
            }
        }
    }
}

#[cfg(unix)]
fn try_docker_desktop_socket(host: &str) -> Result<Docker, bollard::errors::Error> {
    let default_docker_desktop_host =
        format!("{}/.docker/run/docker.sock", home_dir().unwrap().display());
    println!("Failed to connect to DOCKER_HOST: {host}.\nTrying to connect to the default Docker Desktop socket at {default_docker_desktop_host}.");

    Docker::connect_with_unix(
        &default_docker_desktop_host,
        DEFAULT_TIMEOUT,
        API_DEFAULT_VERSION,
    )
}

// When bollard is not able to connect to the docker daemon, it returns a generic ConnectionRefused error
// This method attempts to connect to the docker daemon and returns a more specific error message
async fn check_docker_connection(docker: &Docker) -> Result<(), bollard::errors::Error> {
    // This is a bit hacky, but the `client_addr` field is not directly accessible from the `Docker` struct, but we can access it from the debug string representation of the `Docker` struct
    let docker_debug_string = format!("{docker:#?}");
    let start_of_client_addr = docker_debug_string.find("client_addr: ").unwrap();
    let end_of_client_addr = docker_debug_string[start_of_client_addr..]
        .find(',')
        .unwrap();
    // Extract the substring containing the value of client_addr
    let client_addr = &docker_debug_string
        [start_of_client_addr + "client_addr: ".len()..start_of_client_addr + end_of_client_addr]
        .trim()
        .trim_matches('"');

    match docker.version().await {
        Ok(_version) => Ok(()),
        Err(err) => {
            println!(
                "⛔️ Failed to connect to the Docker daemon at {client_addr:?}. Is the docker daemon running?\nℹ️  Running a local Stellar network requires a Docker-compatible container runtime.\nℹ️  Please note that if you are using Docker Desktop, you may need to utilize the `--docker-host` flag to pass in the location of the docker socket on your machine.\n"
            );
            Err(err)
        }
    }
}
