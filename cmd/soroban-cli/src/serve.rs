use std::collections::HashMap;
use std::{convert::Infallible, fmt::Debug, io, net::SocketAddr, path::PathBuf, rc::Rc, sync::Arc};

use clap::Parser;
use hex::FromHexError;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use soroban_env_host::{
    budget::Budget,
    storage::Storage,
    xdr::{
        self, AccountId, AddressWithNonce, ContractAuth, Error as XdrError,
        FeeBumpTransactionInnerTx, HostFunction, LedgerKey, MuxedAccount, Operation, OperationBody,
        PublicKey, ReadXdr, ScHostStorageErrorCode, ScObject, ScStatus, ScVal, ScVec,
        TransactionEnvelope, WriteXdr,
    },
    Host, HostError,
};
use tokio::sync::Mutex;
use warp::{http::Response, Filter};

use crate::network::SANDBOX_NETWORK_PASSPHRASE;
use crate::strval;
use crate::utils::{self, create_ledger_footprint};
use crate::{jsonrpc, HEADING_SANDBOX};

#[derive(Parser, Debug)]
pub struct Cmd {
    /// Port to listen for requests on.
    #[clap(long, default_value("8000"))]
    port: u16,

    /// File to persist ledger state
    #[clap(
        long,
        parse(from_os_str),
        default_value(".soroban/ledger.json"),
        env = "SOROBAN_LEDGER_FILE",
        help_heading = HEADING_SANDBOX,
    )]
    ledger_file: PathBuf,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io")]
    Io(#[from] io::Error),
    #[error("strval")]
    StrVal(#[from] strval::Error),
    #[error("xdr")]
    Xdr(#[from] XdrError),
    #[error("host")]
    Host(#[from] HostError),
    #[error("snapshot")]
    Snapshot(#[from] soroban_ledger_snapshot::Error),
    #[error("serde")]
    Serde(#[from] serde_json::Error),
    #[error("unsupported transaction: {message}")]
    UnsupportedTransaction { message: String },
    #[error("hex")]
    FromHex(#[from] FromHexError),
    #[error("unknownmethod")]
    UnknownMethod,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
enum Requests {
    NoArg(),
    StringArg(Box<[String]>),
}

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        let ledger_file = Arc::new(self.ledger_file.clone());
        let with_ledger_file = warp::any().map(move || ledger_file.clone());

        // Just track in-flight transactions in-memory for sandbox for now. Simple.
        let transaction_status_map: Arc<Mutex<HashMap<String, Value>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let with_transaction_status_map = warp::any().map(move || transaction_status_map.clone());

        let jsonrpc_route = warp::post()
            .and(warp::path!("soroban" / "rpc"))
            .and(warp::body::json())
            .and(with_ledger_file)
            .and(with_transaction_status_map)
            .and_then(handler);

        // Allow access from all remote sites when we are in local sandbox mode. (Always for now)
        let cors = warp::cors()
            .allow_any_origin()
            .allow_credentials(true)
            .allow_headers(vec![
                "Accept",
                "Access-Control-Request-Headers",
                "Access-Control-Request-Method",
                "Content-Length",
                "Content-Type",
                "Encoding",
                "Origin",
                "Referer",
                "Sec-Fetch-Mode",
                "User-Agent",
                "X-Client-Name",
                "X-Client-Version",
            ])
            .allow_methods(vec!["GET", "POST"]);
        let routes = jsonrpc_route.with(cors);

        let addr: SocketAddr = ([127, 0, 0, 1], self.port).into();
        println!("Listening on: {addr}/soroban/rpc");
        warp::serve(routes).run(addr).await;
        Ok(())
    }
}

async fn handler(
    request: jsonrpc::Request<Requests>,
    ledger_file: Arc<PathBuf>,
    transaction_status_map: Arc<Mutex<HashMap<String, Value>>>,
) -> Result<impl warp::Reply, Infallible> {
    let resp = Response::builder()
        .status(200)
        .header("content-type", "application/json; charset=utf-8");
    if request.jsonrpc != "2.0" {
        return Ok(resp.body(
            json!({
                "jsonrpc": "2.0",
                "id": &request.id,
                "error": {
                    "code":-32600,
                    "message": "Invalid jsonrpc value in request",
                },
            })
            .to_string(),
        ));
    }
    let result = match (request.method.as_str(), request.params) {
        ("getHealth", Some(Requests::NoArg()) | None) => Ok(get_health()),
        ("getLedgerEntry", Some(Requests::StringArg(key))) => get_ledger_entry(key, &ledger_file),
        ("getNetwork", Some(Requests::NoArg())) => Ok(get_network()),
        ("getTransactionStatus", Some(Requests::StringArg(b))) => {
            get_transaction_status(&transaction_status_map, b).await
        }
        ("simulateTransaction", Some(Requests::StringArg(b))) => {
            simulate_transaction(&ledger_file, b)
        }
        ("sendTransaction", Some(Requests::StringArg(b))) => {
            send_transaction(&ledger_file, &transaction_status_map, b).await
        }
        _ => Err(Error::UnknownMethod),
    };
    let r = reply(&request.id, result);
    Ok(resp.body(serde_json::to_string(&r).unwrap_or_else(|_| {
        json!({
            "jsonrpc": "2.0",
            "id": &request.id,
            "error": {
                "code":-32603,
                "message": "Internal server error",
            },
        })
        .to_string()
    })))
}

fn reply(
    id: &Option<jsonrpc::Id>,
    result: Result<Value, Error>,
) -> jsonrpc::Response<Value, Value> {
    match result {
        Ok(res) => jsonrpc::Response::Ok(jsonrpc::ResultResponse {
            jsonrpc: "2.0".to_string(),
            id: id.as_ref().unwrap_or(&jsonrpc::Id::Null).clone(),
            result: res,
        }),
        Err(err) => {
            eprintln!("err: {err:?}");
            jsonrpc::Response::Err(jsonrpc::ErrorResponse {
                jsonrpc: "2.0".to_string(),
                id: id.as_ref().unwrap_or(&jsonrpc::Id::Null).clone(),
                error: jsonrpc::ErrorResponseError {
                    code: match err {
                        Error::Serde(_) => -32700,
                        Error::UnknownMethod => -32601,
                        _ => -32603,
                    },
                    message: err.to_string(),
                    data: None,
                },
            })
        }
    }
}

fn get_ledger_entry(k: Box<[String]>, ledger_file: &PathBuf) -> Result<Value, Error> {
    if let Some(key_xdr) = k.into_vec().first() {
        // Initialize storage and host
        let state = utils::ledger_snapshot_read_or_default(ledger_file).map_err(Error::Snapshot)?;
        let key = LedgerKey::from_xdr_base64(key_xdr)?;

        let snap = Rc::new(state);
        let mut storage = Storage::with_recording_footprint(snap);
        let ledger_entry = storage.get(&key, &soroban_env_host::budget::Budget::default())?;

        Ok(json!({
            "xdr": ledger_entry.data.to_xdr_base64()?,
            "lastModifiedLedgerSeq": ledger_entry.last_modified_ledger_seq,
            // TODO: Find "real" ledger seq number here
            "latestLedger": 1,
        }))
    } else {
        Err(Error::Xdr(XdrError::Invalid))
    }
}

fn get_network() -> Value {
    json!({
        "passphrase": SANDBOX_NETWORK_PASSPHRASE,
        "protocolVersion": 20,
    })
}

fn parse_transaction(
    txn_xdr: &str,
    passphrase: &str,
) -> Result<(AccountId, [u8; 32], Vec<ScVal>), Error> {
    // Parse and validate the txn
    let transaction = TransactionEnvelope::from_xdr_base64(txn_xdr)?;
    let hash = hash_transaction_in_envelope(&transaction, passphrase)?;
    let ops = match &transaction {
        TransactionEnvelope::TxV0(envelope) => &envelope.tx.operations,
        TransactionEnvelope::Tx(envelope) => &envelope.tx.operations,
        TransactionEnvelope::TxFeeBump(envelope) => {
            let FeeBumpTransactionInnerTx::Tx(tx_envelope) = &envelope.tx.inner_tx;
            &tx_envelope.tx.operations
        }
    };
    if ops.len() != 1 {
        return Err(Error::UnsupportedTransaction {
            message: "Must only have one operation".to_string(),
        });
    }
    let op = ops.first().ok_or(Error::Xdr(XdrError::Invalid))?;
    let source_account = parse_op_source_account(&transaction, op);
    let OperationBody::InvokeHostFunction(body) = &op.body else {
        return Err(Error::UnsupportedTransaction {
            message: "Operation must be invokeHostFunction".to_string(),
        });
    };

    // TODO: Support creating contracts and token wrappers here as well.
    let HostFunction::InvokeContract(parameters) = &body.function else {
        return Err(Error::UnsupportedTransaction {
            message: "Function must be invokeContract".to_string(),
        });
    };

    if parameters.len() < 2 {
        return Err(Error::UnsupportedTransaction {
            message: "Function must have at least 2 parameters".to_string(),
        });
    };

    let contract_xdr = parameters.get(0).ok_or(Error::UnsupportedTransaction {
        message: "First parameter must be the contract id".to_string(),
    })?;
    let method_xdr = parameters.get(1).ok_or(Error::UnsupportedTransaction {
        message: "Second parameter must be the contract method".to_string(),
    })?;
    let (_, params) = parameters.split_at(2);

    let contract_id: [u8; 32] = if let ScVal::Object(Some(ScObject::Bytes(bytes))) = contract_xdr {
        bytes
            .as_slice()
            .try_into()
            .map_err(|_| Error::UnsupportedTransaction {
                message: "Could not parse contract id".to_string(),
            })?
    } else {
        return Err(Error::UnsupportedTransaction {
            message: "Could not parse contract id".to_string(),
        });
    };

    // TODO: Figure out and enforce the expected type here. For now, handle both a symbol and a
    // binary. The cap says binary, but other implementations use symbol.
    let method: String = if let ScVal::Object(Some(ScObject::Bytes(bytes))) = method_xdr {
        bytes
            .try_into()
            .map_err(|_| Error::UnsupportedTransaction {
                message: "Could not parse contract method".to_string(),
            })?
    } else if let ScVal::Symbol(bytes) = method_xdr {
        bytes
            .try_into()
            .map_err(|_| Error::UnsupportedTransaction {
                message: "Could not parse contract method".to_string(),
            })?
    } else {
        return Err(Error::UnsupportedTransaction {
            message: "Could not parse contract method".to_string(),
        });
    };

    let mut complete_args = vec![
        ScVal::Object(Some(ScObject::Bytes(contract_id.try_into()?))),
        ScVal::Symbol(method.try_into()?),
    ];
    complete_args.extend_from_slice(params);

    Ok((source_account, hash, complete_args))
}

fn execute_transaction(
    source_account: AccountId,
    args: &Vec<ScVal>,
    ledger_file: &PathBuf,
    commit: bool,
) -> Result<Value, Error> {
    // Initialize storage and host
    let mut state = utils::ledger_snapshot_read_or_default(ledger_file).map_err(Error::Snapshot)?;

    let snap = Rc::new(state.clone());
    let storage = Storage::with_recording_footprint(snap);
    let h = Host::with_storage_and_budget(storage, Budget::default());

    h.switch_to_recording_auth();
    h.set_source_account(source_account);

    let mut ledger_info = state.ledger_info();
    ledger_info.sequence_number += 1;
    ledger_info.timestamp += 5;
    let latest_sequence = ledger_info.sequence_number;
    h.set_ledger_info(ledger_info);

    // TODO: Check the parameters match the contract spec, or return a helpful error message

    // TODO: Handle installing code and creating contracts here as well
    let res = h.invoke_function(HostFunction::InvokeContract(args.try_into()?))?;

    state.update(&h);

    let contract_auth: Vec<String> = h
        .get_recorded_auth_payloads()?
        .into_iter()
        .map(|payload| {
            let address_with_nonce = match (payload.address, payload.nonce) {
                (Some(address), Some(nonce)) => Some(AddressWithNonce { address, nonce }),
                _ => None,
            };
            ContractAuth {
                address_with_nonce,
                root_invocation: payload.invocation,
                signature_args: ScVec::default(),
            }
            .to_xdr_base64()
        })
        .collect::<Result<Vec<_>, _>>()?;

    let (storage, budget, _) = h.try_finish().map_err(|_h| {
        HostError::from(ScStatus::HostStorageError(
            ScHostStorageErrorCode::UnknownError,
        ))
    })?;

    // Calculate the budget usage
    let mut cost = serde_json::Map::new();
    cost.insert(
        "cpuInsns".to_string(),
        Value::String(budget.get_cpu_insns_count().to_string()),
    );
    cost.insert(
        "memBytes".to_string(),
        Value::String(budget.get_mem_bytes_count().to_string()),
    );
    // TODO: Include these extra costs. Figure out the rust type conversions.
    // for cost_type in CostType::variants() {
    //     m.insert(cost_type, b.get_input(*cost_type));
    // }

    // Calculate the storage footprint
    let footprint = create_ledger_footprint(&storage.footprint);

    if commit {
        state.write_file(ledger_file).map_err(Error::Snapshot)?;
    }

    Ok(json!({
        "cost": cost,
        "results": [{
            "auth": contract_auth,
            "footprint": footprint.to_xdr_base64()?,
            "xdr": res.to_xdr_base64()?,
        }],
        "latestLedger": latest_sequence,
    }))
}

fn hash_transaction_in_envelope(
    envelope: &TransactionEnvelope,
    passphrase: &str,
) -> Result<[u8; 32], Error> {
    let tagged_transaction = match envelope {
        TransactionEnvelope::TxV0(envelope) => {
            xdr::TransactionSignaturePayloadTaggedTransaction::Tx(xdr::Transaction {
                source_account: xdr::MuxedAccount::Ed25519(
                    envelope.tx.source_account_ed25519.clone(),
                ),
                fee: envelope.tx.fee,
                seq_num: envelope.tx.seq_num.clone(),
                cond: match envelope.tx.time_bounds.clone() {
                    None => xdr::Preconditions::None,
                    Some(time_bounds) => xdr::Preconditions::Time(time_bounds),
                },
                memo: envelope.tx.memo.clone(),
                operations: envelope.tx.operations.clone(),
                ext: xdr::TransactionExt::V0,
            })
        }
        TransactionEnvelope::Tx(envelope) => {
            xdr::TransactionSignaturePayloadTaggedTransaction::Tx(envelope.tx.clone())
        }
        TransactionEnvelope::TxFeeBump(envelope) => {
            xdr::TransactionSignaturePayloadTaggedTransaction::TxFeeBump(envelope.tx.clone())
        }
    };

    // trim spaces from passphrase
    // Check if network passpharse is empty

    let network_id = xdr::Hash(hash_bytes(passphrase.as_bytes().to_vec()));
    let payload = xdr::TransactionSignaturePayload {
        network_id,
        tagged_transaction,
    };
    let tx_bytes = payload.to_xdr()?;

    // hash it
    Ok(hash_bytes(tx_bytes))
}

fn hash_bytes(b: Vec<u8>) -> [u8; 32] {
    let mut output: [u8; 32] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    let mut hasher = Sha256::new();
    hasher.update(b);
    output.copy_from_slice(&hasher.finalize());
    output
}

fn get_health() -> Value {
    json!({
        "status": "healthy",
    })
}

async fn get_transaction_status(
    transaction_status_map: &Mutex<HashMap<String, Value>>,
    b: Box<[String]>,
) -> Result<Value, Error> {
    if let Some(hash) = b.into_vec().first() {
        let m = transaction_status_map.lock().await;
        let status = m.get(hash);
        Ok(match status {
            Some(status) => status.clone(),
            None => json!({
                "error": {
                    "code":404,
                    "message": "Transaction not found",
                },
            }),
        })
    } else {
        Err(Error::Xdr(XdrError::Invalid))
    }
}

fn simulate_transaction(ledger_file: &PathBuf, b: Box<[String]>) -> Result<Value, Error> {
    if let Some(txn_xdr) = b.into_vec().first() {
        parse_transaction(txn_xdr, SANDBOX_NETWORK_PASSPHRASE)
            // Execute and do NOT commit
            .and_then(|(source_account, _, args)| {
                execute_transaction(source_account, &args, ledger_file, false)
            })
    } else {
        Err(Error::Xdr(XdrError::Invalid))
    }
}

async fn send_transaction(
    ledger_file: &PathBuf,
    transaction_status_map: &Mutex<HashMap<String, Value>>,
    b: Box<[String]>,
) -> Result<Value, Error> {
    if let Some(txn_xdr) = b.into_vec().first() {
        // TODO: Format error object output if txn is invalid
        let mut m = transaction_status_map.lock().await;
        parse_transaction(txn_xdr, SANDBOX_NETWORK_PASSPHRASE).map(
            |(source_account, hash, args)| {
                let id = hex::encode(hash);
                // Execute and commit
                let result = execute_transaction(source_account, &args, ledger_file, true);
                // Add it to our status tracker
                m.insert(
                    id.clone(),
                    match result {
                        Ok(result) => {
                            if let Value::Object(mut o) = result {
                                o.insert("id".to_string(), Value::String(id.to_string()));
                                o.insert(
                                    "status".to_string(),
                                    Value::String("success".to_string()),
                                );
                                Value::Object(o)
                            } else {
                                panic!("Expected object");
                            }
                        }
                        Err(err) => {
                            eprintln!("error: {err:?}");
                            json!({
                                "id": id,
                                "status": "error",
                                "error": {
                                    "code":-32603,
                                    "message": "Internal server error",
                                },
                            })
                        }
                    },
                );
                // Return the hash
                json!({ "id": id, "status": "pending" })
            },
        )
    } else {
        Err(Error::Xdr(XdrError::Invalid))
    }
}

fn parse_op_source_account(transaction: &TransactionEnvelope, op: &Operation) -> AccountId {
    if let Some(source_account) = &op.source_account {
        parse_muxed_account(source_account)
    } else {
        match transaction {
            TransactionEnvelope::TxV0(envelope) => AccountId(PublicKey::PublicKeyTypeEd25519(
                envelope.tx.source_account_ed25519.clone(),
            )),
            TransactionEnvelope::Tx(envelope) => parse_muxed_account(&envelope.tx.source_account),
            TransactionEnvelope::TxFeeBump(envelope) => {
                let FeeBumpTransactionInnerTx::Tx(tx_envelope) = &envelope.tx.inner_tx;
                parse_muxed_account(&tx_envelope.tx.source_account)
            }
        }
    }
}

fn parse_muxed_account(muxed_account: &MuxedAccount) -> AccountId {
    AccountId(PublicKey::PublicKeyTypeEd25519(match muxed_account {
        xdr::MuxedAccount::Ed25519(a) => a.clone(),
        xdr::MuxedAccount::MuxedEd25519(a) => a.ed25519.clone(),
    }))
}
