use clap::{arg, command};

use crate::fee;

use super::super::config::{
    locator,
    secret::{self, Secret},
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Fee(#[from] locator::Error),
}

#[derive(Debug, clap::Parser, Clone)]
#[group(skip)]
pub struct Cmd {
    #[flatten]
    fee: fee::Args,
}

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        todo!()
    }
}
