use crate::rpc::{GetTransactionResponse, GetTransactionResponseRaw, SimulateTransactionResponse};
use directories::ProjectDirs;
use http::Uri;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::xdr::{self, WriteXdr};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to find project directories")]
    FiledToFindProjectDirs,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Http(#[from] http::uri::InvalidUri),
    #[error(transparent)]
    Ulid(#[from] ulid::DecodeError),
}

pub const XDG_DATA_HOME: &str = "XDG_DATA_HOME";

pub fn project_dir() -> Result<directories::ProjectDirs, Error> {
    std::env::var(XDG_DATA_HOME)
        .map_or_else(
            |_| ProjectDirs::from("com", "stellar", "soroban-cli"),
            |data_home| ProjectDirs::from_path(std::path::PathBuf::from(data_home)),
        )
        .ok_or(Error::FiledToFindProjectDirs)
}

pub fn actions_dir() -> Result<std::path::PathBuf, Error> {
    let dir = project_dir()?.data_local_dir().join("actions");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn write(action: Action, rpc_url: Uri) -> Result<ulid::Ulid, Error> {
    let data = Data {
        action,
        rpc_url: rpc_url.to_string(),
    };
    let id = ulid::Ulid::new();
    let file = actions_dir()?.join(id.to_string()).with_extension("json");
    std::fs::write(file, serde_json::to_string(&data)?)?;
    Ok(id)
}

pub fn read(id: &ulid::Ulid) -> Result<(Action, Uri), Error> {
    let file = actions_dir()?.join(id.to_string()).with_extension("json");
    let data: Data = serde_json::from_str(&std::fs::read_to_string(file)?)?;
    Ok((data.action, http::Uri::from_str(&data.rpc_url)?))
}

pub fn list_ulids() -> Result<Vec<ulid::Ulid>, Error> {
    let dir = actions_dir()?;
    let mut list = std::fs::read_dir(dir)?
        .map(|entry| {
            entry
                .map(|e| e.file_name().into_string().unwrap())
                .map_err(Error::from)
        })
        .collect::<Result<Vec<String>, Error>>()?;
    list.sort();
    Ok(list
        .iter()
        .map(|s|ulid::Ulid::from_str(s))
        .collect::<Result<Vec<_>, _>>()?)
}

pub fn list_actions() -> Result<Vec<(ulid::Ulid, Action, Uri)>, Error> {
    list_ulids()?.into_iter()
        .map(|id| {
            let (action, uri) = read(&id)?;
            Ok((id, action, uri))
        })
        .collect::<Result<Vec<_>,Error>>()
}

#[derive(Serialize, Deserialize)]
struct Data {
    action: Action,
    rpc_url: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Action {
    Simulation(SimulateTransactionResponse),
    Transaction(GetTransactionResponseRaw),
}

impl From<SimulateTransactionResponse> for Action {
    fn from(res: SimulateTransactionResponse) -> Self {
        Self::Simulation(res)
    }
}

impl TryFrom<GetTransactionResponse> for Action {
    type Error = xdr::Error;
    fn try_from(res: GetTransactionResponse) -> Result<Self, Self::Error> {
        Ok(Self::Transaction(GetTransactionResponseRaw {
            status: res.status,
            envelope_xdr: res.envelope.map(to_xdr).transpose()?,
            result_xdr: res.result.map(to_xdr).transpose()?,
            result_meta_xdr: res.result_meta.map(to_xdr).transpose()?,
        }))
    }
}

fn to_xdr(data: impl WriteXdr) -> Result<String, xdr::Error> {
    data.to_xdr_base64(xdr::Limits::none())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_write_read() {
        let t = assert_fs::TempDir::new().unwrap();
        std::env::set_var(XDG_DATA_HOME, t.path().to_str().unwrap());
        let rpc_uri = http::uri::Uri::from_str("http://localhost:8000").unwrap();
        let sim = SimulateTransactionResponse::default();
        let original_action: Action = sim.into();

        let id = write(original_action.clone(), rpc_uri.clone()).unwrap();
        let (action, new_rpc_uri) = read(&id).unwrap();
        assert_eq!(rpc_uri, new_rpc_uri);
        match (action, original_action) {
            (Action::Simulation(a), Action::Simulation(b)) => {
                assert_eq!(a.cost.cpu_insns, b.cost.cpu_insns);
            }
            _ => panic!("Action mismatch"),
        }
    }
}
