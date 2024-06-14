use std::str::FromStr;

use async_trait::async_trait;
use clap::{command, error::ErrorKind, CommandFactory, FromArgMatches, Parser};

pub mod cache;
pub mod completion;
pub mod config;
pub mod contract;
pub mod events;
pub mod global;
pub mod keys;
pub mod network;
pub mod plugin;
pub mod snapshot;
pub mod version;

pub mod txn_result;

pub const HEADING_NETWORK: &str = "Options (Network)";
const ABOUT: &str = "Build, deploy, & interact with contracts; set identities to sign with; configure networks; generate keys; and more.

Stellar Docs: https://developers.stellar.org
CLI Full Hep Docs: https://github.com/stellar/stellar-cli/tree/main/FULL_HELP_DOCS.md";

// long_about is shown when someone uses `--help`; short help when using `-h`
const LONG_ABOUT: &str = "

The easiest way to get started is to generate a new identity:

    stellar config identity generate alice

You can use identities with the `--source` flag in other commands later.

Commands that relate to smart contract interactions are organized under the `contract` subcommand. List them:

    stellar contract --help

A Soroban contract has its interface schema types embedded in the binary that gets deployed on-chain, making it possible to dynamically generate a custom CLI for each. The invoke subcommand makes use of this:

    stellar contract invoke --id CCR6QKTWZQYW6YUJ7UP7XXZRLWQPFRV6SWBLQS4ZQOSAF4BOUD77OTE2 --source alice --network testnet -- \
                            --help

Anything after the `--` double dash (the \"slop\") is parsed as arguments to the contract-specific CLI, generated on-the-fly from the embedded schema. For the hello world example, with a function called `hello` that takes one string argument `to`, here's how you invoke it:

    stellar contract invoke --id CCR6QKTWZQYW6YUJ7UP7XXZRLWQPFRV6SWBLQS4ZQOSAF4BOUD77OTE2 --source alice --network testnet -- \
                            hello --to world
";

#[derive(Parser, Debug)]
#[command(
    name = "stellar",
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
            Cmd::Xdr(xdr) => xdr.run()?,
            Cmd::Network(network) => network.run().await?,
            Cmd::Snapshot(snapshot) => snapshot.run().await?,
            Cmd::Version(version) => version.run(),
            Cmd::Keys(id) => id.run().await?,
            Cmd::Cache(data) => data.run()?,
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
    /// Decode and encode XDR
    Xdr(stellar_xdr::cli::Root),
    /// Start and configure networks
    #[command(subcommand)]
    Network(network::Cmd),
    /// Download a snapshot of a ledger from an archive.
    Snapshot(snapshot::Cmd),
    /// Print version information
    Version(version::Cmd),
    /// Cache for tranasctions and contract specs
    #[command(subcommand)]
    Cache(cache::Cmd),
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
    Xdr(#[from] stellar_xdr::cli::Error),
    #[error(transparent)]
    Clap(#[from] clap::error::Error),
    #[error(transparent)]
    Plugin(#[from] plugin::Error),
    #[error(transparent)]
    Network(#[from] network::Error),
    #[error(transparent)]
    Snapshot(#[from] snapshot::Error),
    #[error(transparent)]
    Cache(#[from] cache::Error),
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
