use soroban_test::TestEnv;

use super::util::{deploy_hello, TEST_CONTRACT_ID};

fn write_env_file(e: &TestEnv, contents: &str) {
    let env_file = e.dir().join(".env");
    std::fs::write(&env_file, contents).unwrap();
    assert_eq!(contents, std::fs::read_to_string(env_file).unwrap());
}

fn contract_id() -> String {
    format!("SOROBAN_CONTRACT_ID={TEST_CONTRACT_ID}")
}

#[test]
fn can_read_file() {
    TestEnv::with_default(|e| {
        deploy_hello(e);
        write_env_file(e, &contract_id());
        e.new_assert_cmd("contract")
            .arg("invoke")
            .arg("--")
            .arg("hello")
            .arg("--world=world")
            .assert()
            .stdout("[\"Hello\",\"world\"]\n")
            .success();
    });
}

#[test]
fn current_env_not_overwritten() {
    TestEnv::with_default(|e| {
        deploy_hello(e);
        write_env_file(e, &contract_id());

        e.new_assert_cmd("contract")
            .env("SOROBAN_CONTRACT_ID", "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFCT4")
            .arg("invoke")
            .arg("--")
            .arg("hello")
            .arg("--world=world")
            .assert()
            .stderr("error: Contract not found: CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFCT4\n");
    });
}

#[test]
fn cli_args_have_priority() {
    TestEnv::with_default(|e| {
        deploy_hello(e);
        write_env_file(e, &contract_id());
        e.new_assert_cmd("contract")
            .env(
                "SOROBAN_CONTRACT_ID",
                "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFCT4",
            )
            .arg("invoke")
            .arg("--id")
            .arg(TEST_CONTRACT_ID)
            .arg("--")
            .arg("hello")
            .arg("--world=world")
            .assert()
            .stdout("[\"Hello\",\"world\"]\n");
    });
}
