//! **Soroban Test** - Test framework for invoking Soroban externally.
//!
//! Currently soroban provides a mock test environment for writing unit tets.
//!
//! However, it does not provide a way to run tests against a local sandbox or rpc endpoint.
//!
//! ## Overview
//!
//! - `TestEnv` is a test environment for running tests isolated from each other.
//! - `TestEnv::with_default` invokes a closure, which is passed a reference to a random `TestEnv`.
//! - `TestEnv::new_assert_cmd` creates an `assert_cmd::Command` for a given subcommand and sets the current
//!    directory to be the same as `TestEnv`.
//! - `TestEnv::cmd` is a generic function which parses a command from a string.
//!    Note, however, that it uses `shlex` to tokenize the string. This can cause issues
//!    for commands which contain strings with `"`s. For example, `{"hello": "world"}` becomes
//!    `{hello:world}`. For that reason it's recommended to use `TestEnv::cmd_arr` instead.
//! - `TestEnv::cmd_arr` is a generic function which takes an array of `&str` which is passed directly to clap.
//!    This is the preferred way since it ensures no string parsing footguns.
//! - `TestEnv::invoke` a convenience function for using the invoke command.
//!
#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::missing_panics_doc
)]
use std::{ffi::OsString, fmt::Display, path::Path};

use assert_cmd::{assert::Assert, Command};
use assert_fs::{fixture::FixtureError, prelude::PathChild, TempDir};
use fs_extra::dir::CopyOptions;

use soroban_cli::{
    commands::{config, contract, contract::invoke, global, keys},
    CommandParser, Pwd,
};

pub mod module;
mod wasm;
pub use wasm::Wasm;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    TempDir(#[from] FixtureError),

    #[error(transparent)]
    FsError(#[from] fs_extra::error::Error),

    #[error(transparent)]
    Invoke(#[from] invoke::Error),
}

/// A `TestEnv` is a contained process for a specific test, with its own ENV and
/// its own `TempDir` where it will save test-specific configuration.
pub struct TestEnv {
    pub temp_dir: TempDir,
}

impl Default for TestEnv {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl TestEnv {
    /// Execute a closure which is passed a reference to the `TestEnv`.
    /// `TempDir` implements the `Drop` trait ensuring that the temporary directory
    /// it creates is deleted when the `TestEnv` is dropped. This pattern ensures
    /// that the `TestEnv` cannot be dropped by the closure. For this reason, it's
    /// recommended to use `TempDir::with_default` instead of `new` or `default`.
    ///
    /// ```rust,no_run
    /// use soroban_test::TestEnv;
    /// TestEnv::with_default(|env| {
    ///     env.new_assert_cmd("contract").args(&["invoke", "--id", "1", "--", "hello", "--world=world"]).assert().success();
    /// });
    /// ```
    ///
    pub fn with_default<F: FnOnce(&TestEnv)>(f: F) {
        let test_env = TestEnv::default();
        f(&test_env);
    }
    pub fn new() -> Result<TestEnv, Error> {
        let this = TempDir::new().map(|temp_dir| TestEnv { temp_dir })?;
        std::env::set_var("XDG_CONFIG_HOME", this.temp_dir.as_os_str());
        this.new_assert_cmd("keys")
            .arg("generate")
            .arg("test")
            .arg("-d")
            .arg("--no-fund")
            .assert();
        std::env::set_var("SOROBAN_ACCOUNT", "test");
        Ok(this)
    }

    /// Create a new `assert_cmd::Command` for a given subcommand and set's the current directory
    /// to be the internal `temp_dir`.
    pub fn new_assert_cmd(&self, subcommand: &str) -> Command {
        let mut this = Command::cargo_bin("soroban").unwrap_or_else(|_| Command::new("soroban"));
        this.arg("-q");
        this.arg(subcommand);
        this.current_dir(&self.temp_dir);
        this
    }

    /// Parses a `&str` into a command and sets the pwd to be the same as the current `TestEnv`.
    /// Uses shlex under the hood and thus has issues parsing strings with embedded `"`s.
    /// Thus `TestEnv::cmd_arr` is recommended to instead.
    pub fn cmd<T: CommandParser<T>>(&self, args: &str) -> T {
        Self::cmd_with_pwd(args, self.dir())
    }

    /// Same as `TestEnv::cmd` but sets the pwd can be used instead of the current `TestEnv`.
    pub fn cmd_with_pwd<T: CommandParser<T>>(args: &str, pwd: &Path) -> T {
        let args = format!("--config-dir={pwd:?} {args}");
        T::parse(&args).unwrap()
    }

    /// Same as `TestEnv::cmd_arr` but sets the pwd can be used instead of the current `TestEnv`.
    pub fn cmd_arr_with_pwd<T: CommandParser<T>>(args: &[&str], pwd: &Path) -> T {
        let mut cmds = vec!["--config-dir", pwd.to_str().unwrap()];
        cmds.extend_from_slice(args);
        T::parse_arg_vec(&cmds).unwrap()
    }

    /// Parse a command using an array of `&str`s, which passes the strings directly to clap
    /// avoiding some issues `cmd` has with shlex. Use the current `TestEnv` pwd.
    pub fn cmd_arr<T: CommandParser<T>>(&self, args: &[&str]) -> T {
        Self::cmd_arr_with_pwd(args, self.dir())
    }

    /// A convenience method for using the invoke command.
    pub async fn invoke<I: AsRef<str>>(&self, command_str: &[I]) -> Result<String, invoke::Error> {
        let cmd = contract::invoke::Cmd::parse_arg_vec(
            &command_str
                .iter()
                .map(AsRef::as_ref)
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>(),
        )
        .unwrap();
        self.invoke_cmd(cmd).await
    }

    /// Invoke an already parsed invoke command
    pub async fn invoke_cmd(&self, mut cmd: invoke::Cmd) -> Result<String, invoke::Error> {
        cmd.set_pwd(self.dir());
        cmd.run_against_rpc_server(&global::Args {
            locator: config::locator::Args {
                global: false,
                config_dir: None,
            },
            filter_logs: Vec::default(),
            quiet: false,
            verbose: false,
            very_verbose: false,
            list: false,
        })
        .await
    }

    /// Reference to current directory of the `TestEnv`.
    pub fn dir(&self) -> &TempDir {
        &self.temp_dir
    }

    /// Returns the public key corresponding to the test keys's `hd_path`
    pub fn test_address(&self, hd_path: usize) -> String {
        self.cmd::<keys::address::Cmd>(&format!("--hd-path={hd_path}"))
            .public_key()
            .unwrap()
            .to_string()
    }

    /// Returns the private key corresponding to the test keys's `hd_path`
    pub fn test_show(&self, hd_path: usize) -> String {
        self.cmd::<keys::show::Cmd>(&format!("--hd-path={hd_path}"))
            .private_key()
            .unwrap()
            .to_string()
    }

    /// Copy the contents of the current `TestEnv` to another `TestEnv`
    pub fn fork(&self) -> Result<TestEnv, Error> {
        let this = TestEnv::new()?;
        self.save(&this.temp_dir)?;
        Ok(this)
    }

    /// Save the current state of the `TestEnv` to the given directory.
    pub fn save(&self, dst: &Path) -> Result<(), Error> {
        fs_extra::dir::copy(&self.temp_dir, dst, &CopyOptions::new())?;
        Ok(())
    }
}

pub fn temp_ledger_file() -> OsString {
    TempDir::new()
        .unwrap()
        .child("ledger.json")
        .as_os_str()
        .into()
}

pub trait AssertExt {
    fn stdout_as_str(&self) -> String;
}

impl AssertExt for Assert {
    fn stdout_as_str(&self) -> String {
        String::from_utf8(self.get_output().stdout.clone())
            .expect("failed to make str")
            .trim()
            .to_owned()
    }
}
pub trait CommandExt {
    fn json_arg<A>(&mut self, j: A) -> &mut Self
    where
        A: Display;
}

impl CommandExt for Command {
    fn json_arg<A>(&mut self, j: A) -> &mut Self
    where
        A: Display,
    {
        self.arg(OsString::from(j.to_string()))
    }
}
