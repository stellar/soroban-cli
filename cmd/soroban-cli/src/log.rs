use crate::xdr;
pub mod auth;
pub mod budget;
pub mod contract_event;
pub mod cost;
pub mod diagnostic_event;
pub mod footprint;
pub mod host_event;
#[allow(clippy::module_name_repetitions)]
pub mod log_event;

pub use auth::*;
pub use budget::*;
pub use contract_event::*;
pub use cost::*;
pub use diagnostic_event::*;
pub use footprint::*;
pub use host_event::*;
pub use log_event::*;

pub fn events(events: &[xdr::DiagnosticEvent]) {
    let (contract_events, other_events): (Vec<_>, Vec<_>) =
        events.iter().partition(|e| is_contract_event(e));
    contract_event::contract_events(&contract_events, tracing::Level::INFO);
    let (log_events, other_events): (Vec<_>, Vec<_>) =
        other_events.into_iter().partition(|e| is_log_event(e));
    log_event::log_events(&log_events, tracing::Level::INFO);
    diagnostic_event::diagnostic_events(&other_events, tracing::Level::DEBUG);
}

pub fn extract_events(tx_meta: &xdr::TransactionMeta) -> Vec<xdr::DiagnosticEvent> {
    match tx_meta {
        xdr::TransactionMeta::V3(xdr::TransactionMetaV3 {
            soroban_meta: Some(meta),
            ..
        }) => meta.diagnostic_events.to_vec(),
        _ => Vec::new(),
    }
}

fn is_contract_event(event: &xdr::DiagnosticEvent) -> bool {
    matches!(event.event.type_, xdr::ContractEventType::Contract)
}

fn is_log_event(event: &xdr::DiagnosticEvent) -> bool {
    match &event.event.body {
        xdr::ContractEventBody::V0(xdr::ContractEventV0 { topics, .. }) if topics.len() == 1 => {
            topics[0] == xdr::ScVal::Symbol(str_to_sc_string("log"))
        }
        xdr::ContractEventBody::V0(_) => false,
    }
}

fn str_to_sc_string(s: &str) -> xdr::ScSymbol {
    let inner: xdr::StringM<32> = s.try_into().unwrap();
    xdr::ScSymbol(inner)
}
