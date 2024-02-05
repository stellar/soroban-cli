pub fn diagnostic_events(events: &[impl std::fmt::Debug], level: tracing::Level) {
    for (i, event) in events.iter().enumerate() {
        if level == tracing::Level::TRACE {
            tracing::trace!("{i}: {event:#?}");
        } else if level == tracing::Level::INFO {
            tracing::info!("{i}: {event:#?}");
        } else if level == tracing::Level::ERROR {
            tracing::error!("{i}: {event:#?}");
        }
    }
}
