use crate::util::{temp_ledger_file, test_wasm, Sandbox, SorobanCommand};

#[test]
fn invoke_token() {
    let ledger = temp_ledger_file();
    Sandbox::new_cmd()
        .arg("token")
        .arg("create")
        .arg("--ledger-file")
        .arg(&ledger)
        .arg("--name=tok")
        .arg("--symbol=tok")
        .assert()
        .success()
        .stdout("7794c4a02357bd9063499148e709bde44aa9e643d3fa20fde202f6e84a671e1b\n");

    Sandbox::new_cmd()
        .arg("invoke")
        .arg("--ledger-file")
        .arg(ledger)
        .arg("--id=7794c4a02357bd9063499148e709bde44aa9e643d3fa20fde202f6e84a671e1b")
        .arg("--fn=decimals")
        .assert()
        .success()
        .stdout("7\n");
}

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
        .stdout("86270dcca8dd4e7131c89dcc61223f096d7a1fa4a1d90c39dd6542b562369ecc\n");

    Sandbox::new_cmd()
        .arg("deploy")
        .arg("--ledger-file")
        .arg(&ledger)
        .arg("--wasm-hash=86270dcca8dd4e7131c89dcc61223f096d7a1fa4a1d90c39dd6542b562369ecc")
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
