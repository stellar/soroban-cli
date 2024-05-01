use clap::{arg, command, Parser};

use config;

use crate::utils::contract_id_hash_from_asset;
use crate::utils::parsing::parse_asset;

#[derive(Parser, Debug, Clone)]
#[group(skip)]
pub struct Cmd {
    /// ID of the Stellar classic asset to wrap, e.g. "USDC:G...5"
    #[arg(long)]
    pub asset: String,

    #[command(flatten)]
    pub config: config::Args,
}
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ParseError(#[from] crate::utils::parsing::Error),
    #[error(transparent)]
    ConfigError(#[from] config::Error),
    #[error(transparent)]
    Xdr(#[from] soroban_env_host::xdr::Error),
}
impl Cmd {
    pub fn run(&self) -> Result<(), Error> {
        let asset = parse_asset(&self.asset)?;
        let network = self.config.get_network()?;
        let contract_id = contract_id_hash_from_asset(&asset, &network.network_passphrase)?;
        let strkey_contract_id = stellar_strkey::Contract(contract_id.0).to_string();
        println!("{strkey_contract_id}");
        Ok(())
    }
}
