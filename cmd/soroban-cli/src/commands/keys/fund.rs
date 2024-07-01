use clap::command;

use crate::commands::network;

use super::address;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Address(#[from] address::Error),
    #[error(transparent)]
    Network(#[from] network::Error),
}

#[derive(Debug, clap::Parser, Clone)]
#[group(skip)]
pub struct Cmd {
    #[command(flatten)]
    pub network: network::Args,
    /// Address to fund
    #[command(flatten)]
    pub address: address::Cmd,
}

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        let addr = self.address.public_key().await?;
        self.network
            .get(&self.address.locator)?
            .fund_address(&addr)
            .await?;
        Ok(())
    }
}
