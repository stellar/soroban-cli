use chrono::{DateTime, NaiveDateTime, Utc};
use clap::Parser;
use itertools::Itertools;
use std::{fs, io, path};
use termcolor::{Color, ColorChoice, StandardStream, WriteColor};
use termcolor_output::colored;

use soroban_env_host::{
    events::{self},
    xdr::{self, ReadXdr, WriteXdr},
};

use crate::{rpc, toid, utils};
use crate::{HEADING_RPC, HEADING_SANDBOX};

#[derive(Parser, Debug)]
#[clap()]
pub struct Cmd {
    /// The first ledger sequence number in the range to pull events (required
    /// if not in sandbox mode).
    /// https://developers.stellar.org/docs/encyclopedia/ledger-headers#ledger-sequence
    #[clap(long, conflicts_with = "cursor", required_unless_present = "cursor")]
    start_ledger: Option<u32>,

    /// The cursor corresponding to the start of the event range.
    #[clap(
        long,
        conflicts_with = "start-ledger",
        required_unless_present = "start-ledger"
    )]
    cursor: Option<String>,

    /// Output formatting options for event stream
    #[clap(long, arg_enum, default_value = "pretty")]
    output: OutputFormat,

    /// The maximum number of events to display (specify "0" to show all events
    /// when using sandbox, or to defer to the server-defined limit if using
    /// RPC).
    #[clap(short, long, default_value = "10")]
    count: usize,

    /// RPC server endpoint
    #[clap(long,
        env = "SOROBAN_RPC_URL",
        help_heading = HEADING_RPC,
        conflicts_with = "events-file",
        required_unless_present="events-file",
    )]
    rpc_url: Option<String>,

    /// Local event store (likely generated by `invoke`) to pull events from
    #[clap(
        long,
        parse(from_os_str),
        value_name = "PATH",
        env = "SOROBAN_EVENTS_FILE",
        help_heading = HEADING_SANDBOX,
        conflicts_with = "rpc-url",
        required_unless_present="rpc-url",
    )]
    events_file: Option<std::path::PathBuf>,

    /// A set of (up to 5) contract IDs to filter events on. This parameter can
    /// be passed multiple times, e.g. `--id abc --id def`, or passed with
    /// multiple parameters, e.g. `--id abd def`.
    ///
    /// Though the specification supports multiple filter objects (i.e.
    /// combinations of type, IDs, and topics), only one set can be specified on
    /// the command-line today, though that set can have multiple IDs/topics.
    #[clap(long = "id", multiple = true, max_values(5), help_heading = "FILTERS")]
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
    #[clap(
        long = "topic",
        multiple = true,
        max_values(5),
        help_heading = "FILTERS"
    )]
    topic_filters: Vec<String>,

    /// Specifies which type of contract events to display.
    #[clap(
        long = "type",
        arg_enum,
        default_value = "all",
        help_heading = "FILTERS"
    )]
    event_type: rpc::EventType,
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
        error: hex::FromHexError,
    },

    #[error("invalid JSON string: {error} ({debug})")]
    InvalidJson {
        debug: String,
        error: serde_json::Error,
    },

    #[error("invalid timestamp in event: {ts}")]
    InvalidTimestamp { ts: String },

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
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, clap::ArgEnum)]
pub enum OutputFormat {
    /// Colorful, human-oriented console output
    Pretty,
    /// Human-oriented console output without colors
    Plain,
    /// JSONified console output
    Json,
}

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        let start = match (self.start_ledger, self.cursor.clone()) {
            (Some(start), _) => rpc::EventStart::Ledger(start),
            (_, Some(c)) => rpc::EventStart::Cursor(c),
            // should never happen because of required_unless_present flags
            _ => panic!("missing start_ledger and cursor"),
        };

        // Validate that topics are made up of segments.
        for topic in &self.topic_filters {
            for (i, segment) in topic.split(',').enumerate() {
                if i > 4 {
                    return Err(Error::InvalidTopicFilter {
                        topic: topic.to_string(),
                    });
                }

                if segment != "*" {
                    if let Err(e) = xdr::ScVal::from_xdr_base64(segment) {
                        return Err(Error::InvalidSegment {
                            topic: topic.to_string(),
                            segment: segment.to_string(),
                            error: e,
                        });
                    }
                }
            }
        }

        let response = match (self.rpc_url.as_ref(), self.events_file.as_ref()) {
            (Some(rpc_url), _) => self.run_against_rpc_server(rpc_url, start).await,
            (_, Some(path)) => self.run_in_sandbox(path, start),
            // should never happen because of required_unless_present flags
            _ => panic!("missing target"),
        }?;

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
                OutputFormat::Plain => print_event(event)?,
                OutputFormat::Pretty => pretty_print_event(event)?,
            }
        }
        println!("Latest Ledger: {}", response.latest_ledger);

        Ok(())
    }

    async fn run_against_rpc_server(
        &self,
        rpc_url: &str,
        start: rpc::EventStart,
    ) -> Result<rpc::GetEventsResponse, Error> {
        for raw_contract_id in &self.contract_ids {
            // We parse the contract IDs to ensure they're the correct format,
            // but since we'll be passing them as-is to the RPC server anyway,
            // we disregard the return value.
            utils::id_from_str::<32>(raw_contract_id).map_err(|e| Error::InvalidContractId {
                contract_id: raw_contract_id.clone(),
                error: e,
            })?;
        }

        let client = rpc::Client::new(rpc_url);
        client
            .get_events(
                start,
                Some(self.event_type),
                &self.contract_ids,
                &self.topic_filters,
                Some(self.count),
            )
            .await
            .map_err(|e| Error::Rpc(e))
    }

    fn run_in_sandbox(
        &self,
        path: &path::PathBuf,
        start: rpc::EventStart,
    ) -> Result<rpc::GetEventsResponse, Error> {
        if !path.exists() {
            return Err(Error::InvalidFile {
                path: path.to_str().unwrap().to_string(),
            });
        }

        let count: usize = if self.count == 0 {
            std::usize::MAX
        } else {
            self.count
        };

        let start_cursor = match start {
            rpc::EventStart::Ledger(l) => (toid::Toid::new(l, 0, 0).into(), -1),
            rpc::EventStart::Cursor(c) => parse_cursor(&c)?,
        };

        let file = read(path).map_err(|err| Error::CannotReadFile {
            path: path.to_str().unwrap().to_string(),
            error: err.to_string(),
        })?;

        // Read the JSON events from disk and find the ones that match the
        // contract ID filter(s) that were passed in.
        Ok(rpc::GetEventsResponse {
            events: file
                .events
                .iter()
                .filter(|evt| match parse_cursor(&evt.id) {
                    Ok(event_cursor) => event_cursor > start_cursor,
                    Err(e) => {
                        eprintln!("error parsing key 'ledger': {e:?}");
                        eprintln!(
                            "your sandbox events file ('{}') may be corrupt, consider deleting it",
                            path.to_str().unwrap(),
                        );
                        eprintln!("ignoring this event: {evt:#?}");

                        false
                    }
                })
                .filter(|evt| {
                    // Contract ID filter(s) are optional, so we should render all
                    // events if they're omitted.
                    self.contract_ids.is_empty()
                        || self.contract_ids.iter().any(|id| *id == evt.contract_id)
                })
                .filter(|evt| {
                    // Like before, no topic filters means pass everything through.
                    self.topic_filters.is_empty() ||
                // Reminder: All of the topic filters are part of a single
                // filter object, and each one contains segments, so we need to
                // apply all of them to the given event.
                self.topic_filters
                    .iter()
                    // quadratic, but both are <= 5 long
                    .any(|f| {
                        does_topic_match(
                            &evt.topic,
                            // misc. Rust nonsense: make a copy over the given
                            // split filter, because passing a slice of
                            // references is too much for this language to
                            // handle
                            &f.split(',')
                            .map(std::string::ToString::to_string)
                            .collect::<Vec<String>>()
                        )
                    })
                })
                .take(count)
                .cloned()
                .collect::<Vec<rpc::Event>>(),
            latest_ledger: file.latest_ledger,
        })
    }
}

fn parse_cursor(c: &str) -> Result<(u64, i32), Error> {
    let (toid_part, event_index) = c.split('-').collect_tuple().ok_or(Error::InvalidCursor)?;
    let toid_part: u64 = toid_part.parse().map_err(|_| Error::InvalidCursor)?;
    let start_index: i32 = event_index.parse().map_err(|_| Error::InvalidCursor)?;
    Ok((toid_part, start_index))
}

// Determines whether or not a particular filter matches a topic based on the
// same semantics as the RPC server:
//
//  - for an exact segment match, the filter is a base64-encoded ScVal
//  - for a wildcard, single-segment match, the string "*" matches exactly one
//    segment
//
// The expectation is that a `filter` is a comma-separated list of segments that
// has previously been validated, and `topic` is the list of segments applicable
// for this event.
//
// [API
// Reference](https://docs.google.com/document/d/1TZUDgo_3zPz7TiPMMHVW_mtogjLyPL0plvzGMsxSz6A/edit#bookmark=id.35t97rnag3tx)
// [Code
// Reference](https://github.com/stellar/soroban-tools/blob/bac1be79e8c2590c9c35ad8a0168aab0ae2b4171/cmd/soroban-rpc/internal/methods/get_events.go#L182-L203)
pub fn does_topic_match(topic: &[String], filter: &[String]) -> bool {
    filter.len() == topic.len()
        && filter
            .iter()
            .enumerate()
            .all(|(i, s)| *s == "*" || topic[i] == *s)
}

pub fn print_event(event: &rpc::Event) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Event {} [{}]:",
        event.paging_token,
        event.event_type.to_ascii_uppercase()
    );
    println!(
        "  Ledger:   {} (closed at {})",
        event.ledger, event.ledger_closed_at
    );
    println!("  Contract: {}", event.contract_id);
    println!("  Topics:");
    for topic in &event.topic {
        let scval = xdr::ScVal::from_xdr_base64(topic)?;
        println!("            {scval:?}");
    }
    let scval = xdr::ScVal::from_xdr_base64(&event.value.xdr)?;
    println!("  Value:    {scval:?}");

    Ok(())
}

pub fn pretty_print_event(event: &rpc::Event) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    if !stdout.supports_color() {
        print_event(event)?;
        return Ok(());
    }

    let color = match event.event_type.as_str() {
        "system" => Color::Yellow,
        _ => Color::Blue,
    };
    colored!(
        stdout,
        "{}Event{} {}{}{} [{}{}{}{}]:\n",
        bold!(true),
        bold!(false),
        fg!(Some(Color::Green)),
        event.paging_token,
        reset!(),
        bold!(true),
        fg!(Some(color)),
        event.event_type.to_ascii_uppercase(),
        reset!(),
    )?;

    colored!(
        stdout,
        "  Ledger:   {}{}{} (closed at {}{}{})\n",
        fg!(Some(Color::Green)),
        event.ledger,
        reset!(),
        fg!(Some(Color::Green)),
        event.ledger_closed_at,
        reset!(),
    )?;

    colored!(
        stdout,
        "  Contract: {}0x{}{}\n",
        fg!(Some(Color::Green)),
        event.contract_id,
        reset!(),
    )?;

    colored!(stdout, "  Topics:\n")?;
    for topic in &event.topic {
        let scval = xdr::ScVal::from_xdr_base64(topic)?;
        colored!(
            stdout,
            "            {}{:?}{}\n",
            fg!(Some(Color::Green)),
            scval,
            reset!(),
        )?;
    }

    let scval = xdr::ScVal::from_xdr_base64(&event.value.xdr)?;
    colored!(
        stdout,
        "  Value: {}{:?}{}\n",
        fg!(Some(Color::Green)),
        scval,
        reset!(),
    )?;

    Ok(())
}

/// Returns a list of events from the on-disk event store, which stores events
/// exactly as they'd be returned by an RPC server.
pub fn read(path: &std::path::PathBuf) -> Result<rpc::GetEventsResponse, Error> {
    let reader = std::fs::OpenOptions::new().read(true).open(path)?;
    Ok(serde_json::from_reader(reader)?)
}

/// Reads the existing event file, appends the new events, and writes it all to
/// disk. Note that this almost certainly isn't safe to call in parallel.
pub fn commit(
    new_events: &[events::HostEvent],
    ledger_info: &soroban_ledger_snapshot::LedgerSnapshot,
    output_file: &std::path::PathBuf,
) -> Result<(), Error> {
    // Create the directory tree if necessary, since these are unlikely to be
    // the first events.
    if let Some(dir) = output_file.parent() {
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }
    }

    let mut events: Vec<rpc::Event> = if path::Path::exists(output_file) {
        let mut file = fs::OpenOptions::new().read(true).open(output_file)?;
        let payload: rpc::GetEventsResponse = serde_json::from_reader(&mut file)?;
        payload.events
    } else {
        vec![]
    };

    for (i, event) in new_events.iter().enumerate() {
        let contract_event = match event {
            events::HostEvent::Contract(e) => e,
            events::HostEvent::Debug(e) => {
                return Err(Error::Generic(
                    format!("debug events unsupported: {e:#?}").into(),
                ))
            }
        };

        let topics = match &contract_event.body {
            xdr::ContractEventBody::V0(e) => &e.topics,
        }
        .iter()
        .map(xdr::WriteXdr::to_xdr_base64)
        .collect::<Result<Vec<String>, _>>()?;

        // stolen from
        // https://github.com/stellar/soroban-tools/blob/main/cmd/soroban-rpc/internal/methods/get_events.go#L264
        let id = format!(
            "{}-{:010}",
            toid::Toid::new(
                ledger_info.sequence_number,
                // we should technically inject the tx order here from the
                // ledger info, but the sandbox does one tx/op per ledger
                // anyway, so this is a safe assumption
                1,
                1,
            )
            .to_paging_token(),
            i + 1
        );

        // Misc. timestamp to RFC 3339-formatted datetime nonsense, with an
        // absurd amount of verbosity because every edge case needs its own
        // chain of error-handling methods.
        //
        // Reference: https://stackoverflow.com/a/50072164
        let ts: i64 = ledger_info
            .timestamp
            .try_into()
            .map_err(|_e| Error::InvalidTimestamp {
                ts: ledger_info.timestamp.to_string(),
            })?;
        let ndt =
            NaiveDateTime::from_timestamp_opt(ts, 0).ok_or_else(|| Error::InvalidTimestamp {
                ts: ledger_info.timestamp.to_string(),
            })?;

        let dt: DateTime<Utc> = DateTime::from_utc(ndt, Utc);

        let cereal_event = rpc::Event {
            event_type: match contract_event.type_ {
                xdr::ContractEventType::Contract => "contract",
                xdr::ContractEventType::System => "system",
            }
            .to_string(),
            paging_token: id.clone(),
            id,
            ledger: ledger_info.sequence_number.to_string(),
            ledger_closed_at: dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            contract_id: hex::encode(
                contract_event
                    .contract_id
                    .as_ref()
                    .unwrap_or(&xdr::Hash([0; 32])),
            ),
            topic: topics,
            value: rpc::EventValue {
                xdr: match &contract_event.body {
                    xdr::ContractEventBody::V0(e) => &e.data,
                }
                .to_xdr_base64()?,
            },
        };

        events.push(cereal_event);
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output_file)?;

    serde_json::to_writer_pretty(
        &mut file,
        &rpc::GetEventsResponse {
            events,
            latest_ledger: ledger_info.sequence_number,
        },
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use assert_fs::NamedTempFile;

    use super::*;

    #[test]
    // Taken from [RPC server
    // tests](https://github.com/stellar/soroban-tools/blob/main/cmd/soroban-rpc/internal/methods/get_events_test.go#L21).
    fn test_does_topic_match() {
        struct TestCase<'a> {
            name: &'a str,
            filter: Vec<&'a str>,
            includes: Vec<Vec<&'a str>>,
            excludes: Vec<Vec<&'a str>>,
        }

        let xfer = "AAAABQAAAAh0cmFuc2Zlcg==";
        let number = "AAAAAQB6Mcc=";
        let star = "*";

        for tc in vec![
            // No filter means match nothing.
            TestCase {
                name: "<empty>",
                filter: vec![],
                includes: vec![],
                excludes: vec![vec![xfer]],
            },
            // "*" should match "transfer/" but not "transfer/transfer" or
            // "transfer/amount", because * is specified as a SINGLE segment
            // wildcard.
            TestCase {
                name: "*",
                filter: vec![star],
                includes: vec![vec![xfer]],
                excludes: vec![vec![xfer, xfer], vec![xfer, number]],
            },
            // "*/transfer" should match anything preceding "transfer", but
            // nothing that isn't exactly two segments long.
            TestCase {
                name: "*/transfer",
                filter: vec![star, xfer],
                includes: vec![vec![number, xfer], vec![xfer, xfer]],
                excludes: vec![
                    vec![number],
                    vec![number, number],
                    vec![number, xfer, number],
                    vec![xfer],
                    vec![xfer, number],
                    vec![xfer, xfer, xfer],
                ],
            },
            // The inverse case of before: "transfer/*" should match any single
            // segment after a segment that is exactly "transfer", but no
            // additional segments.
            TestCase {
                name: "transfer/*",
                filter: vec![xfer, star],
                includes: vec![vec![xfer, number], vec![xfer, xfer]],
                excludes: vec![
                    vec![number],
                    vec![number, number],
                    vec![number, xfer, number],
                    vec![xfer],
                    vec![number, xfer],
                    vec![xfer, xfer, xfer],
                ],
            },
            // Here, we extend to exactly two wild segments after transfer.
            TestCase {
                name: "transfer/*/*",
                filter: vec![xfer, star, star],
                includes: vec![vec![xfer, number, number], vec![xfer, xfer, xfer]],
                excludes: vec![
                    vec![number],
                    vec![number, number],
                    vec![number, xfer],
                    vec![number, xfer, number, number],
                    vec![xfer],
                    vec![xfer, xfer, xfer, xfer],
                ],
            },
            // Here, we ensure wildcards can be in the middle of a filter: only
            // exact matches happen on the ends, while the middle can be
            // anything.
            TestCase {
                name: "transfer/*/number",
                filter: vec![xfer, star, number],
                includes: vec![vec![xfer, number, number], vec![xfer, xfer, number]],
                excludes: vec![
                    vec![number],
                    vec![number, number],
                    vec![number, number, number],
                    vec![number, xfer, number],
                    vec![xfer],
                    vec![number, xfer],
                    vec![xfer, xfer, xfer],
                    vec![xfer, number, xfer],
                ],
            },
        ] {
            for topic in tc.includes {
                assert!(
                    does_topic_match(
                        &topic
                            .iter()
                            .map(std::string::ToString::to_string)
                            .collect::<Vec<String>>(),
                        &tc.filter
                            .iter()
                            .map(std::string::ToString::to_string)
                            .collect::<Vec<String>>()
                    ),
                    "test: {}, topic ({:?}) should be matched by filter ({:?})",
                    tc.name,
                    topic,
                    tc.filter
                );
            }

            for topic in tc.excludes {
                assert!(
                    !does_topic_match(
                        // make deep copies of the vecs
                        &topic
                            .iter()
                            .map(std::string::ToString::to_string)
                            .collect::<Vec<String>>(),
                        &tc.filter
                            .iter()
                            .map(std::string::ToString::to_string)
                            .collect::<Vec<String>>()
                    ),
                    "test: {}, topic ({:?}) should NOT be matched by filter ({:?})",
                    tc.name,
                    topic,
                    tc.filter
                );
            }
        }
    }

    #[test]
    fn test_does_event_serialization_match() {
        let temp = NamedTempFile::new("events.json").unwrap();

        // Make a couple of fake events with slightly different properties and
        // write them to disk, then read the serialized versions from disk and
        // ensure the properties match.

        let events: Vec<events::HostEvent> = vec![
            events::HostEvent::Contract(xdr::ContractEvent {
                ext: xdr::ExtensionPoint::V0,
                contract_id: Some(xdr::Hash([0; 32])),
                type_: xdr::ContractEventType::Contract,
                body: xdr::ContractEventBody::V0(xdr::ContractEventV0 {
                    topics: xdr::ScVec(vec![].try_into().unwrap()),
                    data: xdr::ScVal::U32(12345),
                }),
            }),
            events::HostEvent::Contract(xdr::ContractEvent {
                ext: xdr::ExtensionPoint::V0,
                contract_id: Some(xdr::Hash([0x1; 32])),
                type_: xdr::ContractEventType::Contract,
                body: xdr::ContractEventBody::V0(xdr::ContractEventV0 {
                    topics: xdr::ScVec(vec![].try_into().unwrap()),
                    data: xdr::ScVal::I32(67890),
                }),
            }),
        ];

        let snapshot = soroban_ledger_snapshot::LedgerSnapshot {
            protocol_version: 1,
            sequence_number: 2, // this is the only value that matters
            timestamp: 3,
            network_id: [0x1; 32],
            base_reserve: 5,
            ledger_entries: vec![],
        };

        commit(&events, &snapshot, &temp.to_path_buf()).unwrap();

        let file = read(&temp.to_path_buf()).unwrap();
        assert_eq!(file.events.len(), 2);
        assert_eq!(file.events[0].ledger, "2");
        assert_eq!(file.events[1].ledger, "2");
        assert_eq!(file.events[0].contract_id, "0".repeat(64));
        assert_eq!(file.events[1].contract_id, "01".repeat(32));
        assert_eq!(file.latest_ledger, 2);
    }
}
