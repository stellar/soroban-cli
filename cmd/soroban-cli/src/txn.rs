//! This module contains a transaction builder for Stellar.
//!
use soroban_env_host::xdr::{self, Operation, Transaction, Uint256, VecM};


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("")]
    InvokeHostFunctionOpMustBeOnlyOperation,
    #[error(transparent)]
    Xdr(#[from] xdr::Error),
    #[error("invalid source account strkey type")]
    InvalidSourceAccountStrkeyType,
}

fn to_muxed_account(source_account: stellar_strkey::Strkey) -> Result<xdr::MuxedAccount, Error> {
    let raw_bytes = match source_account {
        stellar_strkey::Strkey::PublicKeyEd25519(x) => x.0,
        stellar_strkey::Strkey::MuxedAccountEd25519(x) => x.ed25519,
        _ => return Err(Error::InvalidSourceAccountStrkeyType),
    };
    Ok(xdr::MuxedAccount::Ed25519(xdr::Uint256(raw_bytes)))
}

pub struct TransactionBuilder {
    pub txn: Transaction,
}

impl TransactionBuilder {
    pub fn new(source_account: stellar_strkey::Strkey) -> Result<Self, Error> {
        let source_account = to_muxed_account(source_account)?;
        Ok(Self {
            txn: Transaction {
                source_account,
                fee: 100,
                operations: VecM::default(),
                seq_num: xdr::SequenceNumber(0),
                cond: xdr::Preconditions::None,
                memo: xdr::Memo::None,
                ext: xdr::TransactionExt::V0,
            },
        })
    }

    pub fn set_source_account(&mut self, source_account: stellar_strkey::Strkey) -> Result<&mut Self, Error> {
        self.txn.source_account = to_muxed_account(source_account)?;
        Ok(self)
    }

    pub fn set_fee(&mut self, fee: u32) -> &mut Self {
        self.txn.fee = fee;
        self
    }

    pub fn set_sequence_number(&mut self, sequence_number: i64) -> &mut Self {
        self.txn.seq_num = xdr::SequenceNumber(sequence_number);
        self
    }

    pub fn add_operation(&mut self, operation: Operation) -> Result<&mut Self, Error> {
        if !self.txn.operations.is_empty()
            && matches!(
                operation,
                Operation {
                    body: xdr::OperationBody::InvokeHostFunction(_),
                    ..
                }
            )
        {
            return Err(Error::InvokeHostFunctionOpMustBeOnlyOperation);
        }
        self.txn.operations.push(operation);
        Ok(self)
    }

    pub fn cond(&mut self, cond: xdr::Preconditions) -> &mut Self {
        self.txn.cond = cond;
        self
    }

    pub fn build(&self) -> Transaction {
        self.txn.clone()
    }
}

pub struct OperationBuilder {
    op: Operation,
}

impl OperationBuilder {
    pub fn new() -> Self {
        Self {
            op: Operation {
                source_account: None,
                body: xdr::OperationBody::Inflation,
            },
        }
    }

    pub fn set_source_account(&mut self, source_account: stellar_strkey::Strkey) -> Result<&mut Self, Error> {
        self.op.source_account = Some(to_muxed_account(source_account)?);
        Ok(self)
    }

    pub fn set_body(&mut self, body: xdr::OperationBody) -> &mut Self {
        self.op.body = body;
        self
    }

    pub fn set_host_function(&mut self, host_function: xdr::HostFunction) -> &mut Self {
        if let xdr::OperationBody::InvokeHostFunction(ref mut op) = self.op.body {
            op.host_function = host_function;
        }
        self
    }

    pub fn set_auth(&mut self, auth: VecM<u8>) -> &mut Self {
        if let xdr::OperationBody::InvokeHostFunction(ref mut op) = self.op.body {
            op.auth = auth;
        }
        self
    }

    pub fn build(&self) -> Operation {
        self.op.clone()
    }
}

pub struct OperationBodyBuilder {
    body: xdr::OperationBody,
}

impl OperationBodyBuilder {
    pub fn new() -> Self {
        Self {
            body: xdr::OperationBody::Inflation,
        }
    }

    pub fn set_invoke_host_function(&mut self, invoke_host_function: xdr::InvokeHostFunctionOp) -> &mut Self {
        self.body = xdr::OperationBody::InvokeHostFunction(invoke_host_function);
        self
    }

    pub fn build(&self) -> xdr::OperationBody {
        self.body.clone()
    }
}

pub struct InvokeHostFunctionOpBuilder(xdr::HostFunction, Vec<xdr::SorobanAuthorizationEntry>);

impl InvokeHostFunctionOpBuilder {
    fn new(host_function: xdr::HostFunction) -> Self {
        Self(host_function, vec![])
    }
    pub fn upload(wasm: &[u8]) -> Result<Self, Error> {
        Ok(Self::new(xdr::HostFunction::UploadContractWasm(
            wasm.try_into()?,
        )))
    }

    pub fn create_contract(
        source_account: stellar_strkey::Strkey,
        salt: [u8; 32],
        wasm_hash: xdr::Hash,
    ) -> Result<Self, Error> {
        let stellar_strkey::Strkey::PublicKeyEd25519(bytes) = source_account else {
            panic!("Invalid public key");
        };

        let contract_id_preimage =
            xdr::ContractIdPreimage::Address(xdr::ContractIdPreimageFromAddress {
                address: xdr::ScAddress::Account(xdr::AccountId(
                    xdr::PublicKey::PublicKeyTypeEd25519(bytes.0.into()),
                )),
                salt: Uint256(salt),
            });

        Ok(Self::new(xdr::HostFunction::CreateContract(
            xdr::CreateContractArgs {
                contract_id_preimage,
                executable: xdr::ContractExecutable::Wasm(wasm_hash),
            },
        )))
    }

    pub fn add_auth(&mut self, auth: xdr::SorobanAuthorizationEntry) -> &mut Self {
        self.1.push(auth);
        self
    }

    pub fn build(self) -> Result<xdr::OperationBody, Error> {
        Ok(xdr::OperationBody::InvokeHostFunction(
            xdr::InvokeHostFunctionOp {
                host_function: self.0,
                auth: self.1.try_into()?,
            },
        ))
    }
}
