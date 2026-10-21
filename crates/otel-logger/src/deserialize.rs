//! Serde deserialization helpers and default values for Logger configuration.

use opentelemetry::KeyValue;
use serde::{Deserialize, de::IntoDeserializer};
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

use super::config::{LogFormat, LogRollingRotation};

/// Deserialize LogFormat from string
pub fn deserialize_log_format<'de, D>(deserializer: D) -> Result<LogFormat, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    LogFormat::deserialize(s.to_lowercase().trim().into_deserializer())
}

/// Deserialize an optional LogFormat field
pub fn deserialize_log_format_optional<'de, D>(
    deserializer: D,
) -> Result<Option<LogFormat>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.trim().is_empty() {
        return Ok(None);
    }
    deserialize_log_format(s.into_deserializer()).map(Some)
}

/// Deserialize a required Level field
pub fn deserialize_level_required<'de, D>(deserializer: D) -> Result<Level, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

/// Deserialize an optional Level field
pub fn deserialize_level_optional<'de, D>(deserializer: D) -> Result<Option<Level>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.trim().is_empty() {
        return Ok(None);
    }
    s.parse().map(Some).map_err(serde::de::Error::custom)
}

/// Deserialize attributes from string format "key=value,key2=value2"
pub fn deserialize_attributes<'de, D>(deserializer: D) -> Result<Vec<KeyValue>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(Vec::new());
    }

    s.split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| {
            let s = s.trim();
            let (key, value) = s
                .split_once('=')
                .ok_or_else(|| serde::de::Error::custom(format!("Invalid attribute: '{s}'")))?;

            let key = key.trim();
            let value = value.trim();

            if key.is_empty() || value.is_empty() {
                return Err(serde::de::Error::custom(format!(
                    "Empty key or value: '{s}'"
                )));
            }

            Ok(KeyValue::new(key.to_string(), value.to_string()))
        })
        .collect()
}

/// Deserialize span events from string format like "FMT::NEW|FMT::CLOSE"
pub fn deserialize_span_events<'de, D>(deserializer: D) -> Result<FmtSpan, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let s = s.trim();

    if s.is_empty() {
        return Ok(FmtSpan::NONE);
    }

    let mut result = FmtSpan::NONE;

    for part in s.split('|').map(|p| p.trim()) {
        let span = match part {
            "FMT::NEW" | "FmtSpan::NEW" => FmtSpan::NEW,
            "FMT::ENTER" | "FmtSpan::ENTER" => FmtSpan::ENTER,
            "FMT::EXIT" | "FmtSpan::EXIT" => FmtSpan::EXIT,
            "FMT::CLOSE" | "FmtSpan::CLOSE" => FmtSpan::CLOSE,
            "FMT::NONE" | "FmtSpan::NONE" => FmtSpan::NONE,
            "FMT::ACTIVE" | "FmtSpan::ACTIVE" => FmtSpan::ACTIVE,
            "FMT::FULL" | "FmtSpan::FULL" => return Ok(FmtSpan::FULL),
            _ => {
                return Err(serde::de::Error::custom(format!(
                    "Invalid span events: '{part}'. Valid options: FMT::NEW, FMT::ENTER, FMT::EXIT, FMT::CLOSE, FMT::NONE, FMT::ACTIVE, FMT::FULL"
                )));
            }
        };
        result |= span;
    }

    Ok(result)
}

/// Default values for Logger configuration
pub mod default {
    use super::LogRollingRotation;
    use tracing::Level;
    use tracing_subscriber::fmt::format::FmtSpan;

    /// Returns the default instrumentation scope name, `otel_logger`.
    pub fn service_name() -> String {
        env!("CARGO_CRATE_NAME").to_string()
    }

    /// Returns the default maximum number of retained log files: 5.
    pub fn max_log_files() -> usize {
        5
    }

    /// Returns the default minimum log level: `INFO`.
    pub fn log_level() -> Level {
        Level::INFO
    }

    /// Returns the default log directory: `./logs`.
    pub fn dir() -> String {
        "./logs".to_string()
    }

    /// Returns the default log filename prefix: `combine`.
    pub fn filename_prefix() -> String {
        "combine".to_string()
    }

    /// Returns the default log filename suffix: `log`.
    pub fn filename_suffix() -> String {
        "log".to_string()
    }

    /// Returns the default span events: creation and closure.
    pub fn span_events() -> FmtSpan {
        FmtSpan::NEW | FmtSpan::CLOSE
    }

    /// Returns the default log file rotation: hourly.
    pub fn rotation() -> LogRollingRotation {
        LogRollingRotation::Hourly
    }

    /// Returns whether console logging is enabled by default: `true`.
    pub fn console_enabled() -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::IntoDeserializer;
    type StrDeserializer<'a> = serde::de::value::StrDeserializer<'a, serde::de::value::Error>;

    #[test]
    fn test_parse_log_format() {
        assert_eq!(
            deserialize_log_format::<StrDeserializer>("compact".into_deserializer()).unwrap(),
            LogFormat::Compact
        );
        assert_eq!(
            deserialize_log_format::<StrDeserializer>("pretty".into_deserializer()).unwrap(),
            LogFormat::Pretty
        );
        assert_eq!(
            deserialize_log_format::<StrDeserializer>("json".into_deserializer()).unwrap(),
            LogFormat::Json
        );
    }

    #[test]
    fn test_log_format_compatibility() {
        for (input, expected) in [
            (" COMPACT ", LogFormat::Compact),
            ("Pretty", LogFormat::Pretty),
            ("\tJSON\n", LogFormat::Json),
        ] {
            assert_eq!(
                deserialize_log_format::<StrDeserializer>(input.into_deserializer()).unwrap(),
                expected,
            );
            assert_eq!(
                deserialize_log_format_optional::<StrDeserializer>(input.into_deserializer())
                    .unwrap(),
                Some(expected),
            );
        }
        assert_eq!(
            deserialize_log_format_optional::<StrDeserializer>("  ".into_deserializer()).unwrap(),
            None,
        );
        for input in ["", "invalid"] {
            assert!(deserialize_log_format::<StrDeserializer>(input.into_deserializer()).is_err());
        }
        assert!(
            deserialize_log_format_optional::<StrDeserializer>("invalid".into_deserializer())
                .is_err(),
        );
    }

    #[test]
    fn test_parse_attributes() {
        assert_eq!(
            deserialize_attributes::<StrDeserializer>("".into_deserializer()).unwrap(),
            vec![]
        );

        let attrs = deserialize_attributes::<StrDeserializer>(
            "key1=value1,key2=value2".into_deserializer(),
        )
        .unwrap();
        assert_eq!(attrs.len(), 2);
    }
}
