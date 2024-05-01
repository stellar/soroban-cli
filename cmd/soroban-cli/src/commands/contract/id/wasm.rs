use clap::{arg, command, Parser};
use sha2::{Digest, Sha256};
use soroban_env_host::xdr::{
    self, AccountId, ContractIdPreimage, ContractIdPreimageFromAddress, Hash, HashIdPreimage,
    HashIdPreimageContractId, Limits, PublicKey, ScAddress, Uint256, WriteXdr,
};

use config;

#[derive(Parser, Debug, Clone)]
#[group(skip)]
pub struct Cmd {
    /// ID of the Soroban contract
    #[arg(long)]
    pub salt: String,

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
    Xdr(#[from] xdr::Error),
    #[error("cannot parse salt {0}")]
    CannotParseSalt(String),
}
impl Cmd {
    pub fn run(&self) -> Result<(), Error> {
        let salt: [u8; 32] = soroban_spec_tools::utils::padded_hex_from_str(&self.salt, 32)
            .map_err(|_| Error::CannotParseSalt(self.salt.clone()))?
            .try_into()
            .map_err(|_| Error::CannotParseSalt(self.salt.clone()))?;
        let contract_id_preimage =
            contract_preimage(&self.config.key_pair()?.verifying_key(), salt);
        let contract_id = get_contract_id(
            contract_id_preimage.clone(),
            &self.config.get_network()?.network_passphrase,
        )?;
        let strkey_contract_id = stellar_strkey::Contract(contract_id.0).to_string();
        println!("{strkey_contract_id}");
        Ok(())
    }
}

pub fn contract_preimage(key: &ed25519_dalek::VerifyingKey, salt: [u8; 32]) -> ContractIdPreimage {
    let source_account = AccountId(PublicKey::PublicKeyTypeEd25519(key.to_bytes().into()));
    ContractIdPreimage::Address(ContractIdPreimageFromAddress {
        address: ScAddress::Account(source_account),
        salt: Uint256(salt),
    })
}

pub fn get_contract_id(
    contract_id_preimage: ContractIdPreimage,
    network_passphrase: &str,
) -> Result<Hash, Error> {
    let network_id = Hash(Sha256::digest(network_passphrase.as_bytes()).into());
    let preimage = HashIdPreimage::ContractId(HashIdPreimageContractId {
        network_id,
        contract_id_preimage,
    });
    let preimage_xdr = preimage.to_xdr(Limits::none())?;
    Ok(Hash(Sha256::digest(preimage_xdr).into()))
}
