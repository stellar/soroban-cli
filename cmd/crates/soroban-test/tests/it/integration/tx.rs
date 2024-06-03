use soroban_cli::commands::tx;
use soroban_sdk::xdr::{
    Limits, ReadXdr, Transaction, TransactionEnvelope, TransactionV1Envelope, VecM, WriteXdr,
};
use soroban_test::{AssertExt, TestEnv};

use crate::integration::util::{deploy_contract, DeployKind, HELLO_WORLD};

#[tokio::test]
async fn txn_simulate() {
    let sandbox = &TestEnv::new();
    let xdr_base64_build_only = deploy_contract(sandbox, HELLO_WORLD, DeployKind::BuildOnly).await;
    let xdr_base64_sim_only = deploy_contract(sandbox, HELLO_WORLD, DeployKind::SimOnly).await;
    println!("{xdr_base64_build_only}");
    let cmd = tx::simulate::Cmd::default();
    let tx_env =
        TransactionEnvelope::from_xdr_base64(&xdr_base64_build_only, Limits::none()).unwrap();
    let TransactionEnvelope::Tx(TransactionV1Envelope { tx, .. }) = &tx_env else {
        panic!("Only transaction v1 is supported")
    };
    let assembled = cmd.simulate(tx, &sandbox.client()).await.unwrap();
    let assembled_str = sandbox
        .new_assert_cmd("tx")
        .arg("simulate")
        .write_stdin(xdr_base64_build_only.as_bytes())
        .assert()
        .success()
        .stdout_as_str();
    println!("{assembled_str}");
    assert_eq!(xdr_base64_sim_only, assembled_str);
    let txn_env = TransactionEnvelope::Tx(TransactionV1Envelope {
        tx: assembled.transaction().clone(),
        signatures: VecM::default(),
    });

    assert_eq!(
        txn_env.to_xdr_base64(Limits::none()).unwrap(),
        assembled_str
    );
}

#[tokio::test]
async fn txn_send() {
    let sandbox = &TestEnv::new();
    sandbox
        .new_assert_cmd("contract")
        .arg("install")
        .args(["--wasm", HELLO_WORLD.path().as_os_str().to_str().unwrap()])
        .assert()
        .success();

    let xdr_base64 = deploy_contract(sandbox, HELLO_WORLD, DeployKind::SimOnly).await;
    println!("{xdr_base64}");
    let tx_env = TransactionEnvelope::from_xdr_base64(&xdr_base64, Limits::none()).unwrap();
    let tx_env = sign(sandbox, tx_env);

    println!(
        "Transaction to send:\n{}",
        tx_env.to_xdr_base64(Limits::none()).unwrap()
    );

    let assembled_str = sandbox
        .new_assert_cmd("tx")
        .arg("send")
        .arg("--source=test")
        .write_stdin(tx_env.to_xdr_base64(Limits::none()).unwrap())
        .assert()
        .success()
        .stdout_as_str();
    println!("Transaction sent: {assembled_str}");
}

fn sign(sandbox: &TestEnv, tx_env: TransactionEnvelope) -> TransactionEnvelope {
    TransactionEnvelope::from_xdr_base64(
        sandbox
            .new_assert_cmd("tx")
            .arg("sign")
            .arg("--signer=file")
            .arg("--source=test")
            .arg("-y")
            .write_stdin(tx_env.to_xdr_base64(Limits::none()).unwrap().as_bytes())
            .assert()
            .success()
            .stdout_as_str(),
        Limits::none(),
    )
    .unwrap()
}