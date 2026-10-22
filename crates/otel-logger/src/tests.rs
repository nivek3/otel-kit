//! Tests for Logger configuration.

use super::*;
use opentelemetry::KeyValue;
use serial_test::serial;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

use super::deserialize::deserialize_span_events;

#[derive(Debug, serde::Deserialize)]
struct TestFmtSpan {
    #[serde(deserialize_with = "deserialize_span_events")]
    span_events: FmtSpan,
}

#[test]
#[serial]
fn test_logger_builder() {
    let logger = Logger::new("test-service")
        .with_level(Level::DEBUG)
        .with_attributes(vec![KeyValue::new("test", "value")]);

    assert_eq!(logger.service_name, "test-service");
    assert_eq!(logger.level, Level::DEBUG);
    assert_eq!(logger.attributes.len(), 1);
}

#[test]
#[serial]
fn test_deserialize_span_events_fmt_format() {
    let result: TestFmtSpan = serde_json::from_str(r#"{"span_events": "FMT::NEW"}"#).unwrap();
    assert_eq!(result.span_events, FmtSpan::NEW);

    let result: TestFmtSpan = serde_json::from_str(r#"{"span_events": "FMT::CLOSE"}"#).unwrap();
    assert_eq!(result.span_events, FmtSpan::CLOSE);

    let result: TestFmtSpan =
        serde_json::from_str(r#"{"span_events": "FMT::NEW|FMT::CLOSE"}"#).unwrap();
    assert_eq!(result.span_events, FmtSpan::NEW | FmtSpan::CLOSE);
}

#[test]
#[serial]
fn test_deserialize_span_events_empty() {
    let result: TestFmtSpan = serde_json::from_str(r#"{"span_events": ""}"#).unwrap();
    assert_eq!(result.span_events, FmtSpan::NONE);
}

#[test]
#[serial]
fn test_deserialize_span_events_invalid() {
    let result: Result<TestFmtSpan, _> = serde_json::from_str(r#"{"span_events": "INVALID"}"#);
    assert!(result.is_err());
}

#[test]
#[serial]
fn test_logger_with_file_appender() {
    let file_appender = LoggerFileAppender {
        enable: true,
        non_blocking: false,
        rotation: LogRollingRotation::Daily,
        dir: Some("/var/log/test".to_string()),
        filename_prefix: Some("test".to_string()),
        filename_suffix: Some("log".to_string()),
        max_log_files: 10,
    };

    let logger = Logger::new("test-service").with_file_appender(Some(file_appender));

    assert!(logger.file_appender.is_some());
}

#[test]
#[serial]
#[cfg(feature = "env")]
fn test_env_file_appender_parsing() {
    use super::env::init_logger_from_env;

    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("LOG_FILE_ENABLE", "true");
        std::env::set_var("LOG_FORMAT", "json");
    }

    let logger = init_logger_from_env(None).unwrap();
    assert_eq!(logger.format, LogFormat::Json);
    let file_appender = logger.file_appender.expect("file_appender should be Some");
    assert!(file_appender.enable);

    #[allow(unsafe_code)]
    unsafe {
        std::env::remove_var("LOG_FILE_ENABLE");
        std::env::remove_var("LOG_FORMAT");
    }
}

#[test]
#[serial]
fn test_logger_console_control() {
    let logger = Logger::new("test-service");
    assert!(logger.console_enabled);

    let logger = Logger::new("test-service").with_console_enabled(false);
    assert!(!logger.console_enabled);
}

#[test]
#[serial]
#[cfg(feature = "env")]
fn file_environment_configuration_rejects_invalid_and_removed_settings() {
    let absent = Logger::from_env(Some("OTEL_KIT_CONFIG_TEST")).unwrap();
    assert!(absent.file_appender.is_none());

    let cases = [
        ("ENABLE", "true", true),
        ("ENABLE", "false", true),
        ("ENABLE", "invalid", false),
        ("FORMAT", "json", false),
        ("LEVEL", "warn", false),
        ("ANSI", "false", false),
        ("ROTATION", "invalid", false),
        ("MAX_LOG_FILES", "invalid", false),
    ];
    for (field, value, valid) in cases {
        let key = format!("OTEL_KIT_CONFIG_TEST_FILE_{field}");
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var("OTEL_KIT_CONFIG_TEST_FILE_ENABLE", "true");
            std::env::set_var(&key, value);
        }
        let result = Logger::from_env(Some("OTEL_KIT_CONFIG_TEST"));
        #[allow(unsafe_code)]
        unsafe {
            std::env::remove_var(&key);
            std::env::remove_var("OTEL_KIT_CONFIG_TEST_FILE_ENABLE");
        }
        if valid {
            assert!(result.unwrap().file_appender.is_some());
        } else {
            let error = result.unwrap_err().to_string();
            assert!(error.contains("OTEL_KIT_CONFIG_TEST_FILE"), "{error}");
        }
    }

    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("OTEL_KIT_CONFIG_TEST_FILE_DIR", "./logs");
    }
    let incomplete = Logger::from_env(Some("OTEL_KIT_CONFIG_TEST"));
    #[allow(unsafe_code)]
    unsafe {
        std::env::remove_var("OTEL_KIT_CONFIG_TEST_FILE_DIR");
    }
    assert!(incomplete.is_err(), "file settings require an ENABLE value");
}
