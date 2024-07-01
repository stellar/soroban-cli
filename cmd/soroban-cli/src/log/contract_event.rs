pub fn contract_events(events: &[impl std::fmt::Debug], level: tracing::Level) {
    for (i, event) in events.iter().enumerate() {
        match level {
            tracing::Level::TRACE => {
                tracing::trace!("{i}: {event:#?}");
            }
            tracing::Level::INFO => {
                tracing::info!("{i}: {event:#?}");
            }
            tracing::Level::ERROR => {
                tracing::error!("{i}: {event:#?}");
            }
            tracing::Level::DEBUG => {
                tracing::debug!("{i}: {event:#?}");
            }
            _ => {}
        }
    }
}
