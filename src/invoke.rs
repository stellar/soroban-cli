use std::{fmt::Debug, fs, io, io::Cursor, rc::Rc};

use clap::Parser;
use stellar_contract_env_host::{
    budget::CostType,
    storage::Storage,
    xdr::{
        Error as XdrError, HostFunction, ReadXdr, ScHostStorageErrorCode, ScObject, ScSpecEntry,
        ScSpecFunctionV0, ScStatus, ScVal,
    },
    Host, HostError, Vm,
};

use hex::{FromHex, FromHexError};

use crate::snapshot;
use crate::strval::{self, StrValError};
use crate::utils;

#[derive(Parser, Debug)]
pub struct Cmd {
    /// Name of function to invoke
    #[clap(long = "fn")]
    function: String,
    /// File to read and write ledger
    #[clap(long, parse(from_os_str), default_value("ledger.json"))]
    snapshot_file: std::path::PathBuf,
    /// Output the cost of the invocation to stderr
    #[clap(long = "cost")]
    cost: bool,
    #[clap(long, parse(from_os_str))]
    file: Option<std::path::PathBuf>,
    #[clap(long = "id")]
    contract_id: String,
    /// Argument to pass to the contract function
    #[clap(long = "arg", value_name = "arg", multiple = true)]
    args: Vec<String>,
    /// Argument to pass to the contract function (base64-encoded xdr)
    #[clap(
        long = "arg-xdr",
        value_name = "arg-xdr",
        multiple = true,
        conflicts_with = "args"
    )]
    args_xdr: Vec<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io")]
    Io(#[from] io::Error),
    #[error("strval")]
    StrVal(#[from] StrValError),
    #[error("xdr")]
    Xdr(#[from] XdrError),
    #[error("host")]
    Host(#[from] HostError),
    #[error("snapshot")]
    Snapshot(#[from] snapshot::Error),
    #[error("serde")]
    Serde(#[from] serde_json::Error),
    #[error("hex")]
    FromHex(#[from] FromHexError),
}

impl Cmd {
    pub fn run(&self) -> Result<(), Error> {
        let contract_id: [u8; 32] = FromHex::from_hex(&self.contract_id)?;

        // Initialize storage and host
        // TODO: allow option to separate input and output file
        let mut ledger_entries = snapshot::read(&self.snapshot_file)?;

        //If a file is specified, deploy the contract to storage
        if let Some(f) = &self.file {
            let contract = fs::read(f).unwrap();
            utils::add_contract_to_ledger_entries(&mut ledger_entries, contract_id, contract)?;
        }

        let snap = Rc::new(snapshot::Snap {
            ledger_entries: ledger_entries.clone(),
        });
        let mut storage = Storage::with_recording_footprint(snap);
        let contents = utils::get_contract_wasm_from_storage(&mut storage, contract_id)?;
        let mut h = Host::with_storage(storage);

        let vm = Vm::new(&h, [0; 32].into(), &contents).unwrap();
        let input_types = match Self::function_spec(&vm, &self.function) {
            Some(s) => s.input_types,
            None => {
                // TODO: Better error here
                return Err(Error::StrVal(StrValError::UnknownError));
            }
        };

        // re-assemble the args, to match the order given on the command line
        let args: Vec<ScVal> = if self.args_xdr.is_empty() {
            self.args
                .iter()
                .zip(input_types.iter())
                .map(|(a, t)| strval::from_string(a, t))
                .collect::<Result<Vec<ScVal>, StrValError>>()?
        } else {
            self.args_xdr
                .iter()
                .map(|a| match base64::decode(a) {
                    Err(_) => Err(StrValError::InvalidValue),
                    Ok(b) => ScVal::from_xdr(b).map_err(StrValError::Xdr),
                })
                .collect::<Result<Vec<ScVal>, StrValError>>()?
        };

        let mut complete_args = vec![
            ScVal::Object(Some(ScObject::Binary(contract_id.try_into()?))),
            ScVal::Symbol((&self.function).try_into()?),
        ];
        complete_args.extend_from_slice(args.as_slice());

        let res = h.invoke_function(HostFunction::Call, complete_args.try_into()?)?;
        println!("{}", strval::to_string(&res)?);

        if self.cost {
            h.get_budget(|b| {
                eprintln!("Cpu Insns: {}", b.cpu_insns.get_count());
                eprintln!("Mem Bytes: {}", b.mem_bytes.get_count());
                for cost_type in CostType::variants() {
                    eprintln!("Cost ({:?}): {}", cost_type, b.get_input(*cost_type));
                }
            });
        }

        let storage = h.recover_storage().map_err(|_h| {
            HostError::from(ScStatus::HostStorageError(
                ScHostStorageErrorCode::UnknownError,
            ))
        })?;

        snapshot::commit(ledger_entries, Some(&storage.map), &self.snapshot_file)?;
        Ok(())
    }

    fn function_spec(vm: &Rc<Vm>, name: &str) -> Option<ScSpecFunctionV0> {
        let spec = vm.custom_section("contractspecv0")?;
        let mut cursor = Cursor::new(spec);
        for spec_entry in ScSpecEntry::read_xdr_iter(&mut cursor).flatten() {
            if let ScSpecEntry::FunctionV0(f) = spec_entry {
                if let Ok(n) = f.name.to_string() {
                    if n == name {
                        return Some(f);
                    }
                }
            }
        }
        None
    }
}
