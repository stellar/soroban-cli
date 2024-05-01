use config::network::passphrase;
use soroban_cli::utils::contract_id_hash_from_asset;
use soroban_test::{AssertExt, TestEnv};

#[tokio::test]
#[ignore]
async fn burn() {
    let sandbox = &TestEnv::new();
    let network_passphrase = passphrase::LOCAL;
    let address = sandbox
        .new_assert_cmd("keys")
        .arg("address")
        .arg("test")
        .assert()
        .stdout_as_str();
    let asset = format!("native:{address}");
    sandbox
        .new_assert_cmd("contract")
        .arg("asset")
        .arg("deploy")
        .arg("--source=test")
        .arg("--asset")
        .arg(&asset)
        .assert()
        .success();
    // wrap_cmd(&asset).run().await.unwrap();
    let asset = soroban_cli::utils::parsing::parse_asset(&asset).unwrap();
    let hash = contract_id_hash_from_asset(&asset, network_passphrase).unwrap();
    let id = stellar_strkey::Contract(hash.0).to_string();
    println!("{id}, {address}");
    sandbox
        .new_assert_cmd("contract")
        .args([
            "invoke",
            "--id",
            &id,
            "--",
            "balance",
            "--id",
            &address.to_string(),
        ])
        .assert()
        .stdout("\"9223372036854775807\"\n");
    sandbox
        .new_assert_cmd("contract")
        .arg("invoke")
        .args([
            "--id",
            &id,
            "--",
            "authorized",
            "--id",
            &address.to_string(),
        ])
        .assert()
        .stdout("true\n");
    sandbox
        .new_assert_cmd("contract")
        .args([
            "invoke",
            "--id",
            &id,
            "--",
            "balance",
            "--id",
            &address.to_string(),
        ])
        .assert()
        .stdout("\"9223372036854775807\"\n");
    sandbox
        .new_assert_cmd("contract")
        .arg("invoke")
        .env("RUST_LOGS", "trace")
        .args([
            "--source=test",
            "--id",
            &id,
            "--",
            "burn",
            "--from",
            "test",
            "--amount=100",
        ])
        .assert()
        .stdout("")
        .stderr("");

    println!("hi");
    sandbox
        .new_assert_cmd("contract")
        .args([
            "invoke",
            "--id",
            &id,
            "--",
            "balance",
            "--id",
            &address.to_string(),
        ])
        .assert()
        .stdout("\"9223372036854775707\"\n");
}
