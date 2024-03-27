// https://github.com/zondax/ledger-rs

use ed25519_dalek::Signer;
use sha2::{Digest, Sha256};

use soroban_env_host::xdr::{
    self, AccountId, DecoratedSignature, Hash, HashIdPreimage, HashIdPreimageSorobanAuthorization,
    InvokeHostFunctionOp, Limits, Operation, OperationBody, PublicKey, ScAddress, ScMap, ScSymbol,
    ScVal, Signature, SignatureHint, SorobanAddressCredentials, SorobanAuthorizationEntry,
    SorobanAuthorizedFunction, SorobanCredentials, Transaction, TransactionEnvelope,
    TransactionSignaturePayload, TransactionSignaturePayloadTaggedTransaction,
    TransactionV1Envelope, Uint256, WriteXdr,
};

pub mod app;
use app::get_public_key;

mod emulator;
use emulator::run;

mod docker;

enum Error {}

#[cfg(test)]
mod test {
    use super::*;
    use hidapi::HidApi;
    use ledger_transport_hid::TransportNativeHID;
    use log::info;
    use once_cell::sync::Lazy;
    use serial_test::serial;

    fn init_logging() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn hidapi() -> &'static HidApi {
        static HIDAPI: Lazy<HidApi> = Lazy::new(|| HidApi::new().expect("unable to get HIDAPI"));

        &HIDAPI
    }

    #[test]
    #[serial]
    fn test_get_public_key() {
        let public_key = get_public_key(0);
        println!("{public_key:?}");
        assert!(public_key.is_ok());
    }

    // #[test]
    #[tokio::test]
    async fn test_my_emulator() {
        let r = emulator::run().await;
        assert!(r.is_ok());
    }
}
