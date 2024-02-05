use ed25519_dalek::Signer;
use sha2::{Digest, Sha256};
use stellar_strkey::ed25519::PrivateKey;

use soroban_env_host::xdr::{
    Asset, ContractIdPreimage, DecoratedSignature, Error as XdrError, Hash, HashIdPreimage,
    HashIdPreimageContractId, Limits, Signature, SignatureHint, Transaction, TransactionEnvelope,
    TransactionSignaturePayload, TransactionSignaturePayloadTaggedTransaction,
    TransactionV1Envelope, WriteXdr,
};

pub use soroban_spec_tools::contract as contract_spec;

/// # Errors
///
/// Might return an error
pub fn contract_hash(contract: &[u8]) -> Result<Hash, XdrError> {
    Ok(Hash(Sha256::digest(contract).into()))
}

/// # Errors
///
/// Might return an error
pub fn transaction_hash(tx: &Transaction, network_passphrase: &str) -> Result<[u8; 32], XdrError> {
    let signature_payload = TransactionSignaturePayload {
        network_id: Hash(Sha256::digest(network_passphrase).into()),
        tagged_transaction: TransactionSignaturePayloadTaggedTransaction::Tx(tx.clone()),
    };
    Ok(Sha256::digest(signature_payload.to_xdr(Limits::none())?).into())
}

/// # Errors
///
/// Might return an error
pub fn sign_transaction(
    key: &ed25519_dalek::SigningKey,
    tx: &Transaction,
    network_passphrase: &str,
) -> Result<TransactionEnvelope, XdrError> {
    let tx_hash = transaction_hash(tx, network_passphrase)?;
    let tx_signature = key.sign(&tx_hash);

    let decorated_signature = DecoratedSignature {
        hint: SignatureHint(key.verifying_key().to_bytes()[28..].try_into()?),
        signature: Signature(tx_signature.to_bytes().try_into()?),
    };

    Ok(TransactionEnvelope::Tx(TransactionV1Envelope {
        tx: tx.clone(),
        signatures: vec![decorated_signature].try_into()?,
    }))
}

/// # Errors
///
/// Might return an error
pub fn contract_id_from_str(contract_id: &str) -> Result<[u8; 32], stellar_strkey::DecodeError> {
    Ok(
        if let Ok(strkey) = stellar_strkey::Contract::from_string(contract_id) {
            strkey.0
        } else {
            // strkey failed, try to parse it as a hex string, for backwards compatibility.
            soroban_spec_tools::utils::padded_hex_from_str(contract_id, 32)
                .map_err(|_| stellar_strkey::DecodeError::Invalid)?
                .try_into()
                .map_err(|_| stellar_strkey::DecodeError::Invalid)?
        },
    )
}

/// # Errors
/// May not find a config dir
pub fn find_config_dir(mut pwd: std::path::PathBuf) -> std::io::Result<std::path::PathBuf> {
    let soroban_dir = |p: &std::path::Path| p.join(".soroban");
    while !soroban_dir(&pwd).exists() {
        if !pwd.pop() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "soroban directory not found",
            ));
        }
    }
    Ok(soroban_dir(&pwd))
}

pub(crate) fn into_signing_key(key: &PrivateKey) -> ed25519_dalek::SigningKey {
    let secret: ed25519_dalek::SecretKey = key.0;
    ed25519_dalek::SigningKey::from_bytes(&secret)
}

/// Used in tests
#[allow(unused)]
pub(crate) fn parse_secret_key(
    s: &str,
) -> Result<ed25519_dalek::SigningKey, stellar_strkey::DecodeError> {
    Ok(into_signing_key(&PrivateKey::from_string(s)?))
}

pub fn is_hex_string(s: &str) -> bool {
    s.chars().all(|s| s.is_ascii_hexdigit())
}

pub fn contract_id_hash_from_asset(
    asset: &Asset,
    network_passphrase: &str,
) -> Result<Hash, XdrError> {
    let network_id = Hash(Sha256::digest(network_passphrase.as_bytes()).into());
    let preimage = HashIdPreimage::ContractId(HashIdPreimageContractId {
        network_id,
        contract_id_preimage: ContractIdPreimage::Asset(asset.clone()),
    });
    let preimage_xdr = preimage.to_xdr(Limits::none())?;
    Ok(Hash(Sha256::digest(preimage_xdr).into()))
}

pub mod parsing {

    use regex::Regex;
    use soroban_env_host::xdr::{
        AccountId, AlphaNum12, AlphaNum4, Asset, AssetCode12, AssetCode4, PublicKey,
    };

    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error("invalid asset code: {asset}")]
        InvalidAssetCode { asset: String },
        #[error("cannot parse account id: {account_id}")]
        CannotParseAccountId { account_id: String },
        #[error("cannot parse asset: {asset}")]
        CannotParseAsset { asset: String },
        #[error(transparent)]
        Regex(#[from] regex::Error),
    }

    pub fn parse_asset(str: &str) -> Result<Asset, Error> {
        if str == "native" {
            return Ok(Asset::Native);
        }
        let split: Vec<&str> = str.splitn(2, ':').collect();
        if split.len() != 2 {
            return Err(Error::CannotParseAsset {
                asset: str.to_string(),
            });
        }
        let code = split[0];
        let issuer = split[1];
        let re = Regex::new("^[[:alnum:]]{1,12}$")?;
        if !re.is_match(code) {
            return Err(Error::InvalidAssetCode {
                asset: str.to_string(),
            });
        }
        if code.len() <= 4 {
            let mut asset_code: [u8; 4] = [0; 4];
            for (i, b) in code.as_bytes().iter().enumerate() {
                asset_code[i] = *b;
            }
            Ok(Asset::CreditAlphanum4(AlphaNum4 {
                asset_code: AssetCode4(asset_code),
                issuer: parse_account_id(issuer)?,
            }))
        } else {
            let mut asset_code: [u8; 12] = [0; 12];
            for (i, b) in code.as_bytes().iter().enumerate() {
                asset_code[i] = *b;
            }
            Ok(Asset::CreditAlphanum12(AlphaNum12 {
                asset_code: AssetCode12(asset_code),
                issuer: parse_account_id(issuer)?,
            }))
        }
    }

    pub fn parse_account_id(str: &str) -> Result<AccountId, Error> {
        let pk_bytes = stellar_strkey::ed25519::PublicKey::from_string(str)
            .map_err(|_| Error::CannotParseAccountId {
                account_id: str.to_string(),
            })?
            .0;
        Ok(AccountId(PublicKey::PublicKeyTypeEd25519(pk_bytes.into())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_id_from_str() {
        // strkey
        match contract_id_from_str("CA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQGAXE") {
            Ok(contract_id) => assert_eq!(
                contract_id,
                [
                    0x36, 0x3e, 0xaa, 0x38, 0x67, 0x84, 0x1f, 0xba, 0xd0, 0xf4, 0xed, 0x88, 0xc7,
                    0x79, 0xe4, 0xfe, 0x66, 0xe5, 0x6a, 0x24, 0x70, 0xdc, 0x98, 0xc0, 0xec, 0x9c,
                    0x07, 0x3d, 0x05, 0xc7, 0xb1, 0x03,
                ]
            ),
            Err(err) => panic!("Failed to parse contract id: {err}"),
        }
    }
}
