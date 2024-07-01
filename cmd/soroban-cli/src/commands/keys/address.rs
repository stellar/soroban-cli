use clap::arg;

use super::super::config::{
    locator::{self, KeyName},
    secret,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] locator::Error),

    #[error(transparent)]
    Secret(#[from] secret::Error),

    #[error(transparent)]
    StrKey(#[from] stellar_strkey::DecodeError),
}

#[derive(Debug, clap::Parser, Clone)]
#[group(skip)]
pub struct Cmd {
    /// Name of identity to lookup, default test identity used if not provided
    pub name: KeyName,

    /// If identity is a seed phrase use this hd path, default is 0
    #[arg(long)]
    pub hd_path: Option<usize>,

    #[command(flatten)]
    pub locator: locator::Args,
}

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        println!("{}", self.public_key().await?);
        Ok(())
    }

    pub async fn public_key(&self) -> Result<stellar_strkey::ed25519::PublicKey, Error> {
        Ok(self
            .locator
            .account(&self.name)?
            .public_key(self.hd_path)
            .await?)
    }
}
