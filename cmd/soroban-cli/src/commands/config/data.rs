use crate::rpc::{GetTransactionResponse, GetTransactionResponseRaw, SimulateTransactionResponse};
use directories::ProjectDirs;
use http::Uri;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::xdr::{self, WriteXdr};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to find project directories")]
    FailedToFindProjectDirs,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Http(#[from] http::uri::InvalidUri),
    #[error(transparent)]
    Ulid(#[from] ulid::DecodeError),
    #[error(transparent)]
    Xdr(#[from] xdr::Error),
}

pub const XDG_DATA_HOME: &str = "XDG_DATA_HOME";

pub fn project_dir() -> Result<directories::ProjectDirs, Error> {
    std::env::var(XDG_DATA_HOME)
        .map_or_else(
            |_| ProjectDirs::from("org", "stellar", "stellar-cli"),
            |data_home| ProjectDirs::from_path(std::path::PathBuf::from(data_home)),
        )
        .ok_or(Error::FailedToFindProjectDirs)
}

#[allow(clippy::module_name_repetitions)]
pub fn data_local_dir() -> Result<std::path::PathBuf, Error> {
    Ok(project_dir()?.data_local_dir().to_path_buf())
}

pub fn actions_dir() -> Result<std::path::PathBuf, Error> {
    let dir = data_local_dir()?.join("actions");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn spec_dir() -> Result<std::path::PathBuf, Error> {
    let dir = data_local_dir()?.join("spec");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn bucket_dir() -> Result<std::path::PathBuf, Error> {
    let dir = data_local_dir()?.join("bucket");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn write(action: Action, rpc_url: &Uri) -> Result<ulid::Ulid, Error> {
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

pub fn write_spec(hash: &str, spec_entries: &[xdr::ScSpecEntry]) -> Result<(), Error> {
    let file = spec_dir()?.join(hash);
    tracing::trace!("writing spec to {:?}", file);
    let mut contents: Vec<u8> = Vec::new();
    for entry in spec_entries {
        contents.extend(entry.to_xdr(xdr::Limits::none())?);
    }
    std::fs::write(file, contents)?;
    Ok(())
}

pub fn read_spec(hash: &str) -> Result<Vec<xdr::ScSpecEntry>, Error> {
    let file = spec_dir()?.join(hash);
    tracing::trace!("reading spec from {:?}", file);
    Ok(soroban_spec::read::parse_raw(&std::fs::read(file)?)?)
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
        .map(|s| ulid::Ulid::from_str(s.trim_end_matches(".json")))
        .collect::<Result<Vec<_>, _>>()?)
}

pub fn list_actions() -> Result<Vec<DatedAction>, Error> {
    list_ulids()?
        .into_iter()
        .rev()
        .map(|id| {
            let (action, uri) = read(&id)?;
            Ok(DatedAction(id, action, uri))
        })
        .collect::<Result<Vec<_>, Error>>()
}

pub struct DatedAction(ulid::Ulid, Action, Uri);

impl std::fmt::Display for DatedAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (id, a, uri) = (&self.0, &self.1, &self.2);
        let datetime = to_datatime(id).format("%b %d %H:%M");
        let status = match a {
            Action::Simulate { response } => response
                .error
                .as_ref()
                .map_or_else(|| "SUCCESS".to_string(), |_| "ERROR".to_string()),
            Action::Send { response } => response.status.to_string(),
        };
        write!(f, "{id} {} {status} {datetime} {uri} ", a.type_str(),)
    }
}

impl DatedAction {}

fn to_datatime(id: &ulid::Ulid) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp_millis(id.timestamp_ms().try_into().unwrap()).unwrap()
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct Data {
    action: Action,
    rpc_url: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Simulate {
        response: SimulateTransactionResponse,
    },
    Send {
        response: GetTransactionResponseRaw,
    },
}

impl Action {
    pub fn type_str(&self) -> String {
        match self {
            Action::Simulate { .. } => "Simulate",
            Action::Send { .. } => "Send    ",
        }
        .to_string()
    }
}

impl From<SimulateTransactionResponse> for Action {
    fn from(response: SimulateTransactionResponse) -> Self {
        Self::Simulate { response }
    }
}

impl TryFrom<GetTransactionResponse> for Action {
    type Error = xdr::Error;
    fn try_from(res: GetTransactionResponse) -> Result<Self, Self::Error> {
        Ok(Self::Send {
            response: GetTransactionResponseRaw {
                status: res.status,
                envelope_xdr: res.envelope.as_ref().map(to_xdr).transpose()?,
                result_xdr: res.result.as_ref().map(to_xdr).transpose()?,
                result_meta_xdr: res.result_meta.as_ref().map(to_xdr).transpose()?,
            },
        })
    }
}

fn to_xdr(data: &impl WriteXdr) -> Result<String, xdr::Error> {
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

        let id = write(original_action.clone(), &rpc_uri.clone()).unwrap();
        let (action, new_rpc_uri) = read(&id).unwrap();
        assert_eq!(rpc_uri, new_rpc_uri);
        match (action, original_action) {
            (Action::Simulate { response: a }, Action::Simulate { response: b }) => {
                assert_eq!(a.cost.cpu_insns, b.cost.cpu_insns);
            }
            _ => panic!("Action mismatch"),
        }
    }
}
