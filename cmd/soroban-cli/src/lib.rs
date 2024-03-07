#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::missing_panics_doc
)]
pub(crate) use soroban_rpc as rpc;
pub(crate) use soroban_env_host::xdr;
use std::path::Path;

pub mod commands;
pub mod fee;
pub mod key;
pub mod log;
pub mod toid;
pub mod utils;
pub mod wasm;

pub use commands::Root;

pub fn parse_cmd<T>(s: &str) -> Result<T, clap::Error>
where
    T: clap::CommandFactory + clap::FromArgMatches,
{
    let input = shlex::split(s).ok_or_else(|| {
        clap::Error::raw(
            clap::error::ErrorKind::InvalidValue,
            format!("Invalid input for command:\n{s}"),
        )
    })?;
    T::from_arg_matches_mut(&mut T::command().no_binary_name(true).get_matches_from(input))
}

pub trait CommandParser<T> {
    fn parse(s: &str) -> Result<T, clap::Error>;

    fn parse_arg_vec(s: &[&str]) -> Result<T, clap::Error>;
}

impl<T> CommandParser<T> for T
where
    T: clap::CommandFactory + clap::FromArgMatches,
{
    fn parse(s: &str) -> Result<T, clap::Error> {
        parse_cmd(s)
    }

    fn parse_arg_vec(args: &[&str]) -> Result<T, clap::Error> {
        T::from_arg_matches_mut(&mut T::command().no_binary_name(true).get_matches_from(args))
    }
}

pub trait Pwd {
    fn set_pwd(&mut self, pwd: &Path);
}
