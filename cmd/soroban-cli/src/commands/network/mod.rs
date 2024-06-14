use std::str::FromStr;

use clap::{arg, Parser};
use http::Uri;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stellar_strkey::ed25519::PublicKey;

use crate::{
    commands::HEADING_NETWORK,
    rpc::{self, Client},
};

use super::config::locator;

pub const LOCAL_NETWORK_PASSPHRASE: &str = "Standalone Network ; February 2017";

pub mod add;
pub mod container;
pub mod ls;
pub mod rm;

#[derive(Debug, Parser)]
pub enum Cmd {
    /// Add a new network
    Add(add::Cmd),
    /// Remove a network
    Rm(rm::Cmd),
    /// List networks
    Ls(ls::Cmd),
    /// ⚠️ Deprecated: use `soroban container start` instead
    ///
    /// Start network
    ///
    /// Start a container running a Stellar node, RPC, API, and friendbot (faucet).
    ///
    /// soroban network start <NETWORK> [OPTIONS]
    ///
    /// By default, when starting a testnet container, without any optional arguments, it will run the equivalent of the following docker command:
    /// docker run --rm -p 8000:8000 --name stellar stellar/quickstart:testing --testnet --enable-soroban-rpc
    Start(container::StartCmd),
    /// ⚠️ Deprecated: use `soroban container stop` instead
    ///
    /// Stop a network started with `network start`. For example, if you ran `soroban network start local`, you can use `soroban network stop local` to stop it.
    Stop(container::StopCmd),

    /// Commands to start, stop and get logs for a quickstart container
    #[command(subcommand)]
    Container(container::Cmd),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Add(#[from] add::Error),

    #[error(transparent)]
    Rm(#[from] rm::Error),

    #[error(transparent)]
    Ls(#[from] ls::Error),

    // TODO: remove once `network start` is removed
    #[error(transparent)]
    Start(#[from] container::start::Error),

    // TODO: remove once `network stop` is removed
    #[error(transparent)]
    Stop(#[from] container::stop::Error),

    #[error(transparent)]
    Container(#[from] container::Error),

    #[error(transparent)]
    Config(#[from] locator::Error),

    #[error("network arg or rpc url and network passphrase are required if using the network")]
    Network,
    #[error(transparent)]
    Http(#[from] http::Error),
    #[error(transparent)]
    Rpc(#[from] rpc::Error),
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error("Failed to parse JSON from {0}, {1}")]
    FailedToParseJSON(String, serde_json::Error),
    #[error("Invalid URL {0}")]
    InvalidUrl(String),
    #[error("Inproper response {0}")]
    InproperResponse(String),
    #[error("Currently not supported on windows. Please visit:\n{0}")]
    WindowsNotSupported(String),
    #[error("Archive URL not configured")]
    ArchiveUrlNotConfigured,
}

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        match self {
            Cmd::Add(cmd) => cmd.run()?,
            Cmd::Rm(new) => new.run()?,
            Cmd::Ls(cmd) => cmd.run()?,
            Cmd::Container(cmd) => cmd.run().await?,

            // TODO Remove this once `network start` is removed
            Cmd::Start(cmd) => {
                eprintln!("⚠️ Warning: `network start` has been deprecated. Use `network container start` instead");
                cmd.run().await?;
            }
            // TODO Remove this once `network stop` is removed
            Cmd::Stop(cmd) => {
                println!("⚠️ Warning: `network stop` has been deprecated. Use `network container stop` instead");
                cmd.run().await?;
            }
        };
        Ok(())
    }
}

#[derive(Debug, clap::Args, Clone, Default)]
#[group(skip)]
pub struct Args {
    /// RPC server endpoint
    #[arg(
        long = "rpc-url",
        requires = "network_passphrase",
        required_unless_present = "network",
        env = "STELLAR_RPC_URL",
        help_heading = HEADING_NETWORK,
    )]
    pub rpc_url: Option<String>,
    /// Network passphrase to sign the transaction sent to the rpc server
    #[arg(
        long = "network-passphrase",
        requires = "rpc_url",
        required_unless_present = "network",
        env = "STELLAR_NETWORK_PASSPHRASE",
        help_heading = HEADING_NETWORK,
    )]
    pub network_passphrase: Option<String>,
    /// Archive URL
    #[arg(
        long = "archive-url",
        requires = "network_passphrase",
        env = "STELLAR_ARCHIVE_URL",
        help_heading = HEADING_NETWORK,
    )]
    pub archive_url: Option<String>,
    /// Name of network to use from config
    #[arg(
        long,
        required_unless_present = "rpc_url",
        env = "STELLAR_NETWORK",
        help_heading = HEADING_NETWORK,
    )]
    pub network: Option<String>,
}

impl Args {
    pub fn get(&self, locator: &locator::Args) -> Result<Network, Error> {
        if let Some(name) = self.network.as_deref() {
            if let Ok(network) = locator.read_network(name) {
                return Ok(network);
            }
        }
        if let (Some(rpc_url), Some(network_passphrase)) =
            (self.rpc_url.clone(), self.network_passphrase.clone())
        {
            Ok(Network {
                rpc_url,
                network_passphrase,
                archive_url: self.archive_url.clone(),
            })
        } else {
            Err(Error::Network)
        }
    }
}

#[derive(Debug, clap::Args, Serialize, Deserialize, Clone)]
#[group(skip)]
pub struct Network {
    /// RPC server endpoint
    #[arg(
        long = "rpc-url",
        env = "STELLAR_RPC_URL",
        help_heading = HEADING_NETWORK,
    )]
    pub rpc_url: String,
    /// Network passphrase to sign the transaction sent to the rpc server
    #[arg(
        long,
        env = "STELLAR_NETWORK_PASSPHRASE",
        help_heading = HEADING_NETWORK,
    )]
    pub network_passphrase: String,
    /// Archive URL
    #[arg(
        long = "archive-url",
        env = "STELLAR_ARCHIVE_URL",
        help_heading = HEADING_NETWORK,
    )]
    pub archive_url: Option<String>,
}

impl Network {
    pub async fn helper_url(&self, addr: &str) -> Result<http::Uri, Error> {
        tracing::debug!("address {addr:?}");
        let rpc_uri = Uri::from_str(&self.rpc_url)
            .map_err(|_| Error::InvalidUrl(self.rpc_url.to_string()))?;
        if self.network_passphrase.as_str() == LOCAL_NETWORK_PASSPHRASE {
            let auth = rpc_uri.authority().unwrap().clone();
            let scheme = rpc_uri.scheme_str().unwrap();
            Ok(Uri::builder()
                .authority(auth)
                .scheme(scheme)
                .path_and_query(format!("/friendbot?addr={addr}"))
                .build()?)
        } else {
            let client = Client::new(&self.rpc_url)?;
            let network = client.get_network().await?;
            tracing::debug!("network {network:?}");
            let uri = client.friendbot_url().await?;
            tracing::debug!("URI {uri:?}");
            Uri::from_str(&format!("{uri}?addr={addr}")).map_err(|e| {
                tracing::error!("{e}");
                Error::InvalidUrl(uri.to_string())
            })
        }
    }

    pub fn archive_url(&self) -> Result<http::Uri, Error> {
        // Return the configured archive URL, or if one is not configured, guess
        // at an appropriate archive URL given the network passphrase.
        self.archive_url
            .as_deref()
            .or(match self.network_passphrase.as_str() {
                "Public Global Stellar Network ; September 2015" => {
                    Some("https://history.stellar.org/prd/core-live/core_live_001")
                }
                "Test SDF Network ; September 2015" => {
                    Some("https://history.stellar.org/prd/core-testnet/core_testnet_001")
                }
                "Test SDF Future Network ; October 2022" => {
                    Some("https://history-futurenet.stellar.org")
                }
                _ => None,
            })
            .ok_or(Error::ArchiveUrlNotConfigured)
            .and_then(|archive_url| {
                Uri::from_str(archive_url)
                    .map_err(|_| Error::InvalidUrl((*archive_url).to_string()))
            })
    }

    #[allow(clippy::similar_names)]
    pub async fn fund_address(&self, addr: &PublicKey) -> Result<(), Error> {
        let uri = self.helper_url(&addr.to_string()).await?;
        tracing::debug!("URL {uri:?}");
        let response = match uri.scheme_str() {
            Some("http") => hyper::Client::new().get(uri.clone()).await?,
            Some("https") => {
                let https = hyper_tls::HttpsConnector::new();
                hyper::Client::builder()
                    .build::<_, hyper::Body>(https)
                    .get(uri.clone())
                    .await?
            }
            _ => {
                return Err(Error::InvalidUrl(uri.to_string()));
            }
        };
        let body = hyper::body::to_bytes(response.into_body()).await?;
        let res = serde_json::from_slice::<serde_json::Value>(&body)
            .map_err(|e| Error::FailedToParseJSON(uri.to_string(), e))?;
        tracing::debug!("{res:#?}");
        if let Some(detail) = res.get("detail").and_then(Value::as_str) {
            if detail.contains("createAccountAlreadyExist") {
                eprintln!("Account already exists");
            }
        } else if res.get("successful").is_none() {
            return Err(Error::InproperResponse(res.to_string()));
        }
        Ok(())
    }

    pub fn rpc_uri(&self) -> Result<http::Uri, Error> {
        http::Uri::from_str(&self.rpc_url).map_err(|_| Error::InvalidUrl(self.rpc_url.to_string()))
    }
}

impl Network {
    pub fn pubnet() -> Self {
        Network {
            rpc_url: String::new(),
            network_passphrase: "Public Global Stellar Network ; September 2015".to_owned(),
            archive_url: None,
        }
    }
    pub fn testnet() -> Self {
        Network {
            rpc_url: "https://soroban-testnet.stellar.org:443".to_owned(),
            network_passphrase: "Test SDF Network ; September 2015".to_owned(),
            archive_url: None,
        }
    }
    pub fn futurenet() -> Self {
        Network {
            rpc_url: "https://rpc-futurenet.stellar.org:443".to_owned(),
            network_passphrase: "Test SDF Future Network ; October 2022".to_owned(),
            archive_url: None,
        }
    }
}
