use clap::{arg, command, Parser};
use std::io;

use soroban_env_host::xdr::{self, Limits, ReadXdr};

use config::{locator, network};

use super::{global, NetworkRunnable};
use crate::{rpc, utils};

#[derive(Parser, Debug, Clone)]
#[group(skip)]
pub struct Cmd {
    /// The first ledger sequence number in the range to pull events
    /// https://developers.stellar.org/docs/encyclopedia/ledger-headers#ledger-sequence
    #[arg(long, conflicts_with = "cursor", required_unless_present = "cursor")]
    start_ledger: Option<u32>,
    /// The cursor corresponding to the start of the event range.
    #[arg(
        long,
        conflicts_with = "start_ledger",
        required_unless_present = "start_ledger"
    )]
    cursor: Option<String>,
    /// Output formatting options for event stream
    #[arg(long, value_enum, default_value = "pretty")]
    output: OutputFormat,
    /// The maximum number of events to display (defer to the server-defined limit).
    #[arg(short, long, default_value = "10")]
    count: usize,
    /// A set of (up to 5) contract IDs to filter events on. This parameter can
    /// be passed multiple times, e.g. `--id C123.. --id C456..`, or passed with
    /// multiple parameters, e.g. `--id C123 C456`.
    ///
    /// Though the specification supports multiple filter objects (i.e.
    /// combinations of type, IDs, and topics), only one set can be specified on
    /// the command-line today, though that set can have multiple IDs/topics.
    #[arg(
        long = "id",
        num_args = 1..=6,
        help_heading = "FILTERS"
    )]
    contract_ids: Vec<String>,
    /// A set of (up to 4) topic filters to filter event topics on. A single
    /// topic filter can contain 1-4 different segment filters, separated by
    /// commas, with an asterisk (* character) indicating a wildcard segment.
    ///
    /// For example, this is one topic filter with two segments:
    ///
    ///     --topic "AAAABQAAAAdDT1VOVEVSAA==,*"
    ///
    /// This is two topic filters with one and two segments each:
    ///
    ///     --topic "AAAABQAAAAdDT1VOVEVSAA==" --topic '*,*'
    ///
    /// Note that all of these topic filters are combined with the contract IDs
    /// into a single filter (i.e. combination of type, IDs, and topics).
    #[arg(
        long = "topic",
        num_args = 1..=5,
        help_heading = "FILTERS"
    )]
    topic_filters: Vec<String>,
    /// Specifies which type of contract events to display.
    #[arg(
        long = "type",
        value_enum,
        default_value = "all",
        help_heading = "FILTERS"
    )]
    event_type: rpc::EventType,
    #[command(flatten)]
    locator: locator::Args,
    #[command(flatten)]
    network: network::Args,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("cursor is not valid")]
    InvalidCursor,
    #[error("filepath does not exist: {path}")]
    InvalidFile { path: String },
    #[error("filepath ({path}) cannot be read: {error}")]
    CannotReadFile { path: String, error: String },
    #[error("cannot parse topic filter {topic} into 1-4 segments")]
    InvalidTopicFilter { topic: String },
    #[error("invalid segment ({segment}) in topic filter ({topic}): {error}")]
    InvalidSegment {
        topic: String,
        segment: String,
        error: xdr::Error,
    },
    #[error("cannot parse contract ID {contract_id}: {error}")]
    InvalidContractId {
        contract_id: String,
        error: stellar_strkey::DecodeError,
    },
    #[error("invalid JSON string: {error} ({debug})")]
    InvalidJson {
        debug: String,
        error: serde_json::Error,
    },
    #[error("invalid timestamp in event: {ts}")]
    InvalidTimestamp { ts: String },
    #[error("missing start_ledger and cursor")]
    MissingStartLedgerAndCursor,
    #[error("missing target")]
    MissingTarget,
    #[error(transparent)]
    Rpc(#[from] rpc::Error),
    #[error(transparent)]
    Generic(#[from] Box<dyn std::error::Error>),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Xdr(#[from] xdr::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Network(#[from] network::Error),
    #[error(transparent)]
    Locator(#[from] locator::Error),
    #[error(transparent)]
    Config(#[from] config::Error),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, clap::ValueEnum)]
pub enum OutputFormat {
    /// Colorful, human-oriented console output
    Pretty,
    /// Human-oriented console output without colors
    Plain,
    /// JSONified console output
    Json,
}

impl Cmd {
    pub async fn run(&mut self) -> Result<(), Error> {
        // Validate that topics are made up of segments.
        for topic in &self.topic_filters {
            for (i, segment) in topic.split(',').enumerate() {
                if i > 4 {
                    return Err(Error::InvalidTopicFilter {
                        topic: topic.to_string(),
                    });
                }

                if segment != "*" {
                    if let Err(e) = xdr::ScVal::from_xdr_base64(segment, Limits::none()) {
                        return Err(Error::InvalidSegment {
                            topic: topic.to_string(),
                            segment: segment.to_string(),
                            error: e,
                        });
                    }
                }
            }
        }

        // Validate contract_ids
        for id in &mut self.contract_ids {
            utils::contract_id_from_str(id).map_err(|e| Error::InvalidContractId {
                contract_id: id.clone(),
                error: e,
            })?;
        }

        let response = self.run_against_rpc_server(None, None).await?;

        for event in &response.events {
            match self.output {
                // Should we pretty-print the JSON like we're doing here or just
                // dump an event in raw JSON on each line? The latter is easier
                // to consume programmatically.
                OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&event).map_err(|e| {
                            Error::InvalidJson {
                                debug: format!("{event:#?}"),
                                error: e,
                            }
                        })?,
                    );
                }
                OutputFormat::Plain => println!("{event}"),
                OutputFormat::Pretty => event.pretty_print()?,
            }
        }
        println!("Latest Ledger: {}", response.latest_ledger);

        Ok(())
    }

    fn start(&self) -> Result<rpc::EventStart, Error> {
        let start = match (self.start_ledger, self.cursor.clone()) {
            (Some(start), _) => rpc::EventStart::Ledger(start),
            (_, Some(c)) => rpc::EventStart::Cursor(c),
            // should never happen because of required_unless_present flags
            _ => return Err(Error::MissingStartLedgerAndCursor),
        };
        Ok(start)
    }
}

#[async_trait::async_trait]
impl NetworkRunnable for Cmd {
    type Error = Error;
    type Result = rpc::GetEventsResponse;

    async fn run_against_rpc_server(
        &self,
        _args: Option<&global::Args>,
        config: Option<&config::Args>,
    ) -> Result<rpc::GetEventsResponse, Error> {
        let start = self.start()?;
        let network = if let Some(config) = config {
            Ok(config.get_network()?)
        } else {
            self.network.get(&self.locator)
        }?;

        let client = rpc::Client::new(&network.rpc_url)?;
        client
            .verify_network_passphrase(Some(&network.network_passphrase))
            .await?;
        client
            .get_events(
                start,
                Some(self.event_type),
                &self.contract_ids,
                &self.topic_filters,
                Some(self.count),
            )
            .await
            .map_err(Error::Rpc)
    }
}
