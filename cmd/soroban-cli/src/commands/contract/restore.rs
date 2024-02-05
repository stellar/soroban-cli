use std::{fmt::Debug, path::Path, str::FromStr};

use clap::{command, Parser};
use soroban_env_host::xdr::{
    Error as XdrError, ExtensionPoint, LedgerEntry, LedgerEntryChange, LedgerEntryData,
    LedgerFootprint, Memo, MuxedAccount, Operation, OperationBody, OperationMeta, Preconditions,
    RestoreFootprintOp, SequenceNumber, SorobanResources, SorobanTransactionData, Transaction,
    TransactionExt, TransactionMeta, TransactionMetaV3, TtlEntry, Uint256,
};
use stellar_strkey::DecodeError;

use crate::{
    commands::{
        config::{self, locator},
        contract::extend,
    },
    key,
    rpc::{self, Client},
    wasm, Pwd,
};

#[derive(Parser, Debug, Clone)]
#[group(skip)]
pub struct Cmd {
    #[command(flatten)]
    pub key: key::Args,
    /// Number of ledgers to extend the entry
    #[arg(long)]
    pub ledgers_to_extend: Option<u32>,
    /// Only print the new Time To Live ledger
    #[arg(long)]
    pub ttl_ledger_only: bool,
    #[command(flatten)]
    pub config: config::Args,
    #[command(flatten)]
    pub fee: crate::fee::Args,
}

impl FromStr for Cmd {
    type Err = clap::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use clap::{CommandFactory, FromArgMatches};
        Self::from_arg_matches_mut(&mut Self::command().get_matches_from(s.split_whitespace()))
    }
}

impl Pwd for Cmd {
    fn set_pwd(&mut self, pwd: &Path) {
        self.config.set_pwd(pwd);
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("parsing key {key}: {error}")]
    CannotParseKey {
        key: String,
        error: soroban_spec_tools::Error,
    },
    #[error("parsing XDR key {key}: {error}")]
    CannotParseXdrKey { key: String, error: XdrError },
    #[error("cannot parse contract ID {0}: {1}")]
    CannotParseContractId(String, DecodeError),
    #[error(transparent)]
    Config(#[from] config::Error),
    #[error("either `--key` or `--key-xdr` are required")]
    KeyIsRequired,
    #[error("xdr processing error: {0}")]
    Xdr(#[from] XdrError),
    #[error("Ledger entry not found")]
    LedgerEntryNotFound,
    #[error(transparent)]
    Locator(#[from] locator::Error),
    #[error("missing operation result")]
    MissingOperationResult,
    #[error(transparent)]
    Rpc(#[from] rpc::Error),
    #[error(transparent)]
    Wasm(#[from] wasm::Error),
    #[error(transparent)]
    Key(#[from] key::Error),
    #[error(transparent)]
    Extend(#[from] extend::Error),
}

impl Cmd {
    #[allow(clippy::too_many_lines)]
    pub async fn run(&self) -> Result<(), Error> {
        let expiration_ledger_seq = self.run_against_rpc_server().await?;

        if let Some(ledgers_to_extend) = self.ledgers_to_extend {
            extend::Cmd {
                key: self.key.clone(),
                ledgers_to_extend,
                config: self.config.clone(),
                fee: self.fee.clone(),
                ttl_ledger_only: false,
            }
            .run()
            .await?;
        } else {
            println!("New ttl ledger: {expiration_ledger_seq}");
        }

        Ok(())
    }

    pub async fn run_against_rpc_server(&self) -> Result<u32, Error> {
        let network = self.config.get_network()?;
        tracing::trace!(?network);
        let entry_keys = self.key.parse_keys()?;
        let network = &self.config.get_network()?;
        let client = Client::new(&network.rpc_url)?;
        let key = self.config.key_pair()?;

        // Get the account sequence number
        let public_strkey =
            stellar_strkey::ed25519::PublicKey(key.verifying_key().to_bytes()).to_string();
        let account_details = client.get_account(&public_strkey).await?;
        let sequence: i64 = account_details.seq_num.into();

        let tx = Transaction {
            source_account: MuxedAccount::Ed25519(Uint256(key.verifying_key().to_bytes())),
            fee: self.fee.fee,
            seq_num: SequenceNumber(sequence + 1),
            cond: Preconditions::None,
            memo: Memo::None,
            operations: vec![Operation {
                source_account: None,
                body: OperationBody::RestoreFootprint(RestoreFootprintOp {
                    ext: ExtensionPoint::V0,
                }),
            }]
            .try_into()?,
            ext: TransactionExt::V1(SorobanTransactionData {
                ext: ExtensionPoint::V0,
                resources: SorobanResources {
                    footprint: LedgerFootprint {
                        read_only: vec![].try_into()?,
                        read_write: entry_keys.try_into()?,
                    },
                    instructions: 0,
                    read_bytes: 0,
                    write_bytes: 0,
                },
                resource_fee: 0,
            }),
        };

        let res = client
            .prepare_and_send_transaction(&tx, &key, &[], &network.network_passphrase, None, None)
            .await?;

        let meta = res
            .result_meta
            .as_ref()
            .ok_or(Error::MissingOperationResult)?;
        let events = res.events()?;
        tracing::trace!(?meta);
        if !events.is_empty() {
            tracing::info!("Events:\n {events:#?}");
        }

        // The transaction from core will succeed regardless of whether it actually found &
        // restored the entry, so we have to inspect the result meta to tell if it worked or not.
        let TransactionMeta::V3(TransactionMetaV3 { operations, .. }) = meta else {
            return Err(Error::LedgerEntryNotFound);
        };
        tracing::debug!("Operations:\nlen:{}\n{operations:#?}", operations.len());

        // Simply check if there is exactly one entry here. We only support extending a single
        // entry via this command (which we should fix separately, but).
        if operations.len() == 0 {
            return Err(Error::LedgerEntryNotFound);
        }

        if operations.len() != 1 {
            tracing::warn!(
                "Unexpected number of operations: {}. Currently only handle one.",
                operations[0].changes.len()
            );
        }
        parse_operations(operations).ok_or(Error::MissingOperationResult)
    }
}

fn parse_operations(ops: &[OperationMeta]) -> Option<u32> {
    ops.first().and_then(|op| {
        op.changes.iter().find_map(|entry| match entry {
            LedgerEntryChange::Updated(LedgerEntry {
                data:
                    LedgerEntryData::Ttl(TtlEntry {
                        live_until_ledger_seq,
                        ..
                    }),
                ..
            })
            | LedgerEntryChange::Created(LedgerEntry {
                data:
                    LedgerEntryData::Ttl(TtlEntry {
                        live_until_ledger_seq,
                        ..
                    }),
                ..
            }) => Some(*live_until_ledger_seq),
            _ => None,
        })
    })
}
