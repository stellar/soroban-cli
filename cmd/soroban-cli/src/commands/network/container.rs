pub(crate) mod logs;
mod shared;
pub(crate) mod start;
pub(crate) mod stop;

// TODO: remove once `network start` is removed
pub type StartCmd = start::Cmd;
// TODO: remove once `network top` is removed
pub type StopCmd = stop::Cmd;

#[derive(Debug, clap::Subcommand)]
pub enum Cmd {
    /// Tail logs of a running network container
    Logs(logs::Cmd),
    /// Start network
    ///
    /// Start a container running a Stellar node, RPC, API, and friendbot (faucet).
    ///
    /// stellar network container start <NETWORK> [OPTIONS]
    ///
    /// By default, when starting a testnet container, without any optional arguments, it will run the equivalent of the following docker command:
    /// docker run --rm -p 8000:8000 --name stellar stellar/quickstart:testing --testnet --enable rpc,horizon
    Start(start::Cmd),
    /// Stop a network started with `network container start`. For example, if you ran `network container start local`, you can use `network container stop local` to stop it.
    Stop(stop::Cmd),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Logs(#[from] logs::Error),

    #[error(transparent)]
    Start(#[from] start::Error),

    #[error(transparent)]
    Stop(#[from] stop::Error),
}

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        match &self {
            Cmd::Logs(cmd) => cmd.run().await?,
            Cmd::Start(cmd) => cmd.run().await?,
            Cmd::Stop(cmd) => cmd.run().await?,
        }
        Ok(())
    }
}
