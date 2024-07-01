use async_trait::async_trait;
use soroban_rpc::GetTransactionResponse;

use crate::commands::{config, global, NetworkRunnable};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    XdrArgs(#[from] super::xdr::Error),
    #[error(transparent)]
    Config(#[from] super::super::config::Error),
    #[error(transparent)]
    Rpc(#[from] crate::rpc::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

#[derive(Debug, clap::Parser, Clone)]
#[group(skip)]
/// Command to send a transaction envelope to the network
/// e.g. `cat file.txt | soroban tx send`
pub struct Cmd {
    #[clap(flatten)]
    pub config: super::super::config::Args,
}

impl Cmd {
    pub async fn run(&self, global_args: &global::Args) -> Result<(), Error> {
        let response = self
            .run_against_rpc_server(Some(global_args), Some(&self.config))
            .await?;
        println!("{}", serde_json::to_string_pretty(&response)?);
        Ok(())
    }
}

#[async_trait]
impl NetworkRunnable for Cmd {
    type Error = Error;

    type Result = GetTransactionResponse;
    async fn run_against_rpc_server(
        &self,
        _: Option<&global::Args>,
        config: Option<&config::Args>,
    ) -> Result<Self::Result, Self::Error> {
        let config = config.unwrap_or(&self.config);
        let network = config.get_network()?;
        let client = crate::rpc::Client::new(&network.rpc_url)?;
        let tx_env = super::xdr::tx_envelope_from_stdin()?;
        Ok(client.send_transaction_polling(&tx_env).await?)
    }
}
