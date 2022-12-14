use clap::Parser;
use std::path;
use termcolor::{Color, ColorChoice, StandardStream, WriteColor};
use termcolor_output::colored;

use soroban_env_host::xdr::{self, ReadXdr};

use crate::rpc::{Client, Event};
use crate::{rpc, snapshot, utils};
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

    /// The maximum number of events to display (0 = all)
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
    /// be passed multiple times, e.g. --id abc --id def, or passed with
    /// multiple parameters, e.g. --id abd def.
    ///
    /// Though the specification supports multiple sets of contract/topic
    /// filters, only one set can be specified on the command-line today.
    #[clap(long = "id", multiple = true, max_values(5), help_heading = "FILTERS")]
    contract_ids: Vec<String>,

    /// A set of (up to 5) topic filters to filter events on. See the help for
    /// --id to understand how to pass multiple and the limitations therein.
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
    #[error("invalid ledger range: low bigger than high ({low} > {high})")]
    InvalidLedgerRange { low: u32, high: u32 },

    #[error("filepath does not exist: {path}")]
    InvalidFile { path: String },

    #[error("filepath ({path}) cannot be read: {error}")]
    CannotReadFile { path: String, error: String },

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

    #[error("you must specify either an RPC server or sandbox filepath(s)")]
    TargetRequired,

    #[error("ledger range is required when specifying an RPC server")]
    LedgerRangeRequired,

    #[error(transparent)]
    Rpc(#[from] rpc::Error),

    #[error(transparent)]
    Generic(#[from] Box<dyn std::error::Error>),
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

        let events = match self.rpc_url.as_ref() {
            Some(rpc_url) => self.run_against_rpc_server(rpc_url).await?,
            _ => match self.events_file.as_ref() {
                Some(path) => self.run_in_sandbox(path)?,
                _ => return Err(Error::TargetRequired),
            },
        };

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

    async fn run_against_rpc_server(&self, rpc_url: &str) -> Result<Vec<Event>, Error> {
        if self.start_ledger == 0 && self.end_ledger == 0 {
            return Err(Error::LedgerRangeRequired);
        }

        let client = Client::new(rpc_url);
        Ok(client
            .get_events(
                self.start_ledger,
                self.end_ledger,
                Some(self.event_type),
                &self.contract_ids,
                &self.topic_filters,
                Some(self.count),
            )
            .await?)
    }

    fn run_in_sandbox(&self, path: &path::PathBuf) -> Result<Vec<Event>, Error> {
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
        Ok(snapshot::read_events(path)
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
            // The ledger range is optional in sandbox mode.
            .filter(|evt| {
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
                self.topic_filters.is_empty()
                    || self
                        .topic_filters
                        .iter()
                        // quadratic but both are <= 5 long
                        .any(|f| does_topic_match(&evt.topic, f))
            })
            .take(count)
            .cloned()
            .collect::<Vec<Event>>())
    }
}

pub fn does_topic_match(topics: &Vec<String>, filter: &str) -> bool {
    // FIXME: Do actual topic matching.
    if filter == "*" || filter == "#" {
        return true;
    }

    for topic in topics {
        if topic == filter {
            return true;
        }
    }

    false
}

pub fn print_event(event: &Event) -> Result<(), Box<dyn std::error::Error>> {
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

pub fn pretty_print_event(event: &Event) -> Result<(), Box<dyn std::error::Error>> {
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
