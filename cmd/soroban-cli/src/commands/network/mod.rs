pub mod add;
pub mod ls;
pub mod rm;
pub mod shared;
pub mod start;
pub mod stop;

#[derive(Debug, clap::Parser)]
pub enum Cmd {
    /// Add a new network
    Add(add::Cmd),
    /// Remove a network
    Rm(rm::Cmd),
    /// List networks
    Ls(ls::Cmd),
    /// Start network
    ///
    /// Start a container running a Stellar node, RPC, API, and friendbot (faucet).
    ///
    /// soroban network start <NETWORK> [OPTIONS]
    ///
    /// By default, when starting a testnet container, without any optional arguments, it will run the equivalent of the following docker command:
    /// docker run --rm -p 8000:8000 --name stellar stellar/quickstart:testing --testnet --enable-soroban-rpc
    Start(start::Cmd),
    /// Stop a network started with `network start`. For example, if you ran `soroban network start local`, you can use `soroban network stop local` to stop it.
    Stop(stop::Cmd),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Add(#[from] add::Error),

    #[error(transparent)]
    Rm(#[from] rm::Error),

    #[error(transparent)]
    Ls(#[from] ls::Error),

    #[error(transparent)]
    Start(#[from] start::Error),

    #[error(transparent)]
    Stop(#[from] stop::Error),
}

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        match self {
            Cmd::Add(cmd) => cmd.run()?,
            Cmd::Rm(new) => new.run()?,
            Cmd::Ls(cmd) => cmd.run()?,
            Cmd::Start(cmd) => cmd.run().await?,
            Cmd::Stop(cmd) => cmd.run().await?,
        };
        Ok(())
    }
}
