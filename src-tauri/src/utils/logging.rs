use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize logging with tracing
///
/// This function sets up the tracing subscriber with the following configuration:
/// - Reads filter from RUST_LOG environment variable if available
/// - Falls back to "raflow=debug,warn" if RUST_LOG is not set
/// - Uses a formatted output layer
///
/// # Example
///
/// ```no_run
/// use raflow_lib::utils::logging::init_logging;
///
/// init_logging();
/// ```
pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("raflow=debug,warn"));

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    tracing::info!("RaFlow logging initialized");
}
