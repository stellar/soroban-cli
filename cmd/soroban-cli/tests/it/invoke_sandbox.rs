use crate::util::{temp_ledger_file, test_wasm, Sandbox, SorobanCommand};

#[test]
fn source_account_exists() {
    Sandbox::new_cmd()
        .arg("invoke")
        .arg("--ledger-file")
        .arg(temp_ledger_file())
        .arg("--id=1")
        .arg("--wasm")
        .arg(test_wasm("test_invoker_account_exists"))
        .arg("--fn=invkexists")
        .assert()
        .success()
        .stdout("true\n");
}

#[test]
fn install_wasm_then_deploy_contract() {
    let ledger = temp_ledger_file();
    Sandbox::new_cmd()
        .arg("install")
        .arg("--ledger-file")
        .arg(&ledger)
        .arg("--wasm")
        .arg(test_wasm("test_hello_world"))
        .assert()
        .success()
        .stdout("1b459482ade00a540177e7e4c9417a4fe6ea50f79819bf53319e845e7c65e435\n");

    Sandbox::new_cmd()
        .arg("deploy")
        .arg("--ledger-file")
        .arg(&ledger)
        .arg("--wasm-hash=1b459482ade00a540177e7e4c9417a4fe6ea50f79819bf53319e845e7c65e435")
        .arg("--id=1")
        .assert()
        .success()
        .stdout("0000000000000000000000000000000000000000000000000000000000000001\n");
}

#[test]
fn deploy_contract_with_wasm_file() {
    Sandbox::new_cmd()
        .arg("deploy")
        .arg("--ledger-file")
        .arg(temp_ledger_file())
        .arg("--wasm")
        .arg(test_wasm("test_hello_world"))
        .arg("--id=1")
        .assert()
        .success()
        .stdout("0000000000000000000000000000000000000000000000000000000000000001\n");
}
