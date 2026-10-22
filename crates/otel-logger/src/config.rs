//! Logger and LoggerFileAppender configuration structures.

use anyhow::Result;
use opentelemetry::KeyValue;
use serde::{Deserialize, Serialize};
use tracing::Level;
use tracing_appender::rolling::Rotation;
use tracing_subscriber::fmt::format::FmtSpan;

use super::deserialize::{
    default, deserialize_attributes, deserialize_level_optional, deserialize_level_required,
    deserialize_log_format, deserialize_log_format_optional, deserialize_span_events,
};
use super::guard::LoggerGuard;
use super::init::init_tracing_from_logger;

#[cfg(feature = "env")]
use super::env::init_logger_from_env;

// ============================================================================
// Log format and rotation enums
// ============================================================================

/// Log output format
#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub enum LogFormat {
    /// Compact single-line format
    #[serde(rename = "compact")]
    #[default]
    Compact,
    /// Pretty multi-line format with colors
    #[serde(rename = "pretty")]
    Pretty,
    /// JSON format for structured logging
    #[serde(rename = "json")]
    Json,
}

/// Log file rotation strategy
#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub enum LogRollingRotation {
    /// Rotate every minute
    #[serde(rename = "minutely")]
    Minutely,
    /// Rotate every hour (default)
    #[serde(rename = "hourly")]
    #[default]
    Hourly,
    /// Rotate daily
    #[serde(rename = "daily")]
    Daily,
    /// Never rotate
    #[serde(rename = "never")]
    Never,
}

// ============================================================================
// Logger configuration
// ============================================================================

/// Configuration for the OpenTelemetry tracing and logging system.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Logger {
    /// The tracer's instrumentation scope name.
    /// Resource `service.name` uses SDK defaults or explicit resource attributes.
    #[serde(default = "default::service_name")]
    pub service_name: String,

    /// The format to use for log output.
    #[serde(
        deserialize_with = "deserialize_log_format",
        default = "LogFormat::default"
    )]
    pub format: LogFormat,

    /// The span events to include in the output.
    #[serde(
        default = "default::span_events",
        deserialize_with = "deserialize_span_events"
    )]
    pub span_events: FmtSpan,

    /// Whether to use ANSI colors in the output.
    #[serde(default)]
    pub ansi: bool,

    /// The minimum log level to record.
    #[serde(
        deserialize_with = "deserialize_level_required",
        default = "default::log_level"
    )]
    pub level: Level,

    /// Additional attributes to add to the resource.
    #[serde(default, deserialize_with = "deserialize_attributes")]
    pub attributes: Vec<KeyValue>,

    /// Whether to enable console output.
    #[serde(default = "default::console_enabled")]
    pub console_enabled: bool,

    /// Set this if you want to write log to file
    #[serde(default)]
    pub file_appender: Option<LoggerFileAppender>,

    /// Set this if you want to write log to OpenTelemetry
    #[serde(default)]
    pub otel_logs_enabled: bool,
}

// ============================================================================
// LoggerFileAppender configuration
// ============================================================================

/// Configuration for file-based log appender.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct LoggerFileAppender {
    /// Enable logger file appender
    pub enable: bool,

    /// Enable write log to file non-blocking
    #[serde(default)]
    pub non_blocking: bool,

    /// An optional file-only level filter that further restricts the global filter.
    /// When absent, only the global `RUST_LOG` or Logger level filter applies.
    #[serde(default, deserialize_with = "deserialize_level_optional")]
    pub level: Option<Level>,

    /// Set the logger file appender ansi.
    #[serde(default)]
    pub ansi: bool,

    /// File output format; when absent, inherits Logger's format at initialization.
    #[serde(default, deserialize_with = "deserialize_log_format_optional")]
    pub format: Option<LogFormat>,

    /// Set the logger file appender rotation.
    #[serde(default = "default::rotation")]
    pub rotation: LogRollingRotation,

    /// Set the logger file appender dir
    #[serde(default)]
    pub dir: Option<String>,

    /// Set log filename prefix
    #[serde(default)]
    pub filename_prefix: Option<String>,

    /// Set log filename suffix
    #[serde(default)]
    pub filename_suffix: Option<String>,

    /// Set the logger file appender keep max log files.
    #[serde(default = "default::max_log_files")]
    pub max_log_files: usize,
}

impl LoggerFileAppender {
    /// Inherit `level` and `format` from Logger if not set in FileAppender.
    pub fn merge_with_logger(&self, logger: &Logger) -> LoggerFileAppender {
        LoggerFileAppender {
            level: self.level.or(Some(logger.level)),
            format: self.format.or(Some(logger.format)),
            ..self.clone()
        }
    }

    /// Get directory or default value
    pub fn dir_or_default(&self) -> String {
        self.dir.clone().unwrap_or_else(default::dir)
    }

    /// Get filename prefix or default value
    pub fn filename_prefix_or_default(&self) -> String {
        self.filename_prefix
            .clone()
            .unwrap_or_else(default::filename_prefix)
    }

    /// Get filename suffix or default value
    pub fn filename_suffix_or_default(&self) -> String {
        self.filename_suffix
            .clone()
            .unwrap_or_else(default::filename_suffix)
    }

    /// Get format or default value
    pub fn format_or_default(&self) -> LogFormat {
        self.format.unwrap_or(LogFormat::Compact)
    }

    /// Get rolling rotation from LogRollingRotation
    pub fn get_rolling_rotation(&self) -> Rotation {
        match self.rotation {
            LogRollingRotation::Minutely => Rotation::MINUTELY,
            LogRollingRotation::Hourly => Rotation::HOURLY,
            LogRollingRotation::Daily => Rotation::DAILY,
            LogRollingRotation::Never => Rotation::NEVER,
        }
    }
}

// ============================================================================
// Logger implementation
// ============================================================================

impl Default for Logger {
    fn default() -> Self {
        Self {
            service_name: default::service_name(),
            format: LogFormat::default(),
            span_events: default::span_events(),
            ansi: true,
            level: default::log_level(),
            attributes: vec![],
            console_enabled: default::console_enabled(),
            file_appender: None,
            otel_logs_enabled: false,
        }
    }
}

impl Logger {
    /// Create a new configuration with the given tracer instrumentation scope name.
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            ..Default::default()
        }
    }

    /// Set the tracer's instrumentation scope name.
    /// This does not set the resource's `service.name`.
    pub fn with_service_name(mut self, service_name: impl Into<String>) -> Self {
        self.service_name = service_name.into();
        self
    }

    /// Set the log format (compact, pretty, or json).
    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Set the span events to include in the output.
    pub fn with_span_events(mut self, span_events: FmtSpan) -> Self {
        self.span_events = span_events;
        self
    }

    /// Set whether to use ANSI colors in the output.
    pub fn with_ansi(mut self, ansi: bool) -> Self {
        self.ansi = ansi;
        self
    }

    /// Set the minimum log level to record.
    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Add custom attributes to the resource.
    pub fn with_attributes(mut self, attributes: Vec<KeyValue>) -> Self {
        self.attributes = attributes;
        self
    }

    /// Set whether to enable console output.
    pub fn with_console_enabled(mut self, enabled: bool) -> Self {
        self.console_enabled = enabled;
        self
    }

    /// Set file appender configuration.
    pub fn with_file_appender(mut self, file_appender: Option<LoggerFileAppender>) -> Self {
        self.file_appender = file_appender;
        self
    }

    /// Initialize tracing with this configuration.
    pub fn init(self) -> Result<LoggerGuard> {
        init_tracing_from_logger(self)
    }

    /// Initialize the logger from environment variables.
    #[cfg(feature = "env")]
    pub fn from_env(prefix: Option<&str>) -> Result<Self> {
        init_logger_from_env(prefix)
    }
}
