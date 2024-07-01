use clap::arg;
use serde::{Deserialize, Serialize};
use std::{io::Write, str::FromStr};
use stellar_strkey::ed25519::{PrivateKey, PublicKey};

use crate::{
    signer::{self, LocalKey, Stellar},
    utils,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid secret key")]
    InvalidSecretKey,
    // #[error("seed_phrase must be 12 words long, found {len}")]
    // InvalidSeedPhrase { len: usize },
    #[error("secret input error")]
    PasswordRead,
    #[error(transparent)]
    Secret(#[from] stellar_strkey::DecodeError),
    #[error(transparent)]
    SeedPhrase(#[from] sep5::error::Error),
    #[error(transparent)]
    Ed25519(#[from] ed25519_dalek::SignatureError),
    #[error("Invalid address {0}")]
    InvalidAddress(String),
    #[error("Ledger does not reveal secret key")]
    LedgerDoesNotRevealSecretKey,
    #[error(transparent)]
    Stellar(#[from] signer::Error),
}

#[derive(Debug, clap::Args, Clone)]
#[group(skip)]
pub struct Args {
    /// Add using secret_key
    /// Can provide with SOROBAN_SECRET_KEY
    #[arg(long, conflicts_with = "seed_phrase")]
    pub secret_key: bool,
    /// Add using 12 word seed phrase to generate secret_key
    #[arg(long, conflicts_with = "secret_key")]
    pub seed_phrase: bool,
}

impl Args {
    pub fn kind(&self) -> Result<SignerKind, Error> {
        if let Ok(secret_key) = std::env::var("SOROBAN_SECRET_KEY") {
            Ok(SignerKind::SecretKey { secret_key })
        } else if self.secret_key {
            println!("Type a secret key: ");
            let secret_key = read_password()?;
            let secret_key = PrivateKey::from_string(&secret_key)
                .map_err(|_| Error::InvalidSecretKey)?
                .to_string();
            Ok(SignerKind::SecretKey { secret_key })
        } else if self.seed_phrase {
            println!("Type a 12 word seed phrase: ");
            let seed_phrase = read_password()?;
            let seed_phrase: Vec<&str> = seed_phrase.split_whitespace().collect();
            // if seed_phrase.len() != 12 {
            //     let len = seed_phrase.len();
            //     return Err(Error::InvalidSeedPhrase { len });
            // }
            Ok(SignerKind::SeedPhrase {
                seed_phrase: seed_phrase
                    .into_iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" "),
            })
        } else {
            Err(Error::PasswordRead {})
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SignerKind {
    SecretKey { secret_key: String },
    SeedPhrase { seed_phrase: String },
}

impl FromStr for SignerKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if PrivateKey::from_string(s).is_ok() {
            Ok(SignerKind::SecretKey {
                secret_key: s.to_string(),
            })
        } else if sep5::SeedPhrase::from_str(s).is_ok() {
            Ok(SignerKind::SeedPhrase {
                seed_phrase: s.to_string(),
            })
        } else {
            Err(Error::InvalidAddress(s.to_string()))
        }
    }
}

impl From<PrivateKey> for SignerKind {
    fn from(value: PrivateKey) -> Self {
        SignerKind::SecretKey {
            secret_key: value.to_string(),
        }
    }
}

impl SignerKind {
    pub fn private_key(&self, index: Option<usize>) -> Result<PrivateKey, Error> {
        Ok(match self {
            SignerKind::SecretKey { secret_key } => PrivateKey::from_string(secret_key)?,
            SignerKind::SeedPhrase { seed_phrase } => PrivateKey::from_payload(
                &sep5::SeedPhrase::from_str(seed_phrase)?
                    .from_path_index(index.unwrap_or_default(), None)?
                    .private()
                    .0,
            )?,
        })
    }

    pub async fn public_key(&self, index: Option<usize>) -> Result<PublicKey, Error> {
        let key = self.signer(index, true)?;
        Ok(key.get_public_key().await?)
    }

    pub fn signer(&self, index: Option<usize>, prompt: bool) -> Result<StellarSigner, Error> {
        match self {
            SignerKind::SecretKey { .. } | SignerKind::SeedPhrase { .. } => Ok(
                StellarSigner::Local(LocalKey::new(self.key_pair(index)?, prompt)),
            ),
        }
    }

    pub fn key_pair(&self, index: Option<usize>) -> Result<ed25519_dalek::SigningKey, Error> {
        Ok(utils::into_signing_key(&self.private_key(index)?))
    }

    pub fn from_seed(seed: Option<&str>) -> Result<Self, Error> {
        let seed_phrase = if let Some(seed) = seed.map(str::as_bytes) {
            sep5::SeedPhrase::from_entropy(seed)
        } else {
            sep5::SeedPhrase::random(sep5::MnemonicType::Words12)
        }?
        .seed_phrase
        .into_phrase();
        Ok(SignerKind::SeedPhrase { seed_phrase })
    }

    pub fn test_seed_phrase() -> Result<Self, Error> {
        Self::from_seed(Some("0000000000000000"))
    }
}

pub enum StellarSigner {
    Local(LocalKey),
}

#[async_trait::async_trait]
impl Stellar for StellarSigner {
    async fn get_public_key(&self) -> Result<stellar_strkey::ed25519::PublicKey, signer::Error> {
        match self {
            StellarSigner::Local(signer) => signer.get_public_key().await,
        }
    }

    async fn sign_blob(&self, blob: &[u8]) -> Result<Vec<u8>, signer::Error> {
        match self {
            StellarSigner::Local(signer) => signer.sign_blob(blob).await,
        }
    }
}

fn read_password() -> Result<String, Error> {
    std::io::stdout().flush().map_err(|_| Error::PasswordRead)?;
    rpassword::read_password().map_err(|_| Error::PasswordRead)
}
