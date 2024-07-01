use std::collections::HashMap;
use std::convert::{Infallible, TryInto};
use std::ffi::OsString;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fmt::Debug, fs, io};

use clap::{arg, command, value_parser, Parser};

use heck::ToKebabCase;

use soroban_env_host::{
    xdr::{
        self, AccountEntry, AccountEntryExt, AccountId, Hash, HostFunction, InvokeContractArgs,
        InvokeHostFunctionOp, LedgerEntryData, Limits, Memo, MuxedAccount, Operation,
        OperationBody, Preconditions, PublicKey, ScAddress, ScSpecEntry, ScSpecFunctionV0,
        ScSpecTypeDef, ScVal, ScVec, SequenceNumber, String32, StringM, Thresholds, Transaction,
        TransactionExt, Uint256, VecM, WriteXdr,
    },
    HostError,
};

use soroban_sdk::xdr::ScSpecFunctionInputV0;
use soroban_spec::read::FromWasmError;

use super::super::{
    config::{self, locator},
    events,
};
use crate::commands::config::secret::StellarSigner;
use crate::commands::txn_result::{TxnEnvelopeResult, TxnResult};
use crate::commands::NetworkRunnable;
use crate::get_spec::{self, get_remote_contract_spec};
use crate::signer::{self, Stellar};
use crate::{
    commands::{config::data, global, network},
    rpc, Pwd,
};
use soroban_spec_tools::{contract, Spec};

#[derive(Parser, Debug, Default, Clone)]
#[allow(clippy::struct_excessive_bools)]
#[group(skip)]
pub struct Cmd {
    /// Contract ID to invoke
    #[arg(long = "id", env = "STELLAR_CONTRACT_ID")]
    pub contract_id: String,
    // For testing only
    #[arg(skip)]
    pub wasm: Option<std::path::PathBuf>,
    /// View the result simulating and do not sign and submit transaction
    #[arg(long, env = "STELLAR_INVOKE_VIEW")]
    pub is_view: bool,
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
    CannotAddContractToLedgerEntries(xdr::Error),
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
    #[error("function {0} was not found in the contract")]
    FunctionNotFoundInContractSpec(String),
    #[error("parsing contract spec: {0}")]
    CannotParseContractSpec(FromWasmError),
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
    Xdr(#[from] xdr::Error),
    #[error("error parsing int: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    Rpc(#[from] rpc::Error),
    #[error("unexpected contract code data type: {0:?}")]
    UnexpectedContractCodeDataType(LedgerEntryData),
    #[error("missing operation result")]
    MissingOperationResult,
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
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Data(#[from] data::Error),
    #[error(transparent)]
    Network(#[from] network::Error),
    #[error(transparent)]
    GetSpecError(#[from] get_spec::Error),
    #[error(transparent)]
    Signer(#[from] signer::Error),
}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl Cmd {
    fn is_view(&self) -> bool {
        self.is_view ||
            // TODO: Remove at next major release. Was added to retain backwards
            // compatibility when this env var used to be used for the --is-view
            // option.
            std::env::var("SYSTEM_TEST_VERBOSE_OUTPUT").as_deref() == Ok("true")
    }

    async fn build_host_function_parameters(
        &self,
        contract_id: [u8; 32],
        spec_entries: &[ScSpecEntry],
        config: &config::Args,
    ) -> Result<(String, Spec, InvokeContractArgs, Vec<StellarSigner>), Error> {
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
        let mut signers: Vec<StellarSigner> = vec![];
        let mut parsed_args: Vec<ScVal> = Vec::new();
        for i in func.inputs.iter() {
            let (val, signer) = self.parse_arg(i, matches_, config, &spec).await?;
            parsed_args.push(val);
            if let Some(signer) = signer {
                signers.push(signer);
            }
        }

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

    pub async fn parse_arg(
        &self,
        input: &ScSpecFunctionInputV0,
        matches_: &clap::ArgMatches,
        config: &config::Args,
        spec: &Spec,
    ) -> Result<(ScVal, Option<StellarSigner>), Error> {
        let mut signer: Option<StellarSigner> = None;
        let name = input.name.to_utf8_string()?;
        let sc_val = if let Some(mut val) = matches_.get_raw(&name) {
            let mut s = val.next().unwrap().to_string_lossy().to_string();
            if matches!(input.type_, ScSpecTypeDef::Address) {
                if let Ok(signer_) = config
                    .locator
                    .account(&s)
                    .and_then(|signer| Ok(signer.signer(config.hd_path, config.check)?))
                {
                    s = signer_.get_public_key().await?.to_string();
                    signer = Some(signer_);
                }
            }
            spec.from_string(&s, &input.type_)
                .map_err(|error| Error::CannotParseArg { arg: name, error })?
        } else if matches!(input.type_, ScSpecTypeDef::Option(_)) {
            ScVal::Void
        } else if let Some(arg_path) = matches_.get_one::<PathBuf>(&fmt_arg_file_name(&name)) {
            if matches!(input.type_, ScSpecTypeDef::Bytes | ScSpecTypeDef::BytesN(_)) {
                ScVal::try_from(
                    &std::fs::read(arg_path)
                        .map_err(|_| Error::MissingFileArg(arg_path.clone()))?,
                )
                .map_err(|()| Error::CannotParseArg {
                    arg: name.clone(),
                    error: soroban_spec_tools::Error::Unknown,
                })?
            } else {
                let file_contents = std::fs::read_to_string(arg_path)
                    .map_err(|_| Error::MissingFileArg(arg_path.clone()))?;
                tracing::debug!(
                    "file {arg_path:?}, has contents:\n{file_contents}\nAnd type {:#?}\n{}",
                    input.type_,
                    file_contents.len()
                );
                spec.from_string(&file_contents, &input.type_)
                    .map_err(|error| Error::CannotParseArg { arg: name, error })?
            }
        } else {
            return Err(Error::MissingArgument(name));
        };
        Ok((sc_val, signer))
    }

    pub async fn run(&self, global_args: &global::Args) -> Result<(), Error> {
        let res = self.invoke(global_args).await?.to_envelope();
        match res {
            TxnEnvelopeResult::TxnEnvelope(tx) => println!("{}", tx.to_xdr_base64(Limits::none())?),
            TxnEnvelopeResult::Res(output) => {
                println!("{output}");
            }
        }
        Ok(())
    }

    pub async fn invoke(&self, global_args: &global::Args) -> Result<TxnResult<String>, Error> {
        self.run_against_rpc_server(Some(global_args), None).await
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

#[async_trait::async_trait]
impl NetworkRunnable for Cmd {
    type Error = Error;
    type Result = TxnResult<String>;

    async fn run_against_rpc_server(
        &self,
        global_args: Option<&global::Args>,
        config: Option<&config::Args>,
    ) -> Result<TxnResult<String>, Error> {
        let config = config.unwrap_or(&self.config);
        let network = config.get_network()?;
        tracing::trace!(?network);
        let contract_id = self
            .config
            .locator
            .resolve_contract_id(&self.contract_id, &network.network_passphrase)?
            .0;
        let spec_entries = self.spec_entries()?;
        if let Some(spec_entries) = &spec_entries {
            // For testing wasm arg parsing
            let _ = self
                .build_host_function_parameters(contract_id, spec_entries, config)
                .await?;
        }
        let client = rpc::Client::new(&network.rpc_url)?;
        let account_details = if self.is_view {
            default_account_entry()
        } else {
            client
                .verify_network_passphrase(Some(&network.network_passphrase))
                .await?;
            let key = config.public_key().await?;

            // Get the account sequence number
            client.get_account(&key.to_string()).await?
        };
        let sequence: i64 = account_details.seq_num.into();
        let AccountId(PublicKey::PublicKeyTypeEd25519(account_id)) = account_details.account_id;

        let spec_entries = get_remote_contract_spec(
            &contract_id,
            &config.locator,
            &config.network,
            global_args,
            Some(config),
        )
        .await
        .map_err(Error::from)?;

        // Get the ledger footprint
        let (function, spec, host_function_params, signers) = self
            .build_host_function_parameters(contract_id, &spec_entries, config)
            .await?;
        let tx = build_invoke_contract_tx(
            host_function_params.clone(),
            sequence + 1,
            self.fee.fee,
            account_id,
        )?;
        if self.fee.build_only {
            return Ok(TxnResult::Txn(tx));
        }
        let txn = client.simulate_and_assemble_transaction(&tx).await?;
        let txn = self.fee.apply_to_assembled_txn(txn);
        if self.fee.sim_only {
            return Ok(TxnResult::Txn(txn.transaction().clone()));
        }
        let sim_res = txn.sim_response();
        if global_args.map_or(true, |a| !a.no_cache) {
            data::write(sim_res.clone().into(), &network.rpc_uri()?)?;
        }
        let (return_value, events) = if self.is_view() {
            // log_auth_cost_and_footprint(Some(&sim_res.transaction_data()?.resources));
            (sim_res.results()?[0].xdr.clone(), sim_res.events()?)
        } else {
            let global::Args { no_cache, .. } = global_args.cloned().unwrap_or_default();
            // Need to sign all auth entries
            let mut txn = txn.transaction().clone();
            // let auth = auth_entries(&txn);
            // crate::log::auth(&[auth]);
            for signer in &signers {
                if let Some(tx) = config.sign_soroban_authorizations(signer, &txn).await? {
                    txn = tx;
                }
            }
            // log_auth_cost_and_footprint(resources(&txn));
            let res = client
                .send_transaction_polling(&config.sign(txn).await?)
                .await?;
            if !no_cache {
                data::write(res.clone().try_into()?, &network.rpc_uri()?)?;
            }
            (res.return_value()?, res.contract_events()?)
        };

        crate::log::diagnostic_events(&events, tracing::Level::INFO);
        output_to_string(&spec, &return_value, &function)
    }
}

const DEFAULT_ACCOUNT_ID: AccountId = AccountId(PublicKey::PublicKeyTypeEd25519(Uint256([0; 32])));

// fn log_auth_cost_and_footprint(resources: Option<&SorobanResources>) {
//     if let Some(resources) = resources {
//         crate::log::footprint(&resources.footprint);
//         crate::log::cost(resources);
//     }
// }

// fn resources(tx: &Transaction) -> Option<&SorobanResources> {
//     let TransactionExt::V1(SorobanTransactionData { resources, .. }) = &tx.ext else {
//         return None;
//     };
//     Some(resources)
// }

// fn auth_entries(tx: &Transaction) -> VecM<SorobanAuthorizationEntry> {
//     tx.operations
//         .first()
//         .and_then(|op| match op.body {
//             OperationBody::InvokeHostFunction(ref body) => (matches!(
//                 body.auth.first().map(|x| &x.root_invocation.function),
//                 Some(&SorobanAuthorizedFunction::ContractFn(_))
//             ))
//             .then_some(body.auth.clone()),
//             _ => None,
//         })
//         .unwrap_or_default()
// }

fn default_account_entry() -> AccountEntry {
    AccountEntry {
        account_id: DEFAULT_ACCOUNT_ID,
        balance: 0,
        seq_num: SequenceNumber(0),
        num_sub_entries: 0,
        inflation_dest: None,
        flags: 0,
        home_domain: String32::from(unsafe { StringM::<32>::from_str("TEST").unwrap_unchecked() }),
        thresholds: Thresholds([0; 4]),
        signers: unsafe { [].try_into().unwrap_unchecked() },
        ext: AccountEntryExt::V0,
    }
}

pub fn output_to_string(
    spec: &Spec,
    res: &ScVal,
    function: &str,
) -> Result<TxnResult<String>, Error> {
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
    Ok(TxnResult::Res(res_str))
}

fn build_invoke_contract_tx(
    parameters: InvokeContractArgs,
    sequence: i64,
    fee: u32,
    source_account_id: Uint256,
) -> Result<Transaction, Error> {
    let op = Operation {
        source_account: None,
        body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
            host_function: HostFunction::InvokeContract(parameters),
            auth: VecM::default(),
        }),
    };
    Ok(Transaction {
        source_account: MuxedAccount::Ed25519(source_account_id),
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
            ScSpecTypeDef::Bool => arg
                .num_args(0..1)
                .default_missing_value("true")
                .default_value("false")
                .num_args(0..=1),
            ScSpecTypeDef::Option(_val) => arg.required(false),
            ScSpecTypeDef::I256 | ScSpecTypeDef::I128 | ScSpecTypeDef::I64 | ScSpecTypeDef::I32 => {
                arg.allow_hyphen_values(true)
            }
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
Note: The only types which aren't JSON are Bytes and BytesN, which are raw bytes"#
    )
}
