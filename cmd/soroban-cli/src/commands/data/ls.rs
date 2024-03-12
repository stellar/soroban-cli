use clap::command;

use crate::commands::config::data;
use super::super::config::locator;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] locator::Error),
    #[error(transparent)]
    Data(#[from] data::Error),
}

#[derive(Debug, clap::Parser, Clone)]
#[group(skip)]
pub struct Cmd {
    #[command(flatten)]
    pub config_locator: locator::Args,

    #[arg(long, short = 'l')]
    pub long: bool,
}

impl Cmd {
    pub fn run(&self) -> Result<(), Error> {
        let res = if self.long { self.ls_l() } else { self.ls() }?.join("\n");
        println!("{res}");
        Ok(())
    }

    pub fn ls(&self) -> Result<Vec<String>, Error> {
        Ok(data::list_actions()?
            .iter()
            .map(ToString::to_string)
            .collect())
    }

    pub fn ls_l(&self) -> Result<Vec<String>, Error> {
        Ok(data::list_actions()?
            .iter()
            .map(ToString::to_string)
            .collect())
    }
}
