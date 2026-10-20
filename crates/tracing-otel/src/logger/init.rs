//! Logger initialization functions.

use anyhow::{Context, Result};

use super::config::Logger;
use super::{
    guard::LoggerGuard,
    subscriber::{OutputLayers, create_output_layers, setup_tracing},
};

/// Initialize tracing from a Logger configuration
pub fn init_tracing_from_logger(logger: Logger) -> Result<LoggerGuard> {
    let OutputLayers {
        layers,
        worker_guard,
    } = create_output_layers(&logger)?;
    let otel_guard = setup_tracing(
        &logger.service_name,
        &logger.attributes,
        logger.level,
        layers,
        logger.otel_logs_enabled,
    )
    .context("Failed to initialize tracing")?;
    Ok(LoggerGuard::new(otel_guard, worker_guard))
}

/// Initialize tracing with default settings and the given instrumentation scope name.
pub fn init_logging(service_name: &str) -> Result<LoggerGuard> {
    let logger = Logger::new(service_name);
    init_tracing_from_logger(logger)
}
