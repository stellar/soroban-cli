use std::path::PathBuf;

use clap::{arg, command};
use secret::StellarSigner;
use serde::{Deserialize, Serialize};
use stellar_strkey::ed25519::PublicKey;

use crate::signer;
use crate::xdr::{Transaction, TransactionEnvelope};
use crate::{signer::Stellar, Pwd};

use self::network::Network;

use super::{keys, network};

pub mod alias;
pub mod data;
pub mod locator;
pub mod secret;

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
    #[error(transparent)]
    Rpc(#[from] soroban_rpc::Error),
    #[error(transparent)]
    Signer(#[from] signer::Error),
}

#[derive(Debug, clap::Args, Clone, Default)]
#[group(skip)]
pub struct Args {
    #[command(flatten)]
    pub network: network::Args,

    #[arg(long, visible_alias = "source", env = "STELLAR_ACCOUNT")]
    /// Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…").
    pub source_account: String,

    #[arg(long)]
    /// If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
    pub hd_path: Option<usize>,

    #[command(flatten)]
    pub locator: locator::Args,

    /// Check with user before signature. Eventually this will be replaced with `--yes`, which does the opposite and will force a check without --yes
    #[arg(long)]
    pub check: bool,
}

impl Args {
    pub fn signer(&self) -> Result<StellarSigner, Error> {
        Ok(self
            .locator
            .account(&self.source_account)?
            .signer(self.hd_path, self.check)?)
    }

    pub fn key_pair(&self) -> Result<ed25519_dalek::SigningKey, Error> {
        let key = self.locator.account(&self.source_account)?;
        Ok(key.key_pair(self.hd_path)?)
    }

    pub async fn public_key(&self) -> Result<PublicKey, Error> {
        Ok(self.signer()?.get_public_key().await?)
    }

    pub async fn sign(&self, tx: Transaction) -> Result<TransactionEnvelope, Error> {
        let signer = self.signer()?;
        self.sign_with_signer(&signer, tx).await
    }

    pub async fn sign_with_signer(
        &self,
        signer: &(impl Stellar + std::marker::Sync),
        tx: Transaction,
    ) -> Result<TransactionEnvelope, Error> {
        let Network {
            network_passphrase, ..
        } = &self.get_network()?;
        Ok(signer.sign_txn(tx, network_passphrase).await?)
    }

    pub async fn sign_soroban_authorizations(
        &self,
        signer: &(impl Stellar + std::marker::Sync),
        tx: &Transaction,
    ) -> Result<Option<Transaction>, Error> {
        let network = self.get_network()?;
        Ok(signer
            .sign_soroban_authorizations(tx, &network.network_passphrase)
            .await?)
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
