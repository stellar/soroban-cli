use std::str::FromStr as _;

use serde::{Deserialize, Serialize};
use soroban_rpc::Client;
use stellar_strkey::ed25519::PublicKey;

use crate::{locator, STELLAR_CONFIG};

pub mod passphrase;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] crate::locator::Error),

    #[error("network arg or rpc url and network passphrase are required if using the network")]
    Network,
    #[error("Invalid URL {0}")]
    InvalidUrl(String),
    #[error(transparent)]
    Http(#[from] http::Error),
    #[error(transparent)]
    Rpc(#[from] soroban_rpc::Error),
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error("Failed to parse JSON from {0}, {1}")]
    FailedToParseJSON(String, serde_json::Error),
    #[error("Inproper response {0}")]
    InproperResponse(String),
    #[error("Currently not supported on windows. Please visit:\n{0}")]
    WindowsNotSupported(String),
    #[error("Missing Authority in URL {0}")]
    MissingAuthority(String),
    #[error("Missing Scheme in URL {0}")]
    MissingScheme(String),
}

#[derive(Debug, clap::Args, Clone, Default)]
#[group(skip)]
pub struct Args {
    /// RPC server endpoint
    #[arg(
        long = "rpc-url",
        requires = "network_passphrase",
        required_unless_present = "network",
        env = "SOROBAN_RPC_URL",
        help_heading = STELLAR_CONFIG,
    )]
    pub rpc_url: Option<String>,
    /// Network passphrase to sign the transaction sent to the rpc server
    #[arg(
        long = "network-passphrase",
        requires = "rpc_url",
        required_unless_present = "network",
        env = "SOROBAN_NETWORK_PASSPHRASE",
        help_heading = STELLAR_CONFIG,
    )]
    pub network_passphrase: Option<String>,
    /// Name of network to use from config
    #[arg(
        long,
        required_unless_present = "rpc_url",
        env = "SOROBAN_NETWORK",
        help_heading = STELLAR_CONFIG,
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
        env = "SOROBAN_RPC_URL",
        help_heading = STELLAR_CONFIG,
    )]
    pub rpc_url: String,
    /// Network passphrase to sign the transaction sent to the rpc server
    #[arg(
            long,
            env = "SOROBAN_NETWORK_PASSPHRASE",
            help_heading = STELLAR_CONFIG,
        )]
    pub network_passphrase: String,
}

impl Network {
    pub async fn helper_url(&self, addr: &str) -> Result<http::Uri, Error> {
        use http::Uri;
        tracing::debug!("address {addr:?}");
        let rpc_uri = Uri::from_str(&self.rpc_url)
            .map_err(|_| Error::InvalidUrl(self.rpc_url.to_string()))?;
        if self.network_passphrase.as_str() == passphrase::LOCAL {
            let auth = rpc_uri
                .authority()
                .ok_or_else(|| Error::MissingAuthority(self.rpc_url.clone()))?
                .clone();
            let scheme = rpc_uri
                .scheme_str()
                .ok_or_else(|| Error::MissingScheme(self.rpc_url.clone()))?;

            Ok(Uri::builder()
                .authority(auth)
                .scheme(scheme)
                .path_and_query(format!("/friendbot?addr={addr}"))
                .build()?)
        } else {
            let client = Client::new(&self.rpc_url)?;
            let uri = client.friendbot_url().await?;
            Uri::from_str(&format!("{uri}?addr={addr}")).map_err(|e| {
                tracing::error!("{e}");
                Error::InvalidUrl(uri.to_string())
            })
        }
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
        if let Some(detail) = res.get("detail").and_then(serde_json::Value::as_str) {
            if detail.contains("createAccountAlreadyExist") {
                tracing::warn!("Account already exists");
            }
        } else if res.get("successful").is_none() {
            return Err(Error::InproperResponse(res.to_string()));
        }
        Ok(())
    }
}

impl Network {
    #[must_use]
    pub fn futurenet() -> Self {
        Network {
            rpc_url: "https://rpc-futurenet.stellar.org:443".to_owned(),
            network_passphrase: "Test SDF Future Network ; October 2022".to_owned(),
        }
    }
}
