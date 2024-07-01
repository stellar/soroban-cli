use predicates::boolean::PredicateBooleanExt;
use soroban_cli::commands::{
    config::{locator, secret},
    contract::{self, fetch},
    txn_result::TxnResult,
};
use soroban_rpc::GetLatestLedgerResponse;
use soroban_test::{AssertExt, TestEnv, LOCAL_NETWORK_PASSPHRASE};

use crate::integration::util::extend_contract;

use super::util::{deploy_hello, extend, HELLO_WORLD};

#[allow(clippy::too_many_lines)]
#[tokio::test]
async fn invoke_view_with_non_existent_source_account() {
    let sandbox = &TestEnv::new();
    let id = deploy_hello(sandbox).await;
    let world = "world";
    let mut cmd = hello_world_cmd(&id, world);
    cmd.config.source_account = String::new();
    cmd.is_view = true;
    let res = sandbox.run_cmd_with(cmd, "test").await.unwrap();
    assert_eq!(res, TxnResult::Res(format!(r#"["Hello",{world:?}]"#)));
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn invoke() {
    let sandbox = &TestEnv::new();
    let c = soroban_rpc::Client::new(&sandbox.rpc_url).unwrap();
    let GetLatestLedgerResponse { sequence, .. } = c.get_latest_ledger().await.unwrap();
    sandbox
        .new_assert_cmd("keys")
        .arg("fund")
        .arg("test")
        .assert()
        .stderr(predicates::str::contains("Account already exists"));
    sandbox
        .new_assert_cmd("keys")
        .arg("fund")
        .arg("test")
        .arg("--hd-path=1")
        .assert()
        .success();
    let addr = sandbox
        .new_assert_cmd("keys")
        .arg("address")
        .arg("test")
        .assert()
        .stdout_as_str();
    let addr_1 = sandbox
        .new_assert_cmd("keys")
        .arg("address")
        .arg("test")
        .arg("--hd-path=1")
        .assert()
        .stdout_as_str();
    println!("Addrs {addr}, {addr_1}");

    let secret_key = sandbox
        .new_assert_cmd("keys")
        .arg("show")
        .arg("test")
        .assert()
        .stdout_as_str();
    let secret_key_1 = sandbox
        .new_assert_cmd("keys")
        .arg("show")
        .arg("test")
        .arg("--hd-path=1")
        .assert()
        .stdout_as_str();
    let dir = sandbox.dir();
    let seed_phrase = std::fs::read_to_string(dir.join(".soroban/identity/test.toml")).unwrap();
    let s = toml::from_str::<secret::Secret>(&seed_phrase).unwrap();
    let secret::Secret::SeedPhrase { seed_phrase } = s else {
        panic!("Expected seed phrase")
    };
    let id = &deploy_hello(sandbox).await;
    extend_contract(sandbox, id).await;
    let uid = sandbox
        .new_assert_cmd("cache")
        .arg("actionlog")
        .arg("ls")
        .assert()
        .stdout_as_str();
    ulid::Ulid::from_string(&uid).expect("invalid ulid");
    // Note that all functions tested here have no state
    invoke_hello_world(sandbox, id);

    sandbox
        .new_assert_cmd("events")
        .arg("--start-ledger")
        .arg(&sequence.to_string())
        .arg("--id")
        .arg(id)
        .assert()
        .stdout(predicates::str::contains(id).not())
        .success();
    invoke_hello_world_with_lib(sandbox, id).await;
    let config_locator = locator::Args {
        global: false,
        config_dir: Some(dir.to_path_buf()),
    };
    config_locator
        .write_identity(
            "testone",
            &secret::Secret::SecretKey {
                secret_key: secret_key_1.clone(),
            },
        )
        .unwrap();
    let sk_from_file = std::fs::read_to_string(dir.join(".soroban/identity/testone.toml")).unwrap();

    assert_eq!(sk_from_file, format!("secret_key = \"{secret_key_1}\"\n"));
    let secret_key_1_readin = sandbox
        .new_assert_cmd("keys")
        .arg("show")
        .arg("testone")
        .assert()
        .stdout_as_str();
    assert_eq!(secret_key_1, secret_key_1_readin);
    // list all files recursively from dir including in hidden folders
    for entry in walkdir::WalkDir::new(dir) {
        println!("{}", entry.unwrap().path().display());
    }
    invoke_auth(sandbox, id, &addr);
    invoke_auth_with_identity(sandbox, id, "test", &addr);
    invoke_auth_with_identity(sandbox, id, "testone", &addr_1);
    invoke_auth_with_different_test_account_fail(sandbox, id, &addr_1).await;
    // invoke_auth_with_different_test_account(sandbox, id);
    contract_data_read_failure(sandbox, id);
    invoke_with_seed(sandbox, id, &seed_phrase).await;
    invoke_with_sk(sandbox, id, &secret_key).await;
    // This does add an identity to local config
    invoke_with_id(sandbox, id).await;
    handles_kebab_case(sandbox, id).await;
    fetch(sandbox, id).await;
    invoke_prng_u64_in_range_test(sandbox, id).await;
}

fn invoke_hello_world(sandbox: &TestEnv, id: &str) {
    sandbox
        .new_assert_cmd("contract")
        .arg("invoke")
        .arg("--is-view")
        .arg("--id")
        .arg(id)
        .arg("--")
        .arg("hello")
        .arg("--world=world")
        .assert()
        .stdout("[\"Hello\",\"world\"]\n")
        .success();
}

fn hello_world_cmd(id: &str, arg: &str) -> contract::invoke::Cmd {
    contract::invoke::Cmd {
        contract_id: id.to_string(),
        slop: vec!["hello".into(), format!("--world={arg}").into()],
        ..Default::default()
    }
}

async fn invoke_hello_world_with_lib(e: &TestEnv, id: &str) {
    let cmd = hello_world_cmd(id, "world");
    let res = e.run_cmd_with(cmd, "test").await.unwrap();
    assert_eq!(res, TxnResult::Res(r#"["Hello","world"]"#.to_string()));
}

fn invoke_auth(sandbox: &TestEnv, id: &str, addr: &str) {
    sandbox
        .new_assert_cmd("contract")
        .arg("invoke")
        .arg("--id")
        .arg(id)
        .arg("--")
        .arg("auth")
        .arg("--addr=test")
        .arg("--world=world")
        .assert()
        .stdout(format!("\"{addr}\"\n"))
        .success();

    // Invoke it again without providing the contract, to exercise the deployment
    sandbox
        .new_assert_cmd("contract")
        .arg("invoke")
        .arg("--id")
        .arg(id)
        .arg("--")
        .arg("auth")
        .arg("--addr=test")
        .arg("--world=world")
        .assert()
        .stdout(format!("\"{addr}\"\n"))
        .success();
}

fn invoke_auth_with_identity(sandbox: &TestEnv, id: &str, key: &str, addr: &str) {
    sandbox
        .new_assert_cmd("contract")
        .arg("invoke")
        .arg("--source")
        .arg(key)
        .arg("--id")
        .arg(id)
        .arg("--")
        .arg("auth")
        .arg("--addr")
        .arg(key)
        .arg("--world=world")
        .assert()
        .stdout(format!("\"{addr}\"\n"))
        .success();
}

async fn invoke_auth_with_different_test_account_fail(sandbox: &TestEnv, id: &str, addr: &str) {
    let res = sandbox
        .invoke_with_test(&[
            "--hd-path=0",
            "--id",
            id,
            "--",
            "auth",
            &format!("--addr={addr}"),
            "--world=world",
        ])
        .await;
    let e = res.unwrap_err();
    assert!(
        matches!(e, contract::invoke::Error::Config(_)),
        "Expected config error got {e:?}"
    );
}

fn contract_data_read_failure(sandbox: &TestEnv, id: &str) {
    sandbox
        .new_assert_cmd("contract")
        .arg("read")
        .arg("--id")
        .arg(id)
        .arg("--key=COUNTER")
        .arg("--durability=persistent")
        .assert()
        .failure()
        .stderr(
            "error: no matching contract data entries were found for the specified contract id\n",
        );
}

fn contract_data_read_hash(sandbox: &TestEnv, hash: &str) {
    sandbox
        .new_assert_cmd("contract")
        .arg("read")
        .arg("--wasm-hash")
        .arg(hash)
        .arg("--durability=persistent")
        .assert()
        .success()
        .stdout(predicates::str::starts_with("\"\"\""));
}

#[tokio::test]
async fn contract_data_read() {
    const KEY: &str = "COUNTER";
    let sandbox = &TestEnv::new();
    let id = &deploy_hello(sandbox).await;
    let res = sandbox
        .invoke_with_test(&["--id", id, "--", "inc"])
        .await
        .unwrap();
    assert_eq!(res.trim(), "1");
    extend(sandbox, id, Some(KEY)).await;

    sandbox
        .new_assert_cmd("contract")
        .arg("read")
        .arg("--id")
        .arg(id)
        .arg("--key")
        .arg(KEY)
        .arg("--durability=persistent")
        .assert()
        .success()
        .stdout(predicates::str::starts_with("COUNTER,1"));

    sandbox
        .new_assert_cmd("contract")
        .arg("invoke")
        .arg("--id")
        .arg(id)
        .arg("--")
        .arg("inc")
        .assert()
        .success();

    sandbox
        .new_assert_cmd("contract")
        .arg("read")
        .arg("--id")
        .arg(id)
        .arg("--key")
        .arg(KEY)
        .arg("--durability=persistent")
        .assert()
        .success()
        .stdout(predicates::str::starts_with("COUNTER,2"));
    let hello_world_hash = HELLO_WORLD.hash().expect("Failed to hash HELLO_WORLD");
    let hello_world_hash_hex = hello_world_hash.to_string();
    contract_data_read_hash(sandbox, &hello_world_hash_hex);
}

#[tokio::test]
#[ignore]
async fn half_max_instructions() {
    let sandbox = TestEnv::new();
    let wasm = HELLO_WORLD;
    sandbox
        .new_assert_cmd("contract")
        .arg("deploy")
        .arg("--fee")
        .arg("1000000")
        .arg("--instructions")
        .arg(&(u32::MAX / 2).to_string())
        .arg("--wasm")
        .arg(wasm.path())
        .arg("--ignore-checks")
        .assert()
        .stderr("")
        .stdout_as_str();
}

async fn invoke_with_seed(sandbox: &TestEnv, id: &str, seed_phrase: &str) {
    invoke_with_source(sandbox, seed_phrase, id).await;
}

async fn invoke_with_sk(sandbox: &TestEnv, id: &str, sk: &str) {
    invoke_with_source(sandbox, sk, id).await;
}

async fn invoke_with_id(sandbox: &TestEnv, id: &str) {
    invoke_with_source(sandbox, "test", id).await;
}

async fn invoke_with_source(sandbox: &TestEnv, source: &str, id: &str) {
    let cmd = sandbox
        .invoke_with(&["--id", id, "--", "hello", "--world=world"], source)
        .await
        .unwrap();
    assert_eq!(cmd, "[\"Hello\",\"world\"]");
}

async fn handles_kebab_case(e: &TestEnv, id: &str) {
    assert!(e
        .invoke_with_test(&["--id", id, "--", "multi-word-cmd", "--contract-owner=world",])
        .await
        .is_ok());
}

async fn fetch(sandbox: &TestEnv, id: &str) {
    let f = sandbox.dir().join("contract.wasm");
    let cmd = sandbox.cmd_arr::<fetch::Cmd>(&[
        "--rpc-url",
        &sandbox.rpc_url,
        "--network-passphrase",
        LOCAL_NETWORK_PASSPHRASE,
        "--id",
        id,
        "--out-file",
        f.to_str().unwrap(),
    ]);
    cmd.run().await.unwrap();
    assert!(f.exists());
}

async fn invoke_prng_u64_in_range_test(sandbox: &TestEnv, id: &str) {
    assert!(sandbox
        .invoke_with_test(&[
            "--id",
            id,
            "--",
            "prng_u64_in_range",
            "--low=0",
            "--high=100",
        ])
        .await
        .is_ok());
}
