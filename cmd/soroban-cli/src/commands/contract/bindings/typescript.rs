use std::{ffi::OsString, fmt::Debug, path::PathBuf};

use clap::{command, Parser};
use soroban_spec_tools::contract as contract_spec;
use soroban_spec_typescript::{self as typescript, boilerplate::Project};

use crate::commands::{
    config::locator,
    contract::{self, fetch},
    network::{self, Network},
};
use crate::wasm;

#[derive(Parser, Debug, Clone)]
#[group(skip)]
pub struct Cmd {
    /// Path to optional wasm binary
    #[arg(long)]
    pub wasm: Option<std::path::PathBuf>,
    /// Where to place generated project
    #[arg(long)]
    output_dir: PathBuf,
    /// Whether to overwrite output directory if it already exists
    #[arg(long)]
    overwrite: bool,
    /// The contract ID/address on the network
    #[arg(long, visible_alias = "id")]
    contract_id: String,
    #[command(flatten)]
    locator: locator::Args,
    #[command(flatten)]
    network: network::Args,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed generate TS from file: {0}")]
    GenerateTSFromFile(typescript::GenerateFromFileError),
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("--output-dir cannot be a file: {0:?}")]
    IsFile(PathBuf),

    #[error("--output-dir already exists and you did not specify --overwrite: {0:?}")]
    OutputDirExists(PathBuf),

    #[error("--output-dir filepath not representable as utf-8: {0:?}")]
    NotUtf8(OsString),

    #[error(transparent)]
    Network(#[from] network::Error),

    #[error(transparent)]
    Locator(#[from] locator::Error),
    #[error(transparent)]
    Fetch(#[from] fetch::Error),
    #[error(transparent)]
    Spec(#[from] contract_spec::Error),
    #[error(transparent)]
    Wasm(#[from] wasm::Error),
    #[error("Failed to get file name from path: {0:?}")]
    FailedToGetFileName(PathBuf),
}

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        let spec = if let Some(wasm) = &self.wasm {
            let wasm: wasm::Args = wasm.into();
            wasm.parse()?.spec
        } else {
            let fetch = contract::fetch::Cmd {
                contract_id: self.contract_id.clone(),
                out_file: None,
                locator: self.locator.clone(),
                network: self.network.clone(),
            };
            let bytes = fetch.get_bytes().await?;
            contract_spec::Spec::new(&bytes)?.spec
        };
        if self.output_dir.is_file() {
            return Err(Error::IsFile(self.output_dir.clone()));
        }
        if self.output_dir.exists() {
            if self.overwrite {
                std::fs::remove_dir_all(&self.output_dir)?;
            } else {
                return Err(Error::OutputDirExists(self.output_dir.clone()));
            }
        }
        std::fs::create_dir_all(&self.output_dir)?;
        let p: Project = self.output_dir.clone().try_into()?;
        let Network {
            rpc_url,
            network_passphrase,
            ..
        } = self
            .network
            .get(&self.locator)
            .ok()
            .unwrap_or_else(Network::futurenet);
        let absolute_path = self.output_dir.canonicalize()?;
        let file_name = absolute_path
            .file_name()
            .ok_or_else(|| Error::FailedToGetFileName(absolute_path.clone()))?;
        let contract_name = &file_name
            .to_str()
            .ok_or_else(|| Error::NotUtf8(file_name.to_os_string()))?;
        p.init(
            contract_name,
            &self.contract_id,
            &rpc_url,
            &network_passphrase,
            &spec,
        )?;
        std::process::Command::new("npm")
            .arg("install")
            .current_dir(&self.output_dir)
            .spawn()?
            .wait()?;
        std::process::Command::new("npm")
            .arg("run")
            .arg("build")
            .current_dir(&self.output_dir)
            .spawn()?
            .wait()?;
        Ok(())
    }
}
