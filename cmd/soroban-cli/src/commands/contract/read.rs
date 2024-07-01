use std::{
    fmt::Debug,
    io::{self, stdout},
};

use clap::{command, Parser, ValueEnum};
use soroban_env_host::{
    xdr::{
        ContractDataEntry, Error as XdrError, LedgerEntryData, LedgerKey, LedgerKeyContractData,
        Limits, ScVal, WriteXdr,
    },
    HostError,
};
use soroban_sdk::xdr::{ContractCodeEntry, LedgerKeyContractCode, ScBytes};

use crate::{
    commands::{
        config::{self, locator},
        global, NetworkRunnable,
    },
    key,
    rpc::{self, Client, FullLedgerEntries, FullLedgerEntry},
};

#[derive(Parser, Debug, Clone)]
#[group(skip)]
pub struct Cmd {
    /// Type of output to generate
    #[arg(long, value_enum, default_value("string"))]
    pub output: Output,
    #[command(flatten)]
    pub key: key::Args,
    #[command(flatten)]
    config: config::Args,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, ValueEnum)]
pub enum Output {
    /// String
    String,
    /// Json
    Json,
    /// XDR
    Xdr,
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
    #[error("cannot parse contract ID {contract_id}: {error}")]
    CannotParseContractId {
        contract_id: String,
        error: stellar_strkey::DecodeError,
    },
    #[error("cannot print result {result:?}: {error}")]
    CannotPrintResult {
        result: ScVal,
        error: soroban_spec_tools::Error,
    },
    #[error("cannot print result {result:?}: {error}")]
    CannotPrintJsonResult {
        result: ScVal,
        error: serde_json::Error,
    },
    #[error("cannot print as csv: {error}")]
    CannotPrintAsCsv { error: csv::Error },
    #[error("cannot print: {error}")]
    CannotPrintFlush { error: io::Error },
    #[error(transparent)]
    Config(#[from] config::Error),
    #[error("either `--key` or `--key-xdr` are required when querying a network")]
    KeyIsRequired,
    #[error(transparent)]
    Rpc(#[from] rpc::Error),
    #[error(transparent)]
    Xdr(#[from] XdrError),
    #[error(transparent)]
    // TODO: the Display impl of host errors is pretty user-unfriendly
    //       (it just calls Debug). I think we can do better than that
    Host(#[from] HostError),
    #[error("no matching contract data entries were found for the specified contract id")]
    NoContractDataEntryFoundForContractID,
    #[error(transparent)]
    Key(#[from] key::Error),
    #[error("Only contract code, data, and code keys are allowed")]
    OnlyDataAllowed,
    #[error(transparent)]
    Locator(#[from] locator::Error),
}

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        let entries = self.run_against_rpc_server(None, None).await?;
        self.output_entries(&entries)
    }

    fn output_entries(&self, entries: &FullLedgerEntries) -> Result<(), Error> {
        if entries.entries.is_empty() {
            return Err(Error::NoContractDataEntryFoundForContractID);
        }
        tracing::trace!("{entries:#?}");
        let mut out = csv::Writer::from_writer(stdout());
        for FullLedgerEntry {
            key,
            val,
            live_until_ledger_seq,
            last_modified_ledger,
        } in &entries.entries
        {
            let (key, val) = match (key, val) {
                (
                    LedgerKey::ContractData(LedgerKeyContractData { key, .. }),
                    LedgerEntryData::ContractData(ContractDataEntry { val, .. }),
                ) => (key.clone(), val.clone()),
                (
                    LedgerKey::ContractCode(LedgerKeyContractCode { hash }),
                    LedgerEntryData::ContractCode(ContractCodeEntry { code, .. }),
                ) => {
                    let key = ScVal::Bytes(ScBytes(hash.0.to_vec().try_into().unwrap()));
                    let val = ScVal::Bytes(ScBytes(code.clone()));
                    (key, val)
                }
                _ => return Err(Error::OnlyDataAllowed),
            };
            let output = match self.output {
                Output::String => [
                    soroban_spec_tools::to_string(&key).map_err(|e| Error::CannotPrintResult {
                        result: key.clone(),
                        error: e,
                    })?,
                    soroban_spec_tools::to_string(&val).map_err(|e| Error::CannotPrintResult {
                        result: val.clone(),
                        error: e,
                    })?,
                    last_modified_ledger.to_string(),
                    live_until_ledger_seq.to_string(),
                ],
                Output::Json => [
                    serde_json::to_string_pretty(&key).map_err(|error| {
                        Error::CannotPrintJsonResult {
                            result: key.clone(),
                            error,
                        }
                    })?,
                    serde_json::to_string_pretty(&val).map_err(|error| {
                        Error::CannotPrintJsonResult {
                            result: val.clone(),
                            error,
                        }
                    })?,
                    serde_json::to_string_pretty(&last_modified_ledger).map_err(|error| {
                        Error::CannotPrintJsonResult {
                            result: val.clone(),
                            error,
                        }
                    })?,
                    serde_json::to_string_pretty(&live_until_ledger_seq).map_err(|error| {
                        Error::CannotPrintJsonResult {
                            result: val.clone(),
                            error,
                        }
                    })?,
                ],
                Output::Xdr => [
                    key.to_xdr_base64(Limits::none())?,
                    val.to_xdr_base64(Limits::none())?,
                    last_modified_ledger.to_xdr_base64(Limits::none())?,
                    live_until_ledger_seq.to_xdr_base64(Limits::none())?,
                ],
            };
            out.write_record(output)
                .map_err(|e| Error::CannotPrintAsCsv { error: e })?;
        }
        out.flush()
            .map_err(|e| Error::CannotPrintFlush { error: e })?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl NetworkRunnable for Cmd {
    type Error = Error;
    type Result = FullLedgerEntries;

    async fn run_against_rpc_server(
        &self,
        _: Option<&global::Args>,
        config: Option<&config::Args>,
    ) -> Result<FullLedgerEntries, Error> {
        let config = config.unwrap_or(&self.config);
        let network = config.get_network()?;
        tracing::trace!(?network);
        let client = Client::new(&network.rpc_url)?;
        let contract = config.locator.resolve_contract_id(
            self.key.contract_id.as_ref().unwrap(),
            &network.network_passphrase,
        )?;
        let keys = self.key.parse_keys(contract)?;
        Ok(client.get_full_ledger_entries(&keys).await?)
    }
}
