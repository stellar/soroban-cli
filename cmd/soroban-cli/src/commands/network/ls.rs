use clap::command;

use config::locator;
use config::locator::Location;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] locator::Error),
}

#[derive(Debug, clap::Parser, Clone)]
#[group(skip)]
pub struct Cmd {
    #[command(flatten)]
    pub config_locator: locator::Args,
    /// Get more info about the networks
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
        Ok(self.config_locator.list_networks()?)
    }

    pub fn ls_l(&self) -> Result<Vec<String>, Error> {
        Ok(self
            .config_locator
            .list_networks_long()?
            .iter()
            .filter_map(|(name, network, location)| {
                (!self.config_locator.global || matches!(location, Location::Global(_)))
                    .then(|| Some(format!("{location}\nName: {name}\n{network:#?}\n")))?
            })
            .collect())
    }
}
