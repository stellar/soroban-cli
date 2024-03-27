// use ed25519_dalek::Signer;
// use sha2::{Digest, Sha256};

// use soroban_env_host::xdr::{
//     self, AccountId, DecoratedSignature, Hash, HashIdPreimage, HashIdPreimageSorobanAuthorization,
//     Limits, OperationBody, PublicKey, ScAddress, ScMap, ScSymbol, ScVal, Signature, SignatureHint,
//     SorobanAddressCredentials, SorobanAuthorizationEntry, SorobanCredentials, Transaction,
//     TransactionEnvelope, TransactionSignaturePayload, TransactionSignaturePayloadTaggedTransaction,
//     TransactionV1Envelope, Uint256, WriteXdr, Operation, InvokeHostFunctionOp, SorobanAuthorizedFunction,
// };

// enum Error {}

// fn requires_auth(txn: &Transaction) -> Option<xdr::Operation> {
//   let [op @ Operation {
//       body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp { auth, .. }),
//       ..
//   }] = txn.operations.as_slice()
//   else {
//       return None;
//   };
//   matches!(
//       auth.first().map(|x| &x.root_invocation.function),
//       Some(&SorobanAuthorizedFunction::ContractFn(_))
//   )
//   .then(move || op.clone())
// }

// /// A trait for signing Stellar transactions and Soroban authorization entries
// pub trait Stellar {
//     /// The type of the options that can be passed when creating a new signer
//     type Init;
//     /// Create a new signer with the given network passphrase and options
//     fn new(network_passphrase: &str, options: Option<Self::Init>) -> Self;

//     /// Get the network hash
//     fn network_hash(&self) -> xdr::Hash;

//     /// Sign a transaction hash with the given source account
//     /// # Errors
//     /// Returns an error if the source account is not found
//     fn sign_txn_hash(
//         &self,
//         txn: [u8; 32],
//         source_account: &stellar_strkey::Strkey,
//     ) -> Result<DecoratedSignature, Error>;

//     /// Sign a Soroban authorization entry with the given address
//     /// # Errors
//     /// Returns an error if the address is not found
//     fn sign_soroban_authorization_entry(
//         &self,
//         unsigned_entry: &SorobanAuthorizationEntry,
//         signature_expiration_ledger: u32,
//         address: &[u8; 32],
//     ) -> Result<SorobanAuthorizationEntry, Error>;

//     /// Sign a Stellar transaction with the given source account
//     /// This is a default implementation that signs the transaction hash and returns a decorated signature
//     /// # Errors
//     /// Returns an error if the source account is not found
//     fn sign_txn(
//         &self,
//         txn: Transaction,
//         source_account: &stellar_strkey::Strkey,
//     ) -> Result<TransactionEnvelope, Error> {
//         let signature_payload = TransactionSignaturePayload {
//             network_id: self.network_hash(),
//             tagged_transaction: TransactionSignaturePayloadTaggedTransaction::Tx(txn.clone()),
//         };
//         let hash = Sha256::digest(&signature_payload.to_xdr(Limits::none())?).into();
//         let decorated_signature = self.sign_txn_hash(hash, source_account)?;
//         Ok(TransactionEnvelope::Tx(TransactionV1Envelope {
//             tx: txn,
//             signatures: vec![decorated_signature].try_into()?,
//         }))
//     }

//     /// Sign a Soroban authorization entries for a given transaction and set the expiration ledger
//     /// # Errors
//     /// Returns an error if the address is not found
//     fn sign_soroban_authorizations(
//         &self,
//         raw: &Transaction,
//         signature_expiration_ledger: u32,
//     ) -> Result<Option<Transaction>, Error> {
//         let mut tx = raw.clone();
//         let Some(mut op) = requires_auth(&tx) else {
//             return Ok(None);
//         };

//         let xdr::Operation {
//             body: OperationBody::InvokeHostFunction(ref mut body),
//             ..
//         } = op
//         else {
//             return Ok(None);
//         };

//         let signed_auths = body
//             .auth
//             .as_slice()
//             .iter()
//             .map(|raw_auth| {
//                 self.maybe_sign_soroban_authorization_entry(raw_auth, signature_expiration_ledger)
//             })
//             .collect::<Result<Vec<_>, Error>>()?;

//         body.auth = signed_auths.try_into()?;
//         tx.operations = vec![op].try_into()?;
//         Ok(Some(tx))
//     }

//     /// Sign a Soroban authorization entry if the address is public key
//     /// # Errors
//     /// Returns an error if the address in entry is a contract
//     fn maybe_sign_soroban_authorization_entry(
//         &self,
//         unsigned_entry: &SorobanAuthorizationEntry,
//         signature_expiration_ledger: u32,
//     ) -> Result<SorobanAuthorizationEntry, Error> {
//         if let SorobanAuthorizationEntry {
//             credentials: SorobanCredentials::Address(SorobanAddressCredentials { ref address, .. }),
//             ..
//         } = unsigned_entry
//         {
//             // See if we have a signer for this authorizationEntry
//             // If not, then we Error
//             let needle = match address {
//                 ScAddress::Account(AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(ref a)))) => a,
//                 ScAddress::Contract(Hash(c)) => {
//                     // This address is for a contract. This means we're using a custom
//                     // smart-contract account. Currently the CLI doesn't support that yet.
//                     return Err(Error::MissingSignerForAddress {
//                         address: stellar_strkey::Strkey::Contract(stellar_strkey::Contract(*c))
//                             .to_string(),
//                     });
//                 }
//             };
//             self.sign_soroban_authorization_entry(
//                 unsigned_entry,
//                 signature_expiration_ledger,
//                 needle,
//             )
//         } else {
//             Ok(unsigned_entry.clone())
//         }
//     }
// }


// use std::fmt::Debug;
// #[derive(Debug)]
// pub struct DefaultSigner {
//     network_passphrase: String,
//     keypairs: Vec<ed25519_dalek::SigningKey>,
// }

// impl DefaultSigner {
//     pub fn get_key(
//         &self,
//         key: &stellar_strkey::Strkey,
//     ) -> Result<&ed25519_dalek::SigningKey, Error> {
//         match key {
//             stellar_strkey::Strkey::PublicKeyEd25519(stellar_strkey::ed25519::PublicKey(bytes)) => {
//                 self.keypairs
//                     .iter()
//                     .find(|k| k.verifying_key().to_bytes() == *bytes)
//             }
//             _ => None,
//         }
//         .ok_or_else(|| Error::MissingSignerForAddress {
//             address: key.to_string(),
//         })
//     }
// }

// impl Stellar for DefaultSigner {
//     type Init = Vec<ed25519_dalek::SigningKey>;
//     fn new(network_passphrase: &str, options: Option<Vec<ed25519_dalek::SigningKey>>) -> Self {
//         DefaultSigner {
//             network_passphrase: network_passphrase.to_string(),
//             keypairs: options.unwrap_or_default(),
//         }
//     }

//     fn sign_txn_hash(
//         &self,
//         txn: [u8; 32],
//         source_account: &stellar_strkey::Strkey,
//     ) -> Result<DecoratedSignature, Error> {
//         let source_account = self.get_key(source_account)?;
//         let tx_signature = source_account.sign(&txn);
//         Ok(DecoratedSignature {
//             // TODO: remove this unwrap. It's safe because we know the length of the array
//             hint: SignatureHint(
//                 source_account.verifying_key().to_bytes()[28..]
//                     .try_into()
//                     .unwrap(),
//             ),
//             signature: Signature(tx_signature.to_bytes().try_into()?),
//         })
//     }

//     fn sign_soroban_authorization_entry(
//         &self,
//         unsigned_entry: &SorobanAuthorizationEntry,
//         signature_expiration_ledger: u32,
//         signer: &[u8; 32],
//     ) -> Result<SorobanAuthorizationEntry, Error> {
//         let mut auth = unsigned_entry.clone();
//         let SorobanAuthorizationEntry {
//             credentials: SorobanCredentials::Address(ref mut credentials),
//             ..
//         } = auth
//         else {
//             // Doesn't need special signing
//             return Ok(auth);
//         };
//         let SorobanAddressCredentials { nonce, .. } = credentials;

//         let preimage = HashIdPreimage::SorobanAuthorization(HashIdPreimageSorobanAuthorization {
//             network_id: self.network_hash(),
//             invocation: auth.root_invocation.clone(),
//             nonce: *nonce,
//             signature_expiration_ledger,
//         })
//         .to_xdr(Limits::none())?;

//         let strkey = stellar_strkey::ed25519::PublicKey(*signer);
//         let payload = Sha256::digest(&preimage);
//         let signer = self.get_key(&stellar_strkey::Strkey::PublicKeyEd25519(strkey))?;
//         let signature = signer.sign(&payload);

//         let map = ScMap::sorted_from(vec![
//             (
//                 ScVal::Symbol(ScSymbol("public_key".try_into()?)),
//                 ScVal::Bytes(
//                     signer
//                         .verifying_key()
//                         .to_bytes()
//                         .to_vec()
//                         .try_into()
//                         .map_err(Error::Xdr)?,
//                 ),
//             ),
//             (
//                 ScVal::Symbol(ScSymbol("signature".try_into()?)),
//                 ScVal::Bytes(
//                     signature
//                         .to_bytes()
//                         .to_vec()
//                         .try_into()
//                         .map_err(Error::Xdr)?,
//                 ),
//             ),
//         ])
//         .map_err(Error::Xdr)?;
//         credentials.signature = ScVal::Vec(Some(
//             vec![ScVal::Map(Some(map))].try_into().map_err(Error::Xdr)?,
//         ));
//         credentials.signature_expiration_ledger = signature_expiration_ledger;
//         auth.credentials = SorobanCredentials::Address(credentials.clone());

//         Ok(auth)
//     }

//     fn network_hash(&self) -> xdr::Hash {
//         xdr::Hash(Sha256::digest(self.network_passphrase.as_bytes()).into())
//     }
// }
