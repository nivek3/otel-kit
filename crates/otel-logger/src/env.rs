//! Environment variable configuration for Logger.

use anyhow::{Context, Result};
use config::{Config, Environment, Source};

use super::config::{Logger, LoggerFileAppender};
use super::guard::LoggerGuard;
use super::init::init_tracing_from_logger;

/// Initialize a Logger from environment variables.
///
/// Returns an error for invalid configuration. If any file-appender variables
/// are present, they must include a valid `ENABLE` value. Unset file format and
/// level overrides remain `None` until output layers are created.
pub fn init_logger_from_env(prefix: Option<&str>) -> Result<Logger> {
    let prefix = prefix.unwrap_or("LOG");
    let file_prefix = format!("{prefix}_FILE");

    let config = build_env_config(prefix)?;
    let mut logger: Logger = config
        .try_deserialize()
        .context("Failed to deserialize environment variables")?;

    if let Some(file_appender) = load_file_appender_from_env(&file_prefix)? {
        logger = logger.with_file_appender(Some(file_appender));
    }

    Ok(logger)
}

/// Initialize tracing from environment variables
pub fn init_logging_from_env(prefix: Option<&str>) -> Result<LoggerGuard> {
    let logger = init_logger_from_env(prefix)?;
    init_tracing_from_logger(logger)
}

fn build_env_config(prefix: &str) -> Result<Config> {
    let env_source = Environment::with_prefix(prefix).try_parsing(true);

    Config::builder()
        .add_source(env_source)
        .build()
        .context("Failed to read environment configuration")
}

fn load_file_appender_from_env(prefix: &str) -> Result<Option<LoggerFileAppender>> {
    let config = build_env_config(prefix)?;
    if config.collect()?.is_empty() {
        return Ok(None);
    }
    config
        .try_deserialize()
        .map(Some)
        .with_context(|| format!("Invalid file logging configuration ({prefix})"))
}
