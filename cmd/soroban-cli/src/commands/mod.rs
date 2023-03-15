use std::str::FromStr;

use clap::{command, CommandFactory, FromArgMatches, Parser};

pub mod completion;
pub mod config;
pub mod contract;
pub mod events;
pub mod lab;
pub mod version;

pub const HEADING_SANDBOX: &str = "Options (Sandbox)";
pub const HEADING_RPC: &str = "Options (RPC)";
#[derive(Parser, Debug)]
#[command(
    name = "soroban",
    version = version::short(),
    long_version = version::long(),
    about = "https://soroban.stellar.org",
    disable_help_subcommand = true,
)]
pub struct Root {
    #[command(subcommand)]
    pub cmd: Cmd,
}

impl Root {
    pub fn new() -> Result<Self, clap::Error> {
        let mut matches = Self::command().get_matches();
        Self::from_arg_matches_mut(&mut matches)
    }

    pub fn from_arg_matches<I, T>(itr: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        Self::from_arg_matches_mut(&mut Self::command().get_matches_from(itr))
    }
    pub async fn run(&self) -> Result<(), Error> {
        match &self.cmd {
            Cmd::Contract(contract) => contract.run().await?,
            Cmd::Events(events) => events.run().await?,
            Cmd::Lab(lab) => lab.run().await?,
            Cmd::Version(version) => version.run(),
            Cmd::Completion(completion) => completion.run(),
            Cmd::Config(config) => config.run()?,
        };
        Ok(())
    }
}

impl FromStr for Root {
    type Err = clap::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_arg_matches(s.split_whitespace())
    }
}

#[derive(Parser, Debug)]
pub enum Cmd {
    /// Tools for smart contract developers
    #[command(subcommand)]
    Contract(contract::Cmd),
    /// Read and update config
    #[command(subcommand)]
    Config(config::Cmd),
    /// Run a local webserver for web app development and testing
    Events(events::Cmd),
    /// Experiment with early features and expert tools
    #[command(subcommand)]
    Lab(lab::Cmd),
    /// Print version information
    Version(version::Cmd),
    /// Print shell completion code for the specified shell.
    #[command(long_about = completion::LONG_ABOUT)]
    Completion(completion::Cmd),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // TODO: stop using Debug for displaying errors
    #[error(transparent)]
    Contract(#[from] contract::Error),
    #[error(transparent)]
    Events(#[from] events::Error),

    #[error(transparent)]
    Lab(#[from] lab::Error),
    #[error(transparent)]
    Config(#[from] config::Error),
}
