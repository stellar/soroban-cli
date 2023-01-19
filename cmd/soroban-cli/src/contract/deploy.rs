use std::array::TryFromSliceError;
use std::fmt::Debug;
use std::num::ParseIntError;

use clap::Parser;
use hex::FromHexError;
use rand::Rng;
use sha2::{Digest, Sha256};
use soroban_env_host::xdr::HashIdPreimageSourceAccountContractId;
use soroban_env_host::xdr::{
    AccountId, ContractId, CreateContractArgs, Error as XdrError, Hash, HashIdPreimage,
    HostFunction, InvokeHostFunctionOp, LedgerFootprint, LedgerKey::ContractCode,
    LedgerKey::ContractData, LedgerKeyContractCode, LedgerKeyContractData, Memo, MuxedAccount,
    Operation, OperationBody, Preconditions, PublicKey, ScContractCode, ScStatic, ScVal,
    SequenceNumber, Transaction, TransactionEnvelope, TransactionExt, Uint256, WriteXdr,
};
use soroban_env_host::HostError;

use crate::contract::install;
use crate::rpc::{self, Client};
use crate::{utils, HEADING_RPC, HEADING_SANDBOX};

#[derive(Parser, Debug)]
#[clap(group(
    clap::ArgGroup::new("wasm_src")
        .required(true)
        .args(&["wasm", "wasm-hash"]),
))]
pub struct Cmd {
    /// WASM file to deploy
    #[clap(long, parse(from_os_str), group = "wasm_src")]
    wasm: Option<std::path::PathBuf>,

    /// Hash of the already installed/deployed WASM file
    #[clap(long = "wasm-hash", conflicts_with = "wasm", group = "wasm_src")]
    wasm_hash: Option<String>,

    /// Contract ID to deploy to
    #[clap(
        long = "id",
        conflicts_with = "rpc-url",
        help_heading = HEADING_SANDBOX,
    )]
    contract_id: Option<String>,
    /// File to persist ledger state
    #[clap(
        long,
        parse(from_os_str),
        default_value = ".soroban/ledger.json",
        conflicts_with = "rpc-url",
        env = "SOROBAN_LEDGER_FILE",
        help_heading = HEADING_SANDBOX,
    )]
    ledger_file: std::path::PathBuf,

    /// Secret 'S' key used to sign the transaction sent to the rpc server
    #[clap(
        long = "secret-key",
        env = "SOROBAN_SECRET_KEY",
        help_heading = HEADING_RPC,
    )]
    secret_key: Option<String>,
    /// Custom salt 32-byte salt for the token id
    #[clap(
        long,
        conflicts_with_all = &["contract-id", "ledger-file"],
        help_heading = HEADING_RPC,
    )]
    salt: Option<String>,
    /// RPC server endpoint
    #[clap(
        long,
        conflicts_with = "contract-id",
        requires = "secret-key",
        requires = "network-passphrase",
        env = "SOROBAN_RPC_URL",
        help_heading = HEADING_RPC,
    )]
    rpc_url: Option<String>,
    /// Network passphrase to sign the transaction sent to the rpc server
    #[clap(
        long = "network-passphrase",
        env = "SOROBAN_NETWORK_PASSPHRASE",
        help_heading = HEADING_RPC,
    )]
    network_passphrase: Option<String>,
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
    #[error("reading file {filepath}: {error}")]
    CannotReadLedgerFile {
        filepath: std::path::PathBuf,
        error: soroban_ledger_snapshot::Error,
    },
    #[error("committing file {filepath}: {error}")]
    CannotCommitLedgerFile {
        filepath: std::path::PathBuf,
        error: soroban_ledger_snapshot::Error,
    },
    #[error("cannot parse contract ID {contract_id}: {error}")]
    CannotParseContractId {
        contract_id: String,
        error: FromHexError,
    },
    #[error("cannot parse WASM hash {wasm_hash}: {error}")]
    CannotParseWasmHash {
        wasm_hash: String,
        error: FromHexError,
    },
    #[error("cannot parse secret key")]
    CannotParseSecretKey,
    #[error(transparent)]
    Rpc(#[from] rpc::Error),
}

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        let res_str = self.run_and_get_contract_id().await?;
        println!("{res_str}");
        Ok(())
    }

    pub async fn run_and_get_contract_id(&self) -> Result<String, Error> {
        let wasm_hash = if let Some(wasm) = &self.wasm {
            install::Cmd {
                wasm: wasm.clone(),
                ledger_file: self.ledger_file.clone(),
                secret_key: self.secret_key.clone(),
                rpc_url: self.rpc_url.clone(),
                network_passphrase: self.network_passphrase.clone(),
            }
            .run_and_get_hash()
            .await?
        } else {
            self.wasm_hash.as_ref().unwrap().to_string()
        };

        let hash =
            Hash(
                utils::id_from_str(&wasm_hash).map_err(|e| Error::CannotParseWasmHash {
                    wasm_hash: wasm_hash.clone(),
                    error: e,
                })?,
            );

        if self.rpc_url.is_some() {
            self.run_against_rpc_server(hash).await
        } else {
            self.run_in_sandbox(hash)
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn run_in_sandbox(&self, wasm_hash: Hash) -> Result<String, Error> {
        let contract_id: [u8; 32] = match &self.contract_id {
            Some(id) => utils::id_from_str(id).map_err(|e| Error::CannotParseContractId {
                contract_id: self.contract_id.as_ref().unwrap().clone(),
                error: e,
            })?,
            None => rand::thread_rng().gen::<[u8; 32]>(),
        };

        let mut state = utils::ledger_snapshot_read_or_default(&self.ledger_file).map_err(|e| {
            Error::CannotReadLedgerFile {
                filepath: self.ledger_file.clone(),
                error: e,
            }
        })?;
        utils::add_contract_to_ledger_entries(&mut state.ledger_entries, contract_id, wasm_hash.0);

        state
            .write_file(&self.ledger_file)
            .map_err(|e| Error::CannotCommitLedgerFile {
                filepath: self.ledger_file.clone(),
                error: e,
            })?;

        Ok(hex::encode(contract_id))
    }

    async fn run_against_rpc_server(&self, wasm_hash: Hash) -> Result<String, Error> {
        let salt: [u8; 32] = match &self.salt {
            // Hack: re-use contract_id_from_str to parse the 32-byte salt hex.
            Some(h) => {
                utils::id_from_str(h).map_err(|_| Error::CannotParseSalt { salt: h.clone() })?
            }
            None => rand::thread_rng().gen::<[u8; 32]>(),
        };

        let client = Client::new(self.rpc_url.as_ref().unwrap());
        let key = utils::parse_secret_key(self.secret_key.as_ref().unwrap())
            .map_err(|_| Error::CannotParseSecretKey)?;

        // Get the account sequence number
        let public_strkey = stellar_strkey::ed25519::PublicKey(key.public.to_bytes()).to_string();
        // TODO: create a cmdline parameter for the fee instead of simply using the minimum fee
        let fee: u32 = 100;

        let account_details = client.get_account(&public_strkey).await?;
        let sequence = account_details.sequence.parse::<i64>()?;
        let (tx, contract_id) = build_create_contract_tx(
            wasm_hash,
            sequence + 1,
            fee,
            self.network_passphrase.as_ref().unwrap(),
            salt,
            &key,
        )?;
        client.send_transaction(&tx).await?;

        Ok(hex::encode(contract_id.0))
    }
}

fn build_create_contract_tx(
    hash: Hash,
    sequence: i64,
    fee: u32,
    network_passphrase: &str,
    salt: [u8; 32],
    key: &ed25519_dalek::Keypair,
) -> Result<(TransactionEnvelope, Hash), Error> {
    let network_id = Hash(Sha256::digest(network_passphrase.as_bytes()).into());
    let preimage =
        HashIdPreimage::ContractIdFromSourceAccount(HashIdPreimageSourceAccountContractId {
            network_id,
            source_account: AccountId(PublicKey::PublicKeyTypeEd25519(
                key.public.to_bytes().into(),
            )),
            salt: Uint256(salt),
        });
    let preimage_xdr = preimage.to_xdr()?;
    let contract_id = Sha256::digest(preimage_xdr);

    let op = Operation {
        source_account: None,
        body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
            function: HostFunction::CreateContract(CreateContractArgs {
                contract_id: ContractId::SourceAccount(Uint256(salt)),
                source: ScContractCode::WasmRef(hash.clone()),
            }),
            footprint: LedgerFootprint {
                read_only: vec![ContractCode(LedgerKeyContractCode { hash })].try_into()?,
                read_write: vec![ContractData(LedgerKeyContractData {
                    contract_id: Hash(contract_id.into()),
                    key: ScVal::Static(ScStatic::LedgerKeyContractCode),
                })]
                .try_into()?,
            },
        }),
    };
    let tx = Transaction {
        source_account: MuxedAccount::Ed25519(Uint256(key.public.to_bytes())),
        fee,
        seq_num: SequenceNumber(sequence),
        cond: Preconditions::None,
        memo: Memo::None,
        operations: vec![op].try_into()?,
        ext: TransactionExt::V0,
    };

    let envelope = utils::sign_transaction(key, &tx, network_passphrase)?;

    Ok((envelope, Hash(contract_id.into())))
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
