use std::array::TryFromSliceError;
use std::fmt::Debug;
use std::num::ParseIntError;

use clap::{arg, command, Parser};
use rand::Rng;
use soroban_env_host::{
    xdr::{
        AccountId, ContractExecutable, ContractIdPreimage, ContractIdPreimageFromAddress,
        CreateContractArgs, Error as XdrError, Hash, HostFunction, InvokeHostFunctionOp, Memo,
        MuxedAccount, Operation, OperationBody, Preconditions, PublicKey, ScAddress,
        SequenceNumber, Transaction, TransactionExt, Uint256, VecM,
    },
    HostError,
};

use crate::commands::{
    contract::{self, id::wasm::get_contract_id},
    global, NetworkRunnable,
};
use crate::{
    commands::{contract::install, HEADING_RPC},
    rpc::{self, Client},
    utils, wasm,
};

#[derive(Parser, Debug, Clone)]
#[command(group(
    clap::ArgGroup::new("wasm_src")
        .required(true)
        .args(&["wasm", "wasm_hash"]),
))]
#[group(skip)]
pub struct Cmd {
    /// WASM file to deploy
    #[arg(long, group = "wasm_src")]
    wasm: Option<std::path::PathBuf>,
    /// Hash of the already installed/deployed WASM file
    #[arg(long = "wasm-hash", conflicts_with = "wasm", group = "wasm_src")]
    wasm_hash: Option<String>,
    /// Custom salt 32-byte salt for the token id
    #[arg(
        long,
        help_heading = HEADING_RPC,
    )]
    salt: Option<String>,
    #[command(flatten)]
    config: config::Args,
    #[command(flatten)]
    pub fee: crate::fee::Args,
    #[arg(long, short = 'i', default_value = "false")]
    /// Whether to ignore safety checks when deploying contracts
    pub ignore_checks: bool,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Install(#[from] install::Error),
    #[error(transparent)]
    Host(#[from] HostError),
    #[error("error parsing int: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("internal conversion error: {0}")]
    TryFromSliceError(#[from] TryFromSliceError),
    #[error("xdr processing error: {0}")]
    Xdr(#[from] XdrError),
    #[error("jsonrpc error: {0}")]
    JsonRpc(#[from] jsonrpsee_core::Error),
    #[error("cannot parse salt: {salt}")]
    CannotParseSalt { salt: String },
    #[error("cannot parse contract ID {contract_id}: {error}")]
    CannotParseContractId {
        contract_id: String,
        error: stellar_strkey::DecodeError,
    },
    #[error("cannot parse WASM hash {wasm_hash}: {error}")]
    CannotParseWasmHash {
        wasm_hash: String,
        error: stellar_strkey::DecodeError,
    },
    #[error("Must provide either --wasm or --wash-hash")]
    WasmNotProvided,
    #[error(transparent)]
    Rpc(#[from] rpc::Error),
    #[error(transparent)]
    Config(#[from] config::Error),
    #[error(transparent)]
    StrKey(#[from] stellar_strkey::DecodeError),
    #[error(transparent)]
    Infallible(#[from] std::convert::Infallible),
    #[error(transparent)]
    WasmId(#[from] contract::id::wasm::Error),
}

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        let res_str = self.run_against_rpc_server(None, None).await?;
        println!("{res_str}");
        Ok(())
    }
}

#[async_trait::async_trait]
impl NetworkRunnable for Cmd {
    type Error = Error;
    type Result = String;

    async fn run_against_rpc_server(
        &self,
        global_args: Option<&global::Args>,
        config: Option<&config::Args>,
    ) -> Result<String, Error> {
        let config = config.unwrap_or(&self.config);
        let wasm_hash = if let Some(wasm) = &self.wasm {
            let hash = install::Cmd {
                wasm: wasm::Args { wasm: wasm.clone() },
                config: config.clone(),
                fee: self.fee.clone(),
                ignore_checks: self.ignore_checks,
            }
            .run_against_rpc_server(global_args, Some(config))
            .await?;
            hex::encode(hash)
        } else {
            self.wasm_hash
                .as_ref()
                .ok_or(Error::WasmNotProvided)?
                .to_string()
        };

        let wasm_hash = Hash(utils::contract_id_from_str(&wasm_hash).map_err(|e| {
            Error::CannotParseWasmHash {
                wasm_hash: wasm_hash.clone(),
                error: e,
            }
        })?);
        let network = config.get_network()?;
        let salt: [u8; 32] = match &self.salt {
            Some(h) => soroban_spec_tools::utils::padded_hex_from_str(h, 32)
                .map_err(|_| Error::CannotParseSalt { salt: h.clone() })?
                .try_into()
                .map_err(|_| Error::CannotParseSalt { salt: h.clone() })?,
            None => rand::thread_rng().gen::<[u8; 32]>(),
        };

        let client = Client::new(&network.rpc_url)?;
        client
            .verify_network_passphrase(Some(&network.network_passphrase))
            .await?;
        let key = config.key_pair()?;

        // Get the account sequence number
        let public_strkey =
            stellar_strkey::ed25519::PublicKey(key.verifying_key().to_bytes()).to_string();

        let account_details = client.get_account(&public_strkey).await?;
        let sequence: i64 = account_details.seq_num.into();
        let (txn, contract_id) = build_create_contract_tx(
            wasm_hash,
            sequence + 1,
            self.fee.fee,
            &network.network_passphrase,
            salt,
            &key,
        )?;
        let txn = client.create_assembled_transaction(&txn).await?;
        let txn = self.fee.apply_to_assembled_txn(txn);
        client
            .send_assembled_transaction(txn, &key, &[], &network.network_passphrase, None, None)
            .await?;
        Ok(stellar_strkey::Contract(contract_id.0).to_string())
    }
}

fn build_create_contract_tx(
    hash: Hash,
    sequence: i64,
    fee: u32,
    network_passphrase: &str,
    salt: [u8; 32],
    key: &ed25519_dalek::SigningKey,
) -> Result<(Transaction, Hash), Error> {
    let source_account = AccountId(PublicKey::PublicKeyTypeEd25519(
        key.verifying_key().to_bytes().into(),
    ));

    let contract_id_preimage = ContractIdPreimage::Address(ContractIdPreimageFromAddress {
        address: ScAddress::Account(source_account),
        salt: Uint256(salt),
    });
    let contract_id = get_contract_id(contract_id_preimage.clone(), network_passphrase)?;

    let op = Operation {
        source_account: None,
        body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
            host_function: HostFunction::CreateContract(CreateContractArgs {
                contract_id_preimage,
                executable: ContractExecutable::Wasm(hash),
            }),
            auth: VecM::default(),
        }),
    };
    let tx = Transaction {
        source_account: MuxedAccount::Ed25519(Uint256(key.verifying_key().to_bytes())),
        fee,
        seq_num: SequenceNumber(sequence),
        cond: Preconditions::None,
        memo: Memo::None,
        operations: vec![op].try_into()?,
        ext: TransactionExt::V0,
    };

    Ok((tx, Hash(contract_id.into())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_create_contract() {
        let hash = hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
            .unwrap()
            .try_into()
            .unwrap();
        let result = build_create_contract_tx(
            Hash(hash),
            300,
            1,
            "Public Global Stellar Network ; September 2015",
            [0u8; 32],
            &utils::parse_secret_key("SBFGFF27Y64ZUGFAIG5AMJGQODZZKV2YQKAVUUN4HNE24XZXD2OEUVUP")
                .unwrap(),
        );

        assert!(result.is_ok());
    }
}
