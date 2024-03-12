use clap::Parser;
pub mod ls;

#[derive(Debug, Parser)]
pub enum Cmd {
    /// List identities
    Ls(ls::Cmd),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Ls(#[from] ls::Error),
}

impl Cmd {
    pub fn run(&self) -> Result<(), Error> {
        match self {
            Cmd::Ls(cmd) => cmd.run()?,
        };
        Ok(())
    }
}
