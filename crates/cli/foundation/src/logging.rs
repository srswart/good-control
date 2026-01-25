//! CLI logging/tracing initialization.

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize logging for CLI with the given log level.
///
/// # Arguments
///
/// * `log_level` - The log level filter string (e.g., "info", "debug", "trace").
///
/// # Panics
///
/// Panics if tracing has already been initialized.
pub fn init_logging(log_level: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_level(true)
                .compact(),
        )
        .init();

    tracing::debug!("Logging initialized with level: {}", log_level);
}

/// Initialize logging for tests (only once).
#[cfg(test)]
pub fn init_test_logging() {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let filter = EnvFilter::new("debug");
        let _ = tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_test_writer())
            .try_init();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_test_logging_does_not_panic() {
        // Should not panic even when called multiple times
        init_test_logging();
        init_test_logging();
    }
}

