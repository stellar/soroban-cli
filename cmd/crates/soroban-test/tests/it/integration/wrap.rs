use soroban_cli::CommandParser;
use soroban_cli::{
    commands::{
        config::{self},
        lab::token::wrap,
    },
    utils::contract_id_hash_from_asset,
};
use soroban_test::TestEnv;

use super::util::{network_passphrase, ROOT_ACCOUNT_RPC_TEST};

#[tokio::test]
#[ignore]
async fn xfer_and_burn() {
    let sandbox = &TestEnv::default();
    let (address, id) = &deploy().await;
    assert!(authorize(sandbox, id, address).await);
    assert_eq!(
        "\"9223372036854775807\"",
        balance(sandbox, id, address).await,
    );

    println!(
        "{}",
        sandbox
            .invoke(&[
                "--id",
                &id,
                "--",
                "xfer",
                "--from",
                address,
                "--to",
                ROOT_ACCOUNT_RPC_TEST,
                "--amount=100"
            ])
            .await
            .unwrap()
    );

    assert_eq!(
        "\"9223372036854775707\"",
        balance(sandbox, id, address).await,
    );

    println!(
        "{}",
        sandbox
            .invoke(&["--id", id, "--", "burn", "--id", address, "--amount=100"])
            .await
            .unwrap()
    );

    assert_eq!(
        "\"9223372036854775607\"",
        balance(sandbox, id, address).await,
    );
}

pub async fn deploy() -> (String, String) {
    let address = config::identity::address::Cmd::parse("--hd-path=0")
        .unwrap()
        .public_key()
        .unwrap();
    let asset = format!("native:{address}");
    wrap_cmd(&asset).run().await.unwrap();
    let asset = soroban_cli::utils::parsing::parse_asset(&asset).unwrap();
    let hash = contract_id_hash_from_asset(&asset, &network_passphrase().unwrap()).unwrap();
    let id = stellar_strkey::Contract(hash.0).to_string();
    assert_eq!(
        "CAMTHSPKXZJIRTUXQP5QWJIFH3XIDMKLFAWVQOFOXPTKAW5GKV37ZC4N",
        id
    );
    (address.to_string(), id)
}

fn wrap_cmd(asset: &str) -> wrap::Cmd {
    wrap::Cmd::parse_arg_vec(&[&format!("--asset={asset}")]).unwrap()
}

async fn authorize(sandbox: &TestEnv, id: &str, address: &str) -> bool {
    sandbox
        .invoke(&["--id", id, "--", "authorized", "--id", address])
        .await
        .unwrap()
        == "true"
}

async fn balance(sandbox: &TestEnv, id: &str, address: &str) -> String {
    sandbox
        .invoke(&["--id", id, "--", "balance", "--id", address])
        .await
        .unwrap()
}
