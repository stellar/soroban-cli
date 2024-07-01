use clap::Parser;

use super::global;

pub mod send;
pub mod sign;
pub mod simulate;
pub mod xdr;

#[derive(Debug, Parser)]
pub enum Cmd {
    /// Simulate a transaction envelope from stdin
    Simulate(simulate::Cmd),
    /// Sign a transaction
    Sign(sign::Cmd),
    /// Send a transaction envelope to the network
    Send(send::Cmd),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Simulate(#[from] simulate::Error),
    #[error(transparent)]
    Send(#[from] send::Error),
    #[error(transparent)]
    Sign(#[from] sign::Error),
}

impl Cmd {
    pub async fn run(&self, global_args: &global::Args) -> Result<(), Error> {
        match self {
            Cmd::Simulate(cmd) => cmd.run(global_args).await?,
            Cmd::Sign(cmd) => cmd.run().await?,
            Cmd::Send(cmd) => cmd.run(global_args).await?,
        };
        Ok(())
    }
}
