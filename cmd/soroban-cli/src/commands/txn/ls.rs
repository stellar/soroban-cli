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
    pub fn run(&self) -> Result<(), Error> {
        let res = if self.long { self.ls_l() } else { self.ls() }?.join("\n");
        println!("{res}");
        Ok(())
    }

    pub fn ls(&self) -> Result<Vec<String>, Error> {
       todo!()
    }

    pub fn ls_l(&self) -> Result<Vec<String>, Error> {
        todo!()
    }
}
