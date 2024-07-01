use crate::xdr::{self, Limits, Transaction, TransactionEnvelope, WriteXdr};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    XdrArgs(#[from] super::xdr::Error),
    #[error(transparent)]
    Config(#[from] super::super::config::Error),
    #[error(transparent)]
    Xdr(#[from] xdr::Error),
}

#[derive(Debug, clap::Parser, Clone)]
#[group(skip)]
pub struct Cmd {
    #[clap(flatten)]
    pub config: super::super::config::Args,
}

impl Cmd {
    #[allow(clippy::unused_async)]
    pub async fn run(&self) -> Result<(), Error> {
        let txn_env = super::xdr::tx_envelope_from_stdin()?;
        let envelope = self.sign_env(txn_env).await?;
        println!("{}", envelope.to_xdr_base64(Limits::none())?.trim());
        Ok(())
    }

    pub async fn sign(&self, tx: Transaction) -> Result<TransactionEnvelope, Error> {
        Ok(self.config.sign(tx).await?)
    }

    pub async fn sign_env(
        &self,
        tx_env: TransactionEnvelope,
    ) -> Result<TransactionEnvelope, Error> {
        self.sign(super::xdr::unwrap_envelope_v1(tx_env)?).await
    }
}
