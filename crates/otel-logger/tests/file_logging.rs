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
fn file_output_respects_format_and_level() {
    let cases = [
        ("code", None, None, None, false, [false, true, true]),
        ("code", Some("compact"), None, None, false, [false, true, true]),
        (
            "code",
            None,
            Some("warn"),
            Some("debug"),
            false,
            [false, false, true],
        ),
        ("code", None, Some("debug"), None, true, [false, true, true]),
        (
            "code",
            None,
            Some("info"),
            Some("debug"),
            true,
            [false, true, true],
        ),
        #[cfg(feature = "env")]
        ("env", None, None, None, false, [false, true, true]),
        #[cfg(feature = "env")]
        (
            "env_override",
            None,
            None,
            Some("debug"),
            true,
            [true, true, true],
        ),
        #[cfg(feature = "env")]
        (
            "env_override",
            Some("compact"),
            None,
            None,
            false,
            [false, true, true],
        ),
        #[cfg(feature = "env")]
        (
            "env",
            None,
            Some("warn"),
            Some("debug"),
            true,
            [false, false, true],
        ),
        #[cfg(feature = "env")]
        ("env", None, Some("debug"), None, false, [false, true, true]),
    ];

    for (source, format, level, global_level, non_blocking, expected) in cases {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = LogDir(std::env::temp_dir().join(format!(
            "otel-logger-file-{}-{nonce}",
            std::process::id()
        )));
        fs::create_dir(&dir.0).unwrap();

        // Each child owns its global subscriber and environment configuration.
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
                if source == "env_override" { "compact" } else { "json" },
            )
            .env("LOG_LEVEL", "info")
            .env("LOG_ANSI", "false")
            .env("LOG_SPAN_EVENTS", "FMT::NONE")
            .env("LOG_FILE_ENABLE", "true")
            .env("LOG_FILE_DIR", &dir.0)
            .env("LOG_FILE_FILENAME_PREFIX", "output")
            .env("LOG_FILE_FILENAME_SUFFIX", "log")
            .env("LOG_FILE_ROTATION", "never")
            .env("LOG_FILE_NON_BLOCKING", non_blocking.to_string());
        if let Some(format) = format {
            command.env("LOG_FILE_FORMAT", format);
        }
        if let Some(level) = level {
            command.env("LOG_FILE_LEVEL", level);
        }
        if let Some(level) = global_level {
            command.env("RUST_LOG", level);
        }

        let output = command.output().unwrap();
        assert!(output.status.success(), "{output:?}");
        let console = String::from_utf8(output.stdout).unwrap();
        let file = fs::read_to_string(dir.0.join("output.log")).unwrap();
        let markers = ["file-probe-debug", "file-probe-info", "file-probe-warn"];
        for ((marker, in_file), in_console) in markers
            .into_iter()
            .zip(expected)
            .zip([global_level.is_some(), true, true])
        {
            assert_eq!(file.contains(marker), in_file, "{source}: {file}");
            assert_eq!(console.contains(marker), in_console, "{source}: {console}");
        }
        for line in file.lines().filter(|line| line.contains("file-probe-")) {
            let json = serde_json::from_str::<serde_json::Value>(line);
            if format.is_none() {
                let json = json.expect("inherited format should be JSON");
                assert!(json["message"].as_str().unwrap().starts_with("file-probe-"));
            } else {
                assert!(json.is_err(), "explicit Compact format should win");
            }
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
            .with_format(LogFormat::Json)
            .with_file_appender(Some(LoggerFileAppender {
                enable: true,
                non_blocking: std::env::var("LOG_FILE_NON_BLOCKING").unwrap() == "true",
                level: std::env::var("LOG_FILE_LEVEL")
                    .ok()
                    .map(|value| value.parse().unwrap()),
                ansi: false,
                format: std::env::var("LOG_FILE_FORMAT")
                    .ok()
                    .map(|_| LogFormat::Compact),
                rotation: LogRollingRotation::Never,
                dir: Some(std::env::var("LOG_FILE_DIR").unwrap()),
                filename_prefix: Some("output".to_owned()),
                filename_suffix: Some("log".to_owned()),
                max_log_files: 1,
            })),
        #[cfg(feature = "env")]
        "env" | "env_override" => {
            let logger = Logger::from_env(None).unwrap();
            let file = logger.file_appender.as_ref().unwrap();
            assert_eq!(
                file.format.is_none(),
                std::env::var("LOG_FILE_FORMAT").is_err()
            );
            assert_eq!(file.level.is_none(), std::env::var("LOG_FILE_LEVEL").is_err());
            logger
        }
        _ => panic!("unknown test source: {source}"),
    };
    if source == "env_override" {
        logger = logger.with_format(LogFormat::Json);
    }
    let guard = logger
        .with_level(Level::INFO)
        .with_ansi(false)
        .with_span_events(FmtSpan::NONE)
        .init()
        .unwrap();
    tracing::debug!("file-probe-debug");
    tracing::info!("file-probe-info");
    tracing::warn!("file-probe-warn");
    guard.shutdown().unwrap();
}
