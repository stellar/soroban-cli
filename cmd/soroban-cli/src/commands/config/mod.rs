use std::path::PathBuf;

use clap::{arg, command, Parser};
use serde::{Deserialize, Serialize};

use crate::Pwd;

use self::{network::Network, secret::Secret};

use super::{keys, network};

pub mod data;
pub mod locator;
pub mod secret;

#[derive(Debug, Parser)]
pub enum Cmd {
    /// Configure different networks. Depraecated, use `soroban network` instead.
    #[command(subcommand)]
    Network(network::Cmd),
    /// Identity management. Deprecated, use `soroban keys` instead.
    #[command(subcommand)]
    Identity(keys::Cmd),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Identity(#[from] keys::Error),
    #[error(transparent)]
    Network(#[from] network::Error),
    #[error(transparent)]
    Secret(#[from] secret::Error),
    #[error(transparent)]
    Config(#[from] locator::Error),
}

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        match &self {
            Cmd::Identity(identity) => identity.run().await?,
            Cmd::Network(network) => network.run().await?,
        }
        Ok(())
    }
}

#[derive(Debug, clap::Args, Clone, Default)]
#[group(skip)]
pub struct Args {
    #[command(flatten)]
    pub network: network::Args,

    #[arg(long, visible_alias = "source", env = "SOROBAN_ACCOUNT")]
    /// Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…"). Default: `identity generate --default-seed`
    pub source_account: String,

    #[arg(long)]
    /// If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
    pub hd_path: Option<usize>,

    #[command(flatten)]
    pub locator: locator::Args,
}

impl Args {
    pub fn key_pair(&self) -> Result<ed25519_dalek::SigningKey, Error> {
        let key = self.account(&self.source_account)?;
        Ok(key.key_pair(self.hd_path)?)
    }

    pub fn account(&self, account_str: &str) -> Result<Secret, Error> {
        if let Ok(secret) = self.locator.read_identity(account_str) {
            Ok(secret)
        } else {
            Ok(account_str.parse::<Secret>()?)
        }
    }

    pub fn get_network(&self) -> Result<Network, Error> {
        Ok(self.network.get(&self.locator)?)
    }

    pub fn config_dir(&self) -> Result<PathBuf, Error> {
        Ok(self.locator.config_dir()?)
    }
}

impl Pwd for Args {
    fn set_pwd(&mut self, pwd: &std::path::Path) {
        self.locator.set_pwd(pwd);
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Config {}
