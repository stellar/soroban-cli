use std::array::TryFromSliceError;
use std::fmt::Debug;
use std::num::ParseIntError;

use cargo_metadata::semver::Op;
use clap::{command, Parser};
use soroban_env_host::xdr::{
    self, Error as XdrError, Hash, HostFunction, InvokeHostFunctionOp, Memo, MuxedAccount,
    Operation, OperationBody, Preconditions, ScMetaEntry, ScMetaV0, SequenceNumber, Transaction,
    TransactionExt, TransactionResult, TransactionResultResult, Uint256, VecM,
};

use super::restore;
use crate::commands::{global, NetworkRunnable};
use crate::rpc::{self, Client};
use crate::txn::{InvokeHostFunctionOpBuilder, OperationBuilder, TransactionBuilder};
use crate::{commands::config, utils, wasm};
use crate::{key, txn};

const CONTRACT_META_SDK_KEY: &str = "rssdkver";
const PUBLIC_NETWORK_PASSPHRASE: &str = "Public Global Stellar Network ; September 2015";

#[derive(Parser, Debug, Clone)]
#[group(skip)]
pub struct Cmd {
    #[command(flatten)]
    pub config: config::Args,
    #[command(flatten)]
    pub fee: crate::fee::Args,
    #[command(flatten)]
    pub wasm: wasm::Args,
    #[arg(long, short = 'i', default_value = "false")]
    /// Whether to ignore safety checks when deploying contracts
    pub ignore_checks: bool,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error parsing int: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("internal conversion error: {0}")]
    TryFromSliceError(#[from] TryFromSliceError),
    #[error("xdr processing error: {0}")]
    Xdr(#[from] XdrError),
    #[error("jsonrpc error: {0}")]
    JsonRpc(#[from] jsonrpsee_core::Error),
    #[error(transparent)]
    Rpc(#[from] rpc::Error),
    #[error(transparent)]
    TxnBuilder(#[from] txn::Error),
    #[error(transparent)]
    Config(#[from] config::Error),
    #[error(transparent)]
    Wasm(#[from] wasm::Error),
    #[error("unexpected ({length}) simulate transaction result length")]
    UnexpectedSimulateTransactionResultSize { length: usize },
    #[error(transparent)]
    Restore(#[from] restore::Error),
    #[error("cannot parse WASM file {wasm}: {error}")]
    CannotParseWasm {
        wasm: std::path::PathBuf,
        error: wasm::Error,
    },
    #[error("the deployed smart contract {wasm} was built with Soroban Rust SDK v{version}, a release candidate version not intended for use with the Stellar Public Network. To deploy anyway, use --ignore-checks")]
    ContractCompiledWithReleaseCandidateSdk {
        wasm: std::path::PathBuf,
        version: String,
    },
}

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        let res_str = hex::encode(self.run_against_rpc_server(None, None).await?);
        println!("{res_str}");
        Ok(())
    }
}

#[async_trait::async_trait]
impl NetworkRunnable for Cmd {
    type Error = Error;
    type Result = Hash;
    async fn run_against_rpc_server(
        &self,
        args: Option<&global::Args>,
        config: Option<&config::Args>,
    ) -> Result<Hash, Error> {
        let config = config.unwrap_or(&self.config);
        let contract = self.wasm.read()?;
        let network = config.get_network()?;
        let client = Client::new(&network.rpc_url)?;
        client
            .verify_network_passphrase(Some(&network.network_passphrase))
            .await?;
        let wasm_spec = &self.wasm.parse().map_err(|e| Error::CannotParseWasm {
            wasm: self.wasm.wasm.clone(),
            error: e,
        })?;
        // Check Rust SDK version if using the public network.
        if let Some(rs_sdk_ver) = get_contract_meta_sdk_version(wasm_spec) {
            if rs_sdk_ver.contains("rc")
                && !self.ignore_checks
                && network.network_passphrase == PUBLIC_NETWORK_PASSPHRASE
            {
                return Err(Error::ContractCompiledWithReleaseCandidateSdk {
                    wasm: self.wasm.wasm.clone(),
                    version: rs_sdk_ver,
                });
            } else if rs_sdk_ver.contains("rc")
                && network.network_passphrase == PUBLIC_NETWORK_PASSPHRASE
            {
                tracing::warn!("the deployed smart contract {path} was built with Soroban Rust SDK v{rs_sdk_ver}, a release candidate version not intended for use with the Stellar Public Network", path = self.wasm.wasm.display());
            }
        }
        let key = config.key_pair()?;

        // Get the account sequence number
        let public_strkey =
            stellar_strkey::ed25519::PublicKey(key.verifying_key().to_bytes()).to_string();
        let account_details = client.get_account(&public_strkey).await?;
        let sequence: i64 = account_details.seq_num.into();

        let (tx_without_preflight, hash) =
            build_install_contract_code_tx(&contract, sequence + 1, self.fee.fee, &key)?;

        let code_key =
            xdr::LedgerKey::ContractCode(xdr::LedgerKeyContractCode { hash: hash.clone() });
        let contract_data = client.get_ledger_entries(&[code_key]).await?;
        if !contract_data.entries.unwrap_or_default().is_empty() {
            return Ok(hash);
        }

        let txn = client
            .create_assembled_transaction(&tx_without_preflight)
            .await?;
        let txn = self.fee.apply_to_assembled_txn(txn);

        // Currently internal errors are not returned if the contract code is expired
        if let Some(TransactionResult {
            result: TransactionResultResult::TxInternalError,
            ..
        }) = client
            .send_assembled_transaction(txn, &key, &[], &network.network_passphrase, None, None)
            .await?
            .result
            .as_ref()
        {
            // Now just need to restore it and don't have to install again
            restore::Cmd {
                key: key::Args {
                    contract_id: None,
                    key: None,
                    key_xdr: None,
                    wasm: Some(self.wasm.wasm.clone()),
                    wasm_hash: None,
                    durability: super::Durability::Persistent,
                },
                config: config.clone(),
                fee: self.fee.clone(),
                ledgers_to_extend: None,
                ttl_ledger_only: true,
            }
            .run_against_rpc_server(args, None)
            .await?;
        }

        Ok(hash)
    }
}

fn get_contract_meta_sdk_version(wasm_spec: &soroban_spec_tools::contract::Spec) -> Option<String> {
    let rs_sdk_version_option = if let Some(_meta) = &wasm_spec.meta_base64 {
        wasm_spec.meta.iter().find(|entry| match entry {
            ScMetaEntry::ScMetaV0(ScMetaV0 { key, .. }) => {
                key.to_utf8_string_lossy().contains(CONTRACT_META_SDK_KEY)
            }
        })
    } else {
        None
    };
    if let Some(rs_sdk_version_entry) = &rs_sdk_version_option {
        match rs_sdk_version_entry {
            ScMetaEntry::ScMetaV0(ScMetaV0 { val, .. }) => {
                return Some(val.to_utf8_string_lossy());
            }
        }
    }
    None
}

pub(crate) fn build_install_contract_code_tx(
    source_code: &[u8],
    sequence: i64,
    fee: u32,
    key: &ed25519_dalek::SigningKey,
) -> Result<(Transaction, Hash), XdrError> {
    let source_account = stellar_strkey::Strkey::PublicKeyEd25519(
        stellar_strkey::ed25519::PublicKey(key.verifying_key().to_bytes().try_into()?),
    );
    let mut txn = TransactionBuilder::new(source_account.clone())?;
    let op = OperationBuilder::new()
        .set_source_account(&source_account)
        .set_host_function(HostFunction::UploadContractWasm(source_code.try_into()?))
        .build();
    
    let op = InvokeHostFunctionOpBuilder::upload(source_code)?.build()?
     let op_body =   OperationBuilder::new()
            .set_source_account(&txn.txn.source_account)
            .set_body(op)
            .build();
    

    let hash = utils::contract_hash(source_code)?;

    let op = Operation {
        source_account: Some(MuxedAccount::Ed25519(Uint256(
            key.verifying_key().to_bytes(),
        ))),
        body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
            host_function: HostFunction::UploadContractWasm(source_code.try_into()?),
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

    Ok((tx, hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_install_contract_code() {
        let result = build_install_contract_code_tx(
            b"foo",
            300,
            1,
            &utils::parse_secret_key("SBFGFF27Y64ZUGFAIG5AMJGQODZZKV2YQKAVUUN4HNE24XZXD2OEUVUP")
                .unwrap(),
        );

        assert!(result.is_ok());
    }
}
