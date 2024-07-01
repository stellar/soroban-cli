use clap::command;

use crate::commands::config::locator::KeyName;

use super::super::config::{locator, secret};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Secret(#[from] secret::Error),

    #[error(transparent)]
    Config(#[from] locator::Error),
}

#[derive(Debug, clap::Parser, Clone)]
#[group(skip)]
pub struct Cmd {
    /// Name of identity
    pub name: KeyName,

    #[command(flatten)]
    pub secrets: secret::Args,

    #[command(flatten)]
    pub config_locator: locator::Args,
}

impl Cmd {
    pub fn run(&self) -> Result<(), Error> {
        Ok(self
            .config_locator
            .write_identity(&self.name, &self.secrets.kind()?)?)
    }
}
