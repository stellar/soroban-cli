use std::collections::HashMap;
use std::convert::{Infallible, TryInto};
use std::ffi::OsString;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fmt::Debug, fs, io};

use clap::{arg, command, value_parser, Parser};
use ed25519_dalek::SigningKey;
use heck::ToKebabCase;

use soroban_env_host::{
    xdr::{
        self, Error as XdrError, Hash, HostFunction, InvokeContractArgs, InvokeHostFunctionOp,
        LedgerEntryData, LedgerFootprint, Memo, MuxedAccount, Operation, OperationBody,
        Preconditions, ScAddress, ScSpecEntry, ScSpecFunctionV0, ScSpecTypeDef, ScVal, ScVec,
        SequenceNumber, SorobanAuthorizationEntry, SorobanResources, Transaction, TransactionExt,
        Uint256, VecM,
    },
    HostError,
};

use soroban_spec::read::FromWasmError;
use stellar_strkey::DecodeError;

use super::super::{
    config::{self, locator},
    events,
};
use crate::{commands::global, rpc, Pwd};
use soroban_spec_tools::{contract, Spec};

#[derive(Parser, Debug, Default, Clone)]
#[allow(clippy::struct_excessive_bools)]
#[group(skip)]
pub struct Cmd {
    /// Contract ID to invoke
    #[arg(long = "id", env = "SOROBAN_CONTRACT_ID")]
    pub contract_id: String,
    // For testing only
    #[arg(skip)]
    pub wasm: Option<std::path::PathBuf>,
    /// Output the cost execution to stderr
    #[arg(long = "cost")]
    pub cost: bool,
    /// Number of instructions to simulate
    #[arg(long)]
    pub instructions: Option<u32>,
    /// Function name as subcommand, then arguments for that function as `--arg-name value`
    #[arg(last = true, id = "CONTRACT_FN_AND_ARGS")]
    pub slop: Vec<OsString>,
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
    #[error("parsing argument {arg}: {error}")]
    CannotParseArg {
        arg: String,
        error: soroban_spec_tools::Error,
    },
    #[error("cannot add contract to ledger entries: {0}")]
    CannotAddContractToLedgerEntries(XdrError),
    #[error(transparent)]
    // TODO: the Display impl of host errors is pretty user-unfriendly
    //       (it just calls Debug). I think we can do better than that
    Host(#[from] HostError),
    #[error("reading file {0:?}: {1}")]
    CannotReadContractFile(PathBuf, io::Error),
    #[error("committing file {filepath}: {error}")]
    CannotCommitEventsFile {
        filepath: std::path::PathBuf,
        error: events::Error,
    },
    #[error("cannot parse contract ID {0}: {1}")]
    CannotParseContractId(String, DecodeError),
    #[error("function {0} was not found in the contract")]
    FunctionNotFoundInContractSpec(String),
    #[error("parsing contract spec: {0}")]
    CannotParseContractSpec(FromWasmError),
    // },
    #[error("function name {0} is too long")]
    FunctionNameTooLong(String),
    #[error("argument count ({current}) surpasses maximum allowed count ({maximum})")]
    MaxNumberOfArgumentsReached { current: usize, maximum: usize },
    #[error("cannot print result {result:?}: {error}")]
    CannotPrintResult {
        result: ScVal,
        error: soroban_spec_tools::Error,
    },
    #[error(transparent)]
    Xdr(#[from] XdrError),
    #[error("error parsing int: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    Rpc(#[from] rpc::Error),
    #[error("unexpected contract code data type: {0:?}")]
    UnexpectedContractCodeDataType(LedgerEntryData),
    #[error("missing operation result")]
    MissingOperationResult,
    #[error("missing result")]
    MissingResult,
    #[error(transparent)]
    StrVal(#[from] soroban_spec_tools::Error),
    #[error("error loading signing key: {0}")]
    SignatureError(#[from] ed25519_dalek::SignatureError),
    #[error(transparent)]
    Config(#[from] config::Error),
    #[error("unexpected ({length}) simulate transaction result length")]
    UnexpectedSimulateTransactionResultSize { length: usize },
    #[error("Missing argument {0}")]
    MissingArgument(String),
    #[error(transparent)]
    Clap(#[from] clap::Error),
    #[error(transparent)]
    Locator(#[from] locator::Error),
    #[error("Contract Error\n{0}: {1}")]
    ContractInvoke(String, String),
    #[error(transparent)]
    StrKey(#[from] stellar_strkey::DecodeError),
    #[error(transparent)]
    ContractSpec(#[from] contract::Error),
    #[error("")]
    MissingFileArg(PathBuf),
}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl Cmd {
    fn build_host_function_parameters(
        &self,
        contract_id: [u8; 32],
        spec_entries: &[ScSpecEntry],
    ) -> Result<(String, Spec, InvokeContractArgs, Vec<SigningKey>), Error> {
        let spec = Spec(Some(spec_entries.to_vec()));
        let mut cmd = clap::Command::new(self.contract_id.clone())
            .no_binary_name(true)
            .term_width(300)
            .max_term_width(300);

        for ScSpecFunctionV0 { name, .. } in spec.find_functions()? {
            cmd = cmd.subcommand(build_custom_cmd(&name.to_utf8_string_lossy(), &spec)?);
        }
        cmd.build();
        let long_help = cmd.render_long_help();
        let mut matches_ = cmd.get_matches_from(&self.slop);
        let Some((function, matches_)) = &matches_.remove_subcommand() else {
            println!("{long_help}");
            std::process::exit(1);
        };

        let func = spec.find_function(function)?;
        // create parsed_args in same order as the inputs to func
        let mut signers: Vec<SigningKey> = vec![];
        let parsed_args = func
            .inputs
            .iter()
            .map(|i| {
                let name = i.name.to_utf8_string()?;
                if let Some(mut val) = matches_.get_raw(&name) {
                    let mut s = val.next().unwrap().to_string_lossy().to_string();
                    if matches!(i.type_, ScSpecTypeDef::Address) {
                        let cmd = crate::commands::keys::address::Cmd {
                            name: s.clone(),
                            hd_path: Some(0),
                            locator: self.config.locator.clone(),
                        };
                        if let Ok(address) = cmd.public_key() {
                            s = address.to_string();
                        }
                        if let Ok(key) = cmd.private_key() {
                            signers.push(key);
                        }
                    }
                    spec.from_string(&s, &i.type_)
                        .map_err(|error| Error::CannotParseArg { arg: name, error })
                } else if matches!(i.type_, ScSpecTypeDef::Option(_)) {
                    Ok(ScVal::Void)
                } else if let Some(arg_path) =
                    matches_.get_one::<PathBuf>(&fmt_arg_file_name(&name))
                {
                    if matches!(i.type_, ScSpecTypeDef::Bytes | ScSpecTypeDef::BytesN(_)) {
                        Ok(ScVal::try_from(
                            &std::fs::read(arg_path)
                                .map_err(|_| Error::MissingFileArg(arg_path.clone()))?,
                        )
                        .map_err(|()| Error::CannotParseArg {
                            arg: name.clone(),
                            error: soroban_spec_tools::Error::Unknown,
                        })?)
                    } else {
                        let file_contents = std::fs::read_to_string(arg_path)
                            .map_err(|_| Error::MissingFileArg(arg_path.clone()))?;
                        tracing::debug!(
                            "file {arg_path:?}, has contents:\n{file_contents}\nAnd type {:#?}\n{}",
                            i.type_,
                            file_contents.len()
                        );
                        spec.from_string(&file_contents, &i.type_)
                            .map_err(|error| Error::CannotParseArg { arg: name, error })
                    }
                } else {
                    Err(Error::MissingArgument(name))
                }
            })
            .collect::<Result<Vec<_>, Error>>()?;

        let contract_address_arg = ScAddress::Contract(Hash(contract_id));
        let function_symbol_arg = function
            .try_into()
            .map_err(|()| Error::FunctionNameTooLong(function.clone()))?;

        let final_args =
            parsed_args
                .clone()
                .try_into()
                .map_err(|_| Error::MaxNumberOfArgumentsReached {
                    current: parsed_args.len(),
                    maximum: ScVec::default().max_len(),
                })?;

        let invoke_args = InvokeContractArgs {
            contract_address: contract_address_arg,
            function_name: function_symbol_arg,
            args: final_args,
        };

        Ok((function.clone(), spec, invoke_args, signers))
    }

    pub async fn run(&self, global_args: &global::Args) -> Result<(), Error> {
        let res = self.invoke(global_args).await?;
        println!("{res}");
        Ok(())
    }

    pub async fn invoke(&self, global_args: &global::Args) -> Result<String, Error> {
        self.run_against_rpc_server(global_args).await
    }

    pub async fn run_against_rpc_server(
        &self,
        global_args: &global::Args,
    ) -> Result<String, Error> {
        let network = self.config.get_network()?;
        tracing::trace!(?network);
        let contract_id = self.contract_id()?;
        let spec_entries = self.spec_entries()?;
        if let Some(spec_entries) = &spec_entries {
            // For testing wasm arg parsing
            let _ = self.build_host_function_parameters(contract_id, spec_entries)?;
        }
        let client = rpc::Client::new(&network.rpc_url)?;
        client
            .verify_network_passphrase(Some(&network.network_passphrase))
            .await?;
        let key = self.config.key_pair()?;

        // Get the account sequence number
        let public_strkey =
            stellar_strkey::ed25519::PublicKey(key.verifying_key().to_bytes()).to_string();
        let account_details = client.get_account(&public_strkey).await?;
        let sequence: i64 = account_details.seq_num.into();

        // Get the contract
        let spec_entries = client.get_remote_contract_spec(&contract_id).await?;

        // Get the ledger footprint
        let (function, spec, host_function_params, signers) =
            self.build_host_function_parameters(contract_id, &spec_entries)?;
        let tx = build_invoke_contract_tx(
            host_function_params.clone(),
            sequence + 1,
            self.fee.fee,
            &key,
        )?;
        let mut txn = client.create_assembled_transaction(&tx).await?;
        if let Some(instructions) = self.instructions {
            txn = txn.set_max_instructions(instructions);
        }
        let (return_value, events) = if txn.is_view() {
            (
                txn.sim_res().results()?[0].xdr.clone(),
                txn.sim_res().events()?,
            )
        } else {
            let res = client
                .send_assembled_transaction(
                    txn,
                    &key,
                    &signers,
                    &network.network_passphrase,
                    Some(log_events),
                    (global_args.verbose || global_args.very_verbose || self.cost)
                        .then_some(log_resources),
                )
                .await?;
            (res.return_value()?, res.contract_events()?)
        };

        crate::log::diagnostic_events(&events, tracing::Level::INFO);
        output_to_string(&spec, &return_value, &function)
    }

    pub fn read_wasm(&self) -> Result<Option<Vec<u8>>, Error> {
        Ok(if let Some(wasm) = self.wasm.as_ref() {
            Some(fs::read(wasm).map_err(|e| Error::CannotReadContractFile(wasm.clone(), e))?)
        } else {
            None
        })
    }

    pub fn spec_entries(&self) -> Result<Option<Vec<ScSpecEntry>>, Error> {
        self.read_wasm()?
            .map(|wasm| {
                soroban_spec::read::from_wasm(&wasm).map_err(Error::CannotParseContractSpec)
            })
            .transpose()
    }
}

impl Cmd {
    fn contract_id(&self) -> Result<[u8; 32], Error> {
        soroban_spec_tools::utils::contract_id_from_str(&self.contract_id)
            .map_err(|e| Error::CannotParseContractId(self.contract_id.clone(), e))
    }
}

fn log_events(
    footprint: &LedgerFootprint,
    auth: &[VecM<SorobanAuthorizationEntry>],
    events: &[xdr::DiagnosticEvent],
) {
    crate::log::auth(auth);
    crate::log::diagnostic_events(events, tracing::Level::TRACE);
    crate::log::footprint(footprint);
}

fn log_resources(resources: &SorobanResources) {
    crate::log::cost(resources);
}

pub fn output_to_string(spec: &Spec, res: &ScVal, function: &str) -> Result<String, Error> {
    let mut res_str = String::new();
    if let Some(output) = spec.find_function(function)?.outputs.first() {
        res_str = spec
            .xdr_to_json(res, output)
            .map_err(|e| Error::CannotPrintResult {
                result: res.clone(),
                error: e,
            })?
            .to_string();
    }
    Ok(res_str)
}

fn build_invoke_contract_tx(
    parameters: InvokeContractArgs,
    sequence: i64,
    fee: u32,
    key: &SigningKey,
) -> Result<Transaction, Error> {
    let op = Operation {
        source_account: None,
        body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
            host_function: HostFunction::InvokeContract(parameters),
            auth: VecM::default(),
        }),
    };
    Ok(Transaction {
        source_account: MuxedAccount::Ed25519(Uint256(key.verifying_key().to_bytes())),
        fee,
        seq_num: SequenceNumber(sequence),
        cond: Preconditions::None,
        memo: Memo::None,
        operations: vec![op].try_into()?,
        ext: TransactionExt::V0,
    })
}

fn build_custom_cmd(name: &str, spec: &Spec) -> Result<clap::Command, Error> {
    let func = spec
        .find_function(name)
        .map_err(|_| Error::FunctionNotFoundInContractSpec(name.to_string()))?;

    // Parse the function arguments
    let inputs_map = &func
        .inputs
        .iter()
        .map(|i| (i.name.to_utf8_string().unwrap(), i.type_.clone()))
        .collect::<HashMap<String, ScSpecTypeDef>>();
    let name: &'static str = Box::leak(name.to_string().into_boxed_str());
    let mut cmd = clap::Command::new(name)
        .no_binary_name(true)
        .term_width(300)
        .max_term_width(300);
    let kebab_name = name.to_kebab_case();
    if kebab_name != name {
        cmd = cmd.alias(kebab_name);
    }
    let doc: &'static str = Box::leak(func.doc.to_utf8_string_lossy().into_boxed_str());
    let long_doc: &'static str = Box::leak(arg_file_help(doc).into_boxed_str());

    cmd = cmd.about(Some(doc)).long_about(long_doc);
    for (name, type_) in inputs_map {
        let mut arg = clap::Arg::new(name);
        let file_arg_name = fmt_arg_file_name(name);
        let mut file_arg = clap::Arg::new(&file_arg_name);
        arg = arg
            .long(name)
            .alias(name.to_kebab_case())
            .num_args(1)
            .value_parser(clap::builder::NonEmptyStringValueParser::new())
            .long_help(spec.doc(name, type_)?);

        file_arg = file_arg
            .long(&file_arg_name)
            .alias(file_arg_name.to_kebab_case())
            .num_args(1)
            .hide(true)
            .value_parser(value_parser!(PathBuf))
            .conflicts_with(name);

        if let Some(value_name) = spec.arg_value_name(type_, 0) {
            let value_name: &'static str = Box::leak(value_name.into_boxed_str());
            arg = arg.value_name(value_name);
        }

        // Set up special-case arg rules
        arg = match type_ {
            xdr::ScSpecTypeDef::Bool => arg
                .num_args(0..1)
                .default_missing_value("true")
                .default_value("false")
                .num_args(0..=1),
            xdr::ScSpecTypeDef::Option(_val) => arg.required(false),
            xdr::ScSpecTypeDef::I256
            | xdr::ScSpecTypeDef::I128
            | xdr::ScSpecTypeDef::I64
            | xdr::ScSpecTypeDef::I32 => arg.allow_hyphen_values(true),
            _ => arg,
        };

        cmd = cmd.arg(arg);
        cmd = cmd.arg(file_arg);
    }
    Ok(cmd)
}

fn fmt_arg_file_name(name: &str) -> String {
    format!("{name}-file-path")
}

fn arg_file_help(docs: &str) -> String {
    format!(
        r#"{docs}
Usage Notes:
Each arg has a corresponding --<arg_name>-file-path which is a path to a file containing the corresponding JSON argument.
Note: The only types which aren't JSON are Bytes and Bytes which are raw bytes"#
    )
}
