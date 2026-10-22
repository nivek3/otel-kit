use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use otel_logger::{FmtSpan, LogFormat, LogRollingRotation, Logger, LoggerFileAppender};
use tracing::Level;

struct LogDir(PathBuf);

impl Drop for LogDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn outputs_share_format_filter_and_span_fields() {
    // Source, format, ANSI, non-blocking, console enabled, file enabled, global level.
    let cases = [
        ("code", "compact", true, false, true, true, "info"),
        ("code", "compact", false, true, true, true, "debug"),
        ("code", "pretty", true, true, true, true, "info"),
        ("code", "pretty", false, false, true, true, "debug"),
        ("code", "json", true, true, true, true, "info"),
        ("code", "json", false, false, true, true, "warn"),
        ("code", "compact", false, true, false, true, "info"),
        ("code", "json", false, false, true, false, "info"),
        ("code", "json", false, true, false, false, "info"),
        #[cfg(feature = "env")]
        ("env", "pretty", false, false, true, true, "info"),
        #[cfg(feature = "env")]
        ("env_override", "json", false, true, true, true, "debug"),
    ];
    for (source, format, ansi, non_blocking, console_enabled, file_enabled, level) in cases {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = LogDir(
            std::env::temp_dir().join(format!("otel-logger-file-{}-{nonce}", std::process::id())),
        );
        fs::create_dir(&dir.0).unwrap();
        let mut command = Command::new(std::env::current_exe().unwrap());
        command.args(["--exact", "file_logging_child", "--nocapture"]);
        for (key, _) in std::env::vars_os() {
            let name = key.to_string_lossy();
            if name.starts_with("OTEL_") || name.starts_with("LOG_") || name == "RUST_LOG" {
                command.env_remove(key);
            }
        }
        command
            .env("OTEL_KIT_FILE_TEST_SOURCE", source)
            .env(
                "LOG_FORMAT",
                if source == "env_override" {
                    "compact"
                } else {
                    format
                },
            )
            .env("LOG_LEVEL", "info")
            .env("RUST_LOG", level)
            .env("LOG_ANSI", ansi.to_string())
            .env("LOG_CONSOLE_ENABLED", console_enabled.to_string())
            .env("LOG_SPAN_EVENTS", "FMT::NONE")
            .env("LOG_FILE_ENABLE", file_enabled.to_string())
            .env("LOG_FILE_DIR", &dir.0)
            .env("LOG_FILE_FILENAME_PREFIX", "output")
            .env("LOG_FILE_FILENAME_SUFFIX", "log")
            .env("LOG_FILE_ROTATION", "never")
            .env("LOG_FILE_NON_BLOCKING", non_blocking.to_string());
        let output = command.output().unwrap();
        assert!(output.status.success(), "{output:?}");
        let console = String::from_utf8(output.stdout).unwrap();
        let path = dir.0.join("output.log");
        assert_eq!(path.exists(), file_enabled);
        let file = if file_enabled {
            fs::read_to_string(path).unwrap()
        } else {
            String::new()
        };
        let expected = [level == "debug", level != "warn", true];
        for (marker, enabled) in ["file-probe-debug", "file-probe-info", "file-probe-warn"]
            .into_iter()
            .zip(expected)
        {
            assert_eq!(
                console.contains(marker),
                console_enabled && enabled,
                "{console}"
            );
            assert_eq!(file.contains(marker), file_enabled && enabled, "{file}");
        }
        if !file_enabled {
            continue;
        }
        if console_enabled {
            assert!(
                console.contains(&file),
                "console and file must contain the same formatted bytes\nconsole: {console}\nfile: {file}"
            );
        }
        assert_eq!(file.contains('\u{1b}'), ansi && format != "json", "{file}");
        if format == "json" {
            for line in file.lines() {
                let event: serde_json::Value = serde_json::from_str(line).unwrap();
                // SDK shutdown logs have no request span.
                if event["target"] != "file_logging" {
                    continue;
                }
                assert!(
                    event["message"]
                        .as_str()
                        .unwrap()
                        .starts_with("file-probe-"),
                    "{file}"
                );
                if level != "warn" {
                    assert_eq!(event["span"]["initial"], "initial-marker");
                    assert_eq!(event["span"]["updated"], "updated-marker");
                } else {
                    assert!(
                        event.get("span").is_none(),
                        "globally filtered spans must stay absent"
                    );
                }
            }
        } else {
            let events = expected.into_iter().filter(|enabled| *enabled).count();
            assert_eq!(file.matches("initial-marker").count(), events, "{file}");
            assert_eq!(file.matches("updated-marker").count(), events, "{file}");
        }
    }
}

#[tokio::test]
async fn file_logging_child() {
    let Ok(source) = std::env::var("OTEL_KIT_FILE_TEST_SOURCE") else {
        return;
    };
    let mut logger = match source.as_str() {
        "code" => Logger::new("file-test")
            .with_format(
                serde_json::from_value(std::env::var("LOG_FORMAT").unwrap().into()).unwrap(),
            )
            .with_ansi(std::env::var("LOG_ANSI").unwrap() == "true")
            .with_console_enabled(std::env::var("LOG_CONSOLE_ENABLED").unwrap() == "true")
            .with_file_appender(Some(LoggerFileAppender {
                enable: std::env::var("LOG_FILE_ENABLE").unwrap() == "true",
                non_blocking: std::env::var("LOG_FILE_NON_BLOCKING").unwrap() == "true",
                rotation: LogRollingRotation::Never,
                dir: Some(std::env::var("LOG_FILE_DIR").unwrap()),
                filename_prefix: Some("output".to_owned()),
                filename_suffix: Some("log".to_owned()),
                max_log_files: 1,
            })),
        #[cfg(feature = "env")]
        "env" | "env_override" => Logger::from_env(None).unwrap(),
        _ => panic!("unknown test source: {source}"),
    };
    if source == "env_override" {
        logger = logger.with_format(LogFormat::Json);
    }
    let guard = logger
        .with_level(Level::INFO)
        .with_span_events(FmtSpan::NONE)
        .init()
        .unwrap();
    {
        let span = tracing::info_span!(
            "request",
            initial = "initial-marker",
            updated = tracing::field::Empty
        );
        span.record("updated", "updated-marker");
        let _entered = span.enter();
        tracing::debug!("file-probe-debug");
        tracing::info!("file-probe-info");
        tracing::warn!("file-probe-warn");
    }
    guard.shutdown().unwrap();
}
