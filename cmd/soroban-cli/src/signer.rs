use crossterm::event::{read, Event, KeyCode};
use ed25519_dalek::ed25519::signature::Signer;
use sha2::{Digest, Sha256};

use crate::xdr::{
    self, AccountId, DecoratedSignature, Hash, HashIdPreimage, HashIdPreimageSorobanAuthorization,
    InvokeHostFunctionOp, Limits, Operation, OperationBody, PublicKey, ScAddress, ScMap, ScSymbol,
    ScVal, Signature, SignatureHint, SorobanAddressCredentials, SorobanAuthorizationEntry,
    SorobanAuthorizedFunction, SorobanCredentials, Transaction, TransactionEnvelope,
    TransactionSignaturePayload, TransactionSignaturePayloadTaggedTransaction,
    TransactionV1Envelope, Uint256, WriteXdr,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Contract addresses are not supported to sign auth entries {address}")]
    ContractAddressAreNotSupported { address: String },
    #[error(transparent)]
    Ed25519(#[from] ed25519_dalek::SignatureError),
    #[error("Missing signing key for account {address}")]
    MissingSignerForAddress { address: String },
    #[error(transparent)]
    Xdr(#[from] xdr::Error),
    #[error("User cancelled signing, perhaps need to remove --check")]
    UserCancelledSigning,
}

fn requires_auth(txn: &Transaction) -> Option<xdr::Operation> {
    let [op @ Operation {
        body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp { auth, .. }),
        ..
    }] = txn.operations.as_slice()
    else {
        return None;
    };
    matches!(
        auth.first().map(|x| &x.root_invocation.function),
        Some(&SorobanAuthorizedFunction::ContractFn(_))
    )
    .then(move || op.clone())
}

/// A trait for signing Stellar transactions and Soroban authorization entries
#[async_trait::async_trait]
pub trait Stellar {
    async fn get_public_key(&self) -> Result<stellar_strkey::ed25519::PublicKey, Error>;

    async fn sign_blob(&self, blob: &[u8]) -> Result<Vec<u8>, Error>;

    /// Sign a transaction hash with the given source account
    /// # Errors
    /// Returns an error if the source account is not found
    async fn sign_txn_hash(&self, txn: [u8; 32]) -> Result<DecoratedSignature, Error> {
        let source_account = self.get_public_key().await?;
        eprintln!(
            "{} about to sign hash: {}",
            source_account.to_string(),
            hex::encode(txn)
        );
        let tx_signature = self.sign_blob(&txn).await?;
        Ok(DecoratedSignature {
            // TODO: remove this unwrap. It's safe because we know the length of the array
            hint: SignatureHint(source_account.0[28..].try_into().unwrap()),
            signature: Signature(tx_signature.try_into()?),
        })
    }

    /// Sign a Soroban authorization entry with the given address
    /// # Errors
    /// Returns an error if the address is not found
    async fn sign_soroban_authorization_entry(
        &self,
        unsigned_entry: &SorobanAuthorizationEntry,
        network_passphrase: &str,
    ) -> Result<SorobanAuthorizationEntry, Error> {
        let address = self.get_public_key().await?;
        let mut auth = unsigned_entry.clone();
        let SorobanAuthorizationEntry {
            credentials: SorobanCredentials::Address(ref mut credentials),
            ..
        } = auth
        else {
            // Doesn't need special signing
            return Ok(auth);
        };
        let SorobanAddressCredentials {
            nonce,
            signature_expiration_ledger,
            ..
        } = credentials;

        let preimage = HashIdPreimage::SorobanAuthorization(HashIdPreimageSorobanAuthorization {
            network_id: hash(network_passphrase),
            invocation: auth.root_invocation.clone(),
            nonce: *nonce,
            signature_expiration_ledger: *signature_expiration_ledger,
        })
        .to_xdr(Limits::none())?;

        let payload = Sha256::digest(preimage);
        let signature = self.sign_blob(&payload).await?;

        let map = ScMap::sorted_from(vec![
            (
                ScVal::Symbol(ScSymbol("public_key".try_into()?)),
                ScVal::Bytes(address.0.to_vec().try_into()?),
            ),
            (
                ScVal::Symbol(ScSymbol("signature".try_into()?)),
                ScVal::Bytes(signature.try_into()?),
            ),
        ])?;
        credentials.signature = ScVal::Vec(Some(vec![ScVal::Map(Some(map))].try_into()?));
        auth.credentials = SorobanCredentials::Address(credentials.clone());

        Ok(auth)
    }

    /// Sign a Stellar transaction with the given source account
    /// This is a default implementation that signs the transaction hash and returns a decorated signature
    /// # Errors
    /// Returns an error if the source account is not found
    async fn sign_txn(
        &self,
        txn: Transaction,
        network_passphrase: &str,
    ) -> Result<TransactionEnvelope, Error> {
        let signature_payload = TransactionSignaturePayload {
            network_id: hash(network_passphrase),
            tagged_transaction: TransactionSignaturePayloadTaggedTransaction::Tx(txn.clone()),
        };
        let hash = Sha256::digest(signature_payload.to_xdr(Limits::none())?).into();
        let decorated_signature = self.sign_txn_hash(hash).await?;
        Ok(TransactionEnvelope::Tx(TransactionV1Envelope {
            tx: txn,
            signatures: vec![decorated_signature].try_into()?,
        }))
    }

    /// Sign a Soroban authorization entries for a given transaction and set the expiration ledger
    /// # Errors
    /// Returns an error if the address is not found
    async fn sign_soroban_authorizations(
        &self,
        raw: &Transaction,
        network_passphrase: &str,
    ) -> Result<Option<Transaction>, Error> {
        let mut tx = raw.clone();
        let Some(mut op) = requires_auth(&tx) else {
            return Ok(None);
        };

        let xdr::Operation {
            body: OperationBody::InvokeHostFunction(ref mut body),
            ..
        } = op
        else {
            return Ok(None);
        };

        let mut auths = body.auth.to_vec();
        for auth in &mut auths {
            *auth = self
                .maybe_sign_soroban_authorization_entry(auth, network_passphrase)
                .await?;
        }
        body.auth = auths.try_into()?;
        tx.operations = vec![op].try_into()?;
        Ok(Some(tx))
    }

    /// Sign a Soroban authorization entry if the address is public key
    /// # Errors
    /// Returns an error if the address in entry is a contract
    async fn maybe_sign_soroban_authorization_entry(
        &self,
        unsigned_entry: &SorobanAuthorizationEntry,
        network_passphrase: &str,
    ) -> Result<SorobanAuthorizationEntry, Error> {
        if let SorobanAuthorizationEntry {
            credentials: SorobanCredentials::Address(SorobanAddressCredentials { ref address, .. }),
            ..
        } = unsigned_entry
        {
            // See if we have a signer for this authorizationEntry
            // If not, then we Error
            let needle = match address {
                ScAddress::Account(AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(a)))) => {
                    stellar_strkey::ed25519::PublicKey(*a)
                }
                ScAddress::Contract(Hash(c)) => {
                    // This address is for a contract. This means we're using a custom
                    // smart-contract account. Currently the CLI doesn't support that yet.
                    return Err(Error::MissingSignerForAddress {
                        address: stellar_strkey::Strkey::Contract(stellar_strkey::Contract(*c))
                            .to_string(),
                    });
                }
            };
            if needle == self.get_public_key().await? {
                return Ok(unsigned_entry.clone());
            }
            self.sign_soroban_authorization_entry(unsigned_entry, network_passphrase)
                .await
        } else {
            Ok(unsigned_entry.clone())
        }
    }
}

fn hash(network_passphrase: &str) -> xdr::Hash {
    xdr::Hash(Sha256::digest(network_passphrase.as_bytes()).into())
}

pub struct LocalKey {
    key: ed25519_dalek::SigningKey,
    prompt: bool,
}

impl LocalKey {
    pub fn new(key: ed25519_dalek::SigningKey, prompt: bool) -> Self {
        Self { key, prompt }
    }
}

#[async_trait::async_trait]
impl Stellar for LocalKey {
    async fn sign_blob(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        if self.prompt {
            eprintln!("Press 'y' or 'Y' for yes, any other key for no:");
            match read_key() {
                'y' | 'Y' => {
                    eprintln!("Signing now...");
                }
                _ => return Err(Error::UserCancelledSigning),
            };
        }
        let sig = self.key.sign(data);
        Ok(sig.to_bytes().to_vec())
    }

    async fn get_public_key(&self) -> Result<stellar_strkey::ed25519::PublicKey, Error> {
        Ok(stellar_strkey::ed25519::PublicKey(
            self.key.verifying_key().to_bytes(),
        ))
    }
}

pub fn read_key() -> char {
    loop {
        if let Event::Key(key) = read().unwrap() {
            match key.code {
                KeyCode::Char(c) => return c,
                KeyCode::Esc => return '\x1b', // escape key
                _ => (),
            }
        }
    }
}
