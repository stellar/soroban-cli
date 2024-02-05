use std::path::Path;

use soroban_cli::commands::{
    config::{locator::KeyType, secret::Secret},
    contract,
};
use soroban_test::{TestEnv, Wasm};

pub const CUSTOM_TYPES: &Wasm = &Wasm::Custom("test-wasms", "test_custom_types");

#[derive(Clone)]
pub enum SecretKind {
    Seed,
    Key,
}

#[allow(clippy::needless_pass_by_value)]
pub fn add_key(dir: &Path, name: &str, kind: SecretKind, data: &str) {
    let secret = match kind {
        SecretKind::Seed => Secret::SeedPhrase {
            seed_phrase: data.to_string(),
        },
        SecretKind::Key => Secret::SecretKey {
            secret_key: data.to_string(),
        },
    };

    KeyType::Identity
        .write(name, &secret, &dir.join(".soroban"))
        .unwrap();
}

pub fn add_test_id(dir: &Path) -> String {
    let name = "test_id";
    add_key(
        dir,
        name,
        SecretKind::Key,
        "SBGWSG6BTNCKCOB3DIFBGCVMUPQFYPA2G4O34RMTB343OYPXU5DJDVMN",
    );
    name.to_owned()
}

pub const DEFAULT_SEED_PHRASE: &str =
    "coral light army gather adapt blossom school alcohol coral light army giggle";

#[allow(dead_code)]
pub async fn invoke_custom(
    sandbox: &TestEnv,
    id: &str,
    func: &str,
    arg: &str,
    wasm: &Path,
) -> Result<String, contract::invoke::Error> {
    let mut i: contract::invoke::Cmd = sandbox.cmd_arr(&[
        "--id",
        id,
        "--network",
        "futurenet",
        "--source",
        "default",
        "--",
        func,
        arg,
    ]);
    i.wasm = Some(wasm.to_path_buf());
    i.config.network.network = Some("futurenet".to_owned());
    i.invoke(&soroban_cli::commands::global::Args::default())
        .await
}

pub const DEFAULT_CONTRACT_ID: &str = "CDR6QKTWZQYW6YUJ7UP7XXZRLWQPFRV6SWBLQS4ZQOSAF4BOUD77OO5Z";
