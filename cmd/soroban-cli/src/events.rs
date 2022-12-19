use chrono::{DateTime, NaiveDateTime, Utc};
use clap::Parser;
use std::{fs, io, path};
use termcolor::{Color, ColorChoice, StandardStream, WriteColor};
use termcolor_output::colored;

use soroban_env_host::{
    events,
    xdr::{self, ReadXdr, WriteXdr},
};

use crate::{rpc, toid, utils};
use crate::{HEADING_RPC, HEADING_SANDBOX};

#[derive(Parser, Debug)]
#[clap()]
pub struct Cmd {
    /// The first ledger sequence number in the range to pull events (required
    /// if not in sandbox mode).
    #[clap(short, long, default_value = "0")]
    start_ledger: u32,

    /// The last (and inclusive) ledger sequence number in the range to pull
    /// events (required if not in sandbox mode).
    /// https://developers.stellar.org/docs/encyclopedia/ledger-headers#ledger-sequence
    #[clap(short, long, default_value = "0")]
    end_ledger: u32,

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
    #[error("invalid ledger range: --start-ledger bigger than --end-ledger ({low} > {high})")]
    InvalidLedgerRange { low: u32, high: u32 },

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

    #[error("you must specify either an RPC server or sandbox filepath(s)")]
    TargetRequired,

    #[error("ledger range (-s and -e) is required when specifying an RPC server")]
    LedgerRangeRequired,

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
    pub async fn run(&self, _matches: &clap::ArgMatches) -> Result<(), Error> {
        if self.start_ledger > self.end_ledger {
            return Err(Error::InvalidLedgerRange {
                low: self.start_ledger,
                high: self.end_ledger,
            });
        }

        for raw_contract_id in &self.contract_ids {
            // We parse the contract IDs to ensure they're the correct format,
            // but since we'll be passing them as-is to the RPC server anyway,
            // we disregard the return value.
            utils::id_from_str::<32>(raw_contract_id).map_err(|e| Error::InvalidContractId {
                contract_id: raw_contract_id.clone(),
                error: e,
            })?;
        }

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

        let events = match (self.rpc_url.as_ref(), self.events_file.as_ref()) {
            (Some(rpc_url), _) => self.run_against_rpc_server(rpc_url).await,
            (_, Some(path)) => self.run_in_sandbox(path),
            _ => Err(Error::TargetRequired),
        }?;

        for event in &events {
            match self.output {
                // Should we pretty-print the JSON like we're doing here or just
                // dump an event in raw JSON on each line? The latter is easier
                // to consume programmatically.
                OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&event).map_err(|e| {
                            Error::InvalidJson {
                                debug: format!("{:#?}", event),
                                error: e,
                            }
                        })?,
                    );
                }
                OutputFormat::Plain => print_event(event)?,
                OutputFormat::Pretty => pretty_print_event(event)?,
            }
        }

        Ok(())
    }

    async fn run_against_rpc_server(&self, rpc_url: &str) -> Result<Vec<rpc::Event>, Error> {
        if self.start_ledger == 0 && self.end_ledger == 0 {
            return Err(Error::LedgerRangeRequired);
        }

        let client = rpc::Client::new(rpc_url);
        Ok(client
            .get_events(
                self.start_ledger,
                self.end_ledger,
                Some(self.event_type),
                &self.contract_ids,
                &self.topic_filters,
                Some(self.count),
            )
            .await?
            .unwrap_or_default())
    }

    fn run_in_sandbox(&self, path: &path::PathBuf) -> Result<Vec<rpc::Event>, Error> {
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

        // Read the JSON events from disk and find the ones that match the
        // contract ID filter(s) that were passed in.
        Ok(read_events(path)
            .map_err(|err| Error::CannotReadFile {
                path: path.to_str().unwrap().to_string(),
                error: err.to_string(),
            })?
            .iter()
            // FIXME: We assume here that events are read off-disk in
            // chronological order, but we should probably be sorting by ledger
            // number (and ID, for events within the same ledger), instead,
            // though it's likely that this logic belongs more in
            // `snapshot::read_events()`.
            .rev()
            .filter(|evt| {
                // The ledger range is optional in sandbox mode.
                if self.start_ledger == 0 && self.end_ledger == 0 {
                    return true;
                }

                match evt.ledger.parse::<u32>() {
                    Ok(seq) => seq >= self.start_ledger && seq <= self.end_ledger,
                    Err(e) => {
                        eprintln!("error parsing key 'ledger': {:?}", e);
                        eprintln!(
                            "your sandbox events file ('{}') may be corrupt",
                            path.to_str().unwrap(),
                        );
                        eprintln!("ignoring this event: {:#?}", evt);

                        false
                    }
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
                            &f.split(',').map(|s| s.to_string()).collect::<Vec<String>>()
                        )
                    })
            })
            .take(count)
            .cloned()
            .collect::<Vec<rpc::Event>>())
    }
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
    let mut idx = 0;

    for segment in filter {
        if idx >= topic.len() {
            // Nothing to match, need at least one segment.
            return false;
        }

        if *segment == "*" {
            // One-segment wildcard: ignore this token
        } else if *segment != topic[idx] {
            // Exact match the ScVal (decodability is assumed)
            return false;
        }

        idx += 1;
    }

    // Check we had no leftovers
    idx >= topic.len()
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
        println!("            {:?}", scval);
    }
    let scval = xdr::ScVal::from_xdr_base64(&event.value.xdr)?;
    println!("  Value:    {:?}", scval);

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
pub fn read_events(path: &std::path::PathBuf) -> Result<Vec<rpc::Event>, Error> {
    let reader = std::fs::OpenOptions::new().read(true).open(path)?;
    Ok(serde_json::from_reader(reader)?)
}

/// Reads the existing event file, appends the new events, and writes it all to
/// disk. Note that this almost certainly isn't safe to call in parallel.
pub fn commit_events(
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

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .read(true)
        .open(output_file)?;
    let mut events: rpc::GetEventsResponse = serde_json::from_reader(&mut file)?;

    for (i, event) in new_events.iter().enumerate() {
        let contract_event = match event {
            events::HostEvent::Contract(e) => e,
            events::HostEvent::Debug(_e) => todo!(),
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
            event_type: "contract".to_string(),
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
        .write(true)
        .truncate(true)
        .open(output_file)?;

    serde_json::to_writer_pretty(&mut file, &events)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // Taken from [RPC server
    // tests](https://github.com/stellar/soroban-tools/blob/main/cmd/soroban-rpc/internal/methods/get_events_test.go#L21).
    fn test_does_topic_match() {
        let xfer = "AAAABQAAAAh0cmFuc2Zlcg==";
        let number = "AAAAAQB6Mcc=";
        let star = "*";

        struct TestCase<'a> {
            name: &'a str,
            filter: Vec<String>,
            includes: Vec<Vec<String>>,
            excludes: Vec<Vec<String>>,
        }

        for tc in vec![
            // No filter means match nothing.
            TestCase {
                name: "<empty>",
                filter: vec![],
                includes: vec![],
                excludes: vec![vec![xfer.to_string()]],
            },
            // "*" should match "transfer/" but not "transfer/transfer" or
            // "transfer/amount", because * is specified as a SINGLE segment
            // wildcard.
            TestCase {
                name: "*",
                filter: vec![star.to_string()],
                includes: vec![vec![xfer.to_string()]],
                excludes: vec![
                    vec![xfer.to_string(), xfer.to_string()],
                    vec![xfer.to_string(), number.to_string()],
                ],
            },
            // "*/transfer" should match anything preceding "transfer", but
            // nothing that isn't exactly two segments long.
            TestCase {
                name: "*/transfer",
                filter: vec![star.to_string(), xfer.to_string()],
                includes: vec![
                    vec![number.to_string(), xfer.to_string()],
                    vec![xfer.to_string(), xfer.to_string()],
                ],
                excludes: vec![
                    vec![number.to_string()],
                    vec![number.to_string(), number.to_string()],
                    vec![number.to_string(), xfer.to_string(), number.to_string()],
                    vec![xfer.to_string()],
                    vec![xfer.to_string(), number.to_string()],
                    vec![xfer.to_string(), xfer.to_string(), xfer.to_string()],
                ],
            },
            // The inverse case of before: "transfer/*" should match any single
            // segment after a segment that is exactly "transfer", but no
            // additional segments.
            TestCase {
                name: "transfer/*",
                filter: vec![xfer.to_string(), star.to_string()],
                includes: vec![
                    vec![xfer.to_string(), number.to_string()],
                    vec![xfer.to_string(), xfer.to_string()],
                ],
                excludes: vec![
                    vec![number.to_string()],
                    vec![number.to_string(), number.to_string()],
                    vec![number.to_string(), xfer.to_string(), number.to_string()],
                    vec![xfer.to_string()],
                    vec![number.to_string(), xfer.to_string()],
                    vec![xfer.to_string(), xfer.to_string(), xfer.to_string()],
                ],
            },
            // Here, we extend to exactly two wild segments after transfer.
            TestCase {
                name: "transfer/*/*",
                filter: vec![xfer.to_string(), star.to_string(), star.to_string()],
                includes: vec![
                    vec![xfer.to_string(), number.to_string(), number.to_string()],
                    vec![xfer.to_string(), xfer.to_string(), xfer.to_string()],
                ],
                excludes: vec![
                    vec![number.to_string()],
                    vec![number.to_string(), number.to_string()],
                    vec![number.to_string(), xfer.to_string()],
                    vec![
                        number.to_string(),
                        xfer.to_string(),
                        number.to_string(),
                        number.to_string(),
                    ],
                    vec![xfer.to_string()],
                    vec![
                        xfer.to_string(),
                        xfer.to_string(),
                        xfer.to_string(),
                        xfer.to_string(),
                    ],
                ],
            },
            // Here, we ensure wildcards can be in the middle of a filter: only
            // exact matches happen on the ends, while the middle can be
            // anything.
            TestCase {
                name: "transfer/*/number",
                filter: vec![xfer.to_string(), star.to_string(), number.to_string()],
                includes: vec![
                    vec![xfer.to_string(), number.to_string(), number.to_string()],
                    vec![xfer.to_string(), xfer.to_string(), number.to_string()],
                ],
                excludes: vec![
                    vec![number.to_string()],
                    vec![number.to_string(), number.to_string()],
                    vec![number.to_string(), number.to_string(), number.to_string()],
                    vec![number.to_string(), xfer.to_string(), number.to_string()],
                    vec![xfer.to_string()],
                    vec![number.to_string(), xfer.to_string()],
                    vec![xfer.to_string(), xfer.to_string(), xfer.to_string()],
                    vec![xfer.to_string(), number.to_string(), xfer.to_string()],
                ],
            },
        ] {
            for topic in tc.includes {
                assert!(
                    does_topic_match(&topic, &tc.filter),
                    "test: {}, topic ({:?}) should be matched by filter ({:?})",
                    tc.name,
                    topic,
                    tc.filter
                );
            }

            for topic in tc.excludes {
                assert!(
                    !does_topic_match(&topic, &tc.filter),
                    "test: {}, topic ({:?}) should NOT be matched by filter ({:?})",
                    tc.name,
                    topic,
                    tc.filter
                );
            }
        }
    }
}
