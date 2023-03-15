use clap::{command, Parser};
use std::fmt::Debug;

use crate::wasm;

#[derive(Parser, Debug)]
pub struct Cmd {
    #[command(flatten)]
    wasm: wasm::Args,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Wasm(#[from] wasm::Error),
}

impl Cmd {
    pub fn run(&self) -> Result<(), Error> {
        println!("File: {}", self.wasm.wasm.to_string_lossy());
        print!("{:#?}", self.wasm.parse()?.spec);
        Ok(())
    }
}
