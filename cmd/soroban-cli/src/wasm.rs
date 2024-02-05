use clap::arg;
use soroban_env_host::xdr::{self, LedgerKey, LedgerKeyContractCode};
use soroban_spec_tools::contract::{self, Spec};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::utils::{self};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("reading file {filepath}: {error}")]
    CannotReadContractFile {
        filepath: std::path::PathBuf,
        error: io::Error,
    },
    #[error("cannot parse wasm file {file}: {error}")]
    CannotParseWasm {
        file: std::path::PathBuf,
        error: wasmparser::BinaryReaderError,
    },
    #[error("xdr processing error: {0}")]
    Xdr(#[from] xdr::Error),

    #[error(transparent)]
    Parser(#[from] wasmparser::BinaryReaderError),
    #[error(transparent)]
    ContractSpec(#[from] contract::Error),
}

#[derive(Debug, clap::Args, Clone)]
#[group(skip)]
pub struct Args {
    /// Path to wasm binary
    #[arg(long)]
    pub wasm: PathBuf,
}

impl Args {
    /// # Errors
    /// May fail to read wasm file
    pub fn read(&self) -> Result<Vec<u8>, Error> {
        fs::read(&self.wasm).map_err(|e| Error::CannotReadContractFile {
            filepath: self.wasm.clone(),
            error: e,
        })
    }

    /// # Errors
    /// May fail to read wasm file
    pub fn len(&self) -> Result<u64, Error> {
        len(&self.wasm)
    }

    /// # Errors
    /// May fail to read wasm file
    pub fn is_empty(&self) -> Result<bool, Error> {
        self.len().map(|len| len == 0)
    }

    /// # Errors
    /// May fail to read wasm file or parse xdr section
    pub fn parse(&self) -> Result<Spec, Error> {
        let contents = self.read()?;
        Ok(Spec::new(&contents)?)
    }
}

impl From<&PathBuf> for Args {
    fn from(wasm: &PathBuf) -> Self {
        Self { wasm: wasm.clone() }
    }
}

impl TryInto<LedgerKey> for Args {
    type Error = Error;
    fn try_into(self) -> Result<LedgerKey, Self::Error> {
        Ok(LedgerKey::ContractCode(LedgerKeyContractCode {
            hash: utils::contract_hash(&self.read()?)?,
        }))
    }
}

/// # Errors
/// May fail to read wasm file
pub fn len(p: &Path) -> Result<u64, Error> {
    Ok(std::fs::metadata(p)
        .map_err(|e| Error::CannotReadContractFile {
            filepath: p.to_path_buf(),
            error: e,
        })?
        .len())
}
