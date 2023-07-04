use clap::Parser;

pub mod add;
pub mod address;
pub mod generate;
pub mod ls;
pub mod rm;
pub mod show;

#[derive(Debug, Parser)]
pub enum Cmd {
    /// Add a new identity (keypair, ledger, macOS keychain)
    Add(add::Cmd),
    /// Given an identity return its address (public key)
    Address(address::Cmd),
    /// Generate a new identity with a seed phrase, currently 12 words
    Generate(generate::Cmd),
    /// List identities
    #[command(aliases = ["list"])]
    Ls(ls::Cmd),
    /// Remove an identity
    Rm(rm::Cmd),
    /// Given an identity return its private key
    Show(show::Cmd),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Add(#[from] add::Error),

    #[error(transparent)]
    Address(#[from] address::Error),

    #[error(transparent)]
    Generate(#[from] generate::Error),
    #[error(transparent)]
    Ls(#[from] ls::Error),

    #[error(transparent)]
    Rm(#[from] rm::Error),
    #[error(transparent)]
    Show(#[from] show::Error),
}

impl Cmd {
    pub fn run(&self) -> Result<(), Error> {
        match self {
            Cmd::Add(cmd) => cmd.run()?,
            Cmd::Address(cmd) => cmd.run()?,
            Cmd::Rm(cmd) => cmd.run()?,
            Cmd::Ls(cmd) => cmd.run()?,
            Cmd::Generate(cmd) => cmd.run()?,
            Cmd::Show(cmd) => cmd.run()?,
        };
        Ok(())
    }
}
