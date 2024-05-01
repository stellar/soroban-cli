use std::str::FromStr;

use async_trait::async_trait;
use clap::{command, error::ErrorKind, CommandFactory, FromArgMatches, Parser};

pub mod completion;

pub mod contract;
pub mod events;
pub mod global;
pub mod keys;
pub mod lab;
pub mod network;
pub mod plugin;
pub mod version;

pub const HEADING_RPC: &str = "Options (RPC)";
const ABOUT: &str = "Build, deploy, & interact with contracts; set identities to sign with; configure networks; generate keys; and more.

Intro: https://soroban.stellar.org/docs
CLI Reference: https://github.com/stellar/soroban-cli/tree/main/docs/soroban-cli-full-docs.md";

// long_about is shown when someone uses `--help`; short help when using `-h`
const LONG_ABOUT: &str = "

The easiest way to get started is to generate a new identity:

    soroban config identity generate alice

You can use identities with the `--source` flag in other commands later.

Commands that relate to smart contract interactions are organized under the `contract` subcommand. List them:

    soroban contract --help

A Soroban contract has its interface schema types embedded in the binary that gets deployed on-chain, making it possible to dynamically generate a custom CLI for each. `soroban contract invoke` makes use of this:

    soroban contract invoke --id CCR6QKTWZQYW6YUJ7UP7XXZRLWQPFRV6SWBLQS4ZQOSAF4BOUD77OTE2 --source alice --network testnet -- \
                            --help

Anything after the `--` double dash (the \"slop\") is parsed as arguments to the contract-specific CLI, generated on-the-fly from the embedded schema. For the hello world example, with a function called `hello` that takes one string argument `to`, here's how you invoke it:

    soroban contract invoke --id CCR6QKTWZQYW6YUJ7UP7XXZRLWQPFRV6SWBLQS4ZQOSAF4BOUD77OTE2 --source alice --network testnet -- \
                            hello --to world

Full CLI reference: https://github.com/stellar/soroban-tools/tree/main/docs/soroban-cli-full-docs.md";

#[derive(Parser, Debug)]
#[command(
    name = "soroban",
    about = ABOUT,
    version = version::long(),
    long_about = ABOUT.to_string() + LONG_ABOUT,
    disable_help_subcommand = true,
)]
pub struct Root {
    #[clap(flatten)]
    pub global_args: global::Args,

    #[command(subcommand)]
    pub cmd: Cmd,
}

impl Root {
    pub fn new() -> Result<Self, Error> {
        Self::try_parse().map_err(|e| {
            if std::env::args().any(|s| s == "--list") {
                let plugins = plugin::list().unwrap_or_default();
                if plugins.is_empty() {
                    println!("No Plugins installed. E.g. soroban-hello");
                } else {
                    println!("Installed Plugins:\n    {}", plugins.join("\n    "));
                }
                std::process::exit(0);
            }
            match e.kind() {
                ErrorKind::InvalidSubcommand => match plugin::run() {
                    Ok(()) => Error::Clap(e),
                    Err(e) => Error::Plugin(e),
                },
                _ => Error::Clap(e),
            }
        })
    }

    pub fn from_arg_matches<I, T>(itr: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        Self::from_arg_matches_mut(&mut Self::command().get_matches_from(itr))
    }
    pub async fn run(&mut self) -> Result<(), Error> {
        match &mut self.cmd {
            Cmd::Completion(completion) => completion.run(),
            Cmd::Contract(contract) => contract.run(&self.global_args).await?,
            Cmd::Events(events) => events.run().await?,
            Cmd::Lab(lab) => lab.run()?,
            Cmd::Network(network) => network.run().await?,
            Cmd::Version(version) => version.run(),
            Cmd::Keys(id) => id.run().await?,
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
    /// Print shell completion code for the specified shell.
    #[command(long_about = completion::LONG_ABOUT)]
    Completion(completion::Cmd),
    /// Tools for smart contract developers
    #[command(subcommand)]
    Contract(contract::Cmd),
    /// Watch the network for contract events
    Events(events::Cmd),
    /// Create and manage identities including keys and addresses
    #[command(subcommand)]
    Keys(keys::Cmd),
    /// Experiment with early features and expert tools
    #[command(subcommand)]
    Lab(lab::Cmd),
    /// Start and configure networks
    #[command(subcommand)]
    Network(network::Cmd),
    /// Print version information
    Version(version::Cmd),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // TODO: stop using Debug for displaying errors
    #[error(transparent)]
    Contract(#[from] contract::Error),
    #[error(transparent)]
    Events(#[from] events::Error),
    #[error(transparent)]
    Keys(#[from] keys::Error),
    #[error(transparent)]
    Lab(#[from] lab::Error),
    #[error(transparent)]
    Clap(#[from] clap::error::Error),
    #[error(transparent)]
    Plugin(#[from] plugin::Error),
    #[error(transparent)]
    Network(#[from] network::Error),
}

#[async_trait]
pub trait NetworkRunnable {
    type Error;
    type Result;

    async fn run_against_rpc_server(
        &self,
        global_args: Option<&global::Args>,
        config: Option<&config::Args>,
    ) -> Result<Self::Result, Self::Error>;
}
