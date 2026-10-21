//! Environment variable configuration for Logger.

use anyhow::{Context, Result};
use config::{Config, Environment};

use super::config::{Logger, LoggerFileAppender};
use super::guard::LoggerGuard;
use super::init::init_tracing_from_logger;

/// Initialize a Logger from environment variables
pub fn init_logger_from_env(prefix: Option<&str>) -> Result<Logger> {
    let prefix = prefix.unwrap_or("LOG");
    let file_prefix = format!("{prefix}_FILE");

    let config = build_env_config(prefix)?;
    let mut logger: Logger = config
        .try_deserialize()
        .context("Failed to deserialize environment variables")?;

    if let Some(file_appender) = load_file_appender_from_env(&file_prefix) {
        let merged_file_appender = file_appender.merge_with_logger(&logger);
        logger = logger.with_file_appender(Some(merged_file_appender));
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

fn load_file_appender_from_env(prefix: &str) -> Option<LoggerFileAppender> {
    build_env_config(prefix).ok()?.try_deserialize().ok()
}
