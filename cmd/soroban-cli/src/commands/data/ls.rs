use clap::command;

use crate::commands::config::data::{self, Action};

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
        data::list_actions()?
            .iter()
            .map(|(id, action, uri)| {
                Ok(format!(
                    "{} {} {uri}\n",
                    to_datatime(id),
                    action_type(action)
                ))
            })
            .collect()
    }

    pub fn ls_l(&self) -> Result<Vec<String>, Error> {
        todo!()
    }
}

fn to_datatime(id: &ulid::Ulid) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp_millis(id.timestamp_ms().try_into().unwrap()).unwrap()
}

fn action_type(a: &Action) -> String {
    match a {
        Action::Simulation(_) => "Simulation",
        Action::Transaction(_) => "Transaction",
    }
    .to_string()
}
