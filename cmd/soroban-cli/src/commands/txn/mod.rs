use clap::Parser;

pub mod build;
pub mod send;
pub mod sign;
pub mod simulate;
pub mod ls;
pub mod rm;

#[derive(Debug, Parser)]
pub enum Cmd {
    /// Simulate a transaction
    Simulate(simulate::Cmd),
    /// Given an identity return its address (public key)
    Sign(sign::Cmd),
    /// Submit a transaction to the network
    Send(send::Cmd),
    /// Build a transaction
    Build(generate::Cmd),
    /// List cached transactions
    Ls(ls::Cmd),
    /// Remove cached transaction(s)
    Rm(rm::Cmd),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Simulate(#[from] simulate::Error),

    #[error(transparent)]
    Sign(#[from] sign::Error),
    #[error(transparent)]
    Send(#[from] fund::Error),

    #[error(transparent)]
    Build(#[from] build::Error),
    #[error(transparent)]
    Rm(#[from] rm::Error),
    #[error(transparent)]
    Ls(#[from] ls::Error),

}

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        match self {
            Cmd::Simulate(cmd) => cmd.run()?,
            Cmd::Sign(cmd) => cmd.run()?,
            Cmd::Send(cmd) => cmd.run().await?,
            Cmd::Build(cmd) => cmd.run().await?,
            Cmd::Ls(cmd) => cmd.run()?,
            Cmd::Rm(cmd) => cmd.run()?,
            
        };
        Ok(())
    }
}
