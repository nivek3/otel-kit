//! Runtime ownership for initialized logging resources.

use anyhow::Result;
use tracing_appender::non_blocking::WorkerGuard;

use otel_init::OtelGuard;

/// Owns the OpenTelemetry providers and non-blocking log writers initialized by [`super::Logger`].
///
/// Keep this guard alive for as long as logging is needed. Dropping it performs automatic
/// cleanup, or call [`Self::shutdown`] to handle OpenTelemetry shutdown errors explicitly.
///
/// # Examples
///
/// ```no_run
/// use otel_logger::Logger;
///
/// # fn main() -> anyhow::Result<()> {
/// let guard = Logger::new("my-service").init()?;
///
/// // Run the application while `guard` remains in scope.
/// guard.shutdown()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct LoggerGuard {
    // Field order is intentional: Rust drops fields in declaration order. Keep the provider
    // guard first so shutdown diagnostics can be flushed before the writer guard is released.
    otel_guard: OtelGuard,
    worker_guard: Option<WorkerGuard>,
}

impl LoggerGuard {
    pub(crate) fn new(otel_guard: OtelGuard, worker_guard: Option<WorkerGuard>) -> Self {
        Self {
            otel_guard,
            worker_guard,
        }
    }

    /// Shut down the OpenTelemetry providers, then release the file writer guard.
    ///
    /// Releasing the writer guard triggers tracing-appender's shutdown and flush
    /// path, which does not report its outcome to the caller.
    ///
    /// # Errors
    ///
    /// Returns an error if any OpenTelemetry provider fails to shut down. Log
    /// writers are still released after the shutdown attempt.
    pub fn shutdown(self) -> Result<()> {
        let result = self.otel_guard.shutdown();
        drop(self.worker_guard);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        io::Write,
        path::Path,
        sync::mpsc::{self, Sender},
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    const LOG_MARKER: &str = "logger-guard-flush-marker";
    const LOG_RECORDS: usize = 256;

    struct DropProbe(Sender<()>);

    impl Write for DropProbe {
        fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
            Ok(buffer.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl Drop for DropProbe {
        fn drop(&mut self) {
            let _ = self.0.send(());
        }
    }

    #[test]
    fn dropping_logger_guard_releases_file_writer() {
        let (guard, _writer, writer_dropped) = logger_guard_with_drop_probe();

        drop(guard);

        writer_dropped
            .recv_timeout(Duration::from_secs(1))
            .expect("file writer should be released with logger guard");
    }

    #[test]
    fn shutdown_releases_file_writer() {
        let (guard, _writer, writer_dropped) = logger_guard_with_drop_probe();

        guard.shutdown().expect("providers should shut down");

        writer_dropped
            .recv_timeout(Duration::from_secs(1))
            .expect("file writer should be released after shutdown");
    }

    #[test]
    fn dropping_logger_guard_flushes_non_blocking_file_logs() -> Result<()> {
        struct TempDirGuard(std::path::PathBuf);

        impl Drop for TempDirGuard {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }

        let log_dir = create_tmp_log_dir();
        fs::create_dir_all(&log_dir)?;
        let _cleanup = TempDirGuard(log_dir.clone());
        let file_appender = tracing_appender::rolling::never(&log_dir, "logger-guard.log");
        let (writer, worker_guard) = tracing_appender::non_blocking(file_appender);
        let guard = LoggerGuard::new(OtelGuard::new(None, None, None), Some(worker_guard));
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_writer(writer)
            .finish();

        tracing::subscriber::with_default(subscriber, move || {
            for index in 0..LOG_RECORDS {
                tracing::info!("{LOG_MARKER}-{index:03}|");
            }
            drop(guard);
        });

        let logs = read_logs(&log_dir)?;
        for index in 0..LOG_RECORDS {
            let record = format!("{LOG_MARKER}-{index:03}|");
            assert!(logs.contains(&record), "missing {record:?}");
        }
        assert_eq!(logs.matches(LOG_MARKER).count(), LOG_RECORDS);

        Ok(())
    }

    fn logger_guard_with_drop_probe() -> (
        LoggerGuard,
        tracing_appender::non_blocking::NonBlocking,
        mpsc::Receiver<()>,
    ) {
        let (writer_dropped, receiver) = mpsc::channel();
        let (writer, worker_guard) = tracing_appender::non_blocking(DropProbe(writer_dropped));
        let guard = LoggerGuard::new(OtelGuard::new(None, None, None), Some(worker_guard));
        (guard, writer, receiver)
    }

    fn create_tmp_log_dir() -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("otel-logger-{}-{nonce}", std::process::id()))
    }

    fn read_logs(log_dir: &Path) -> std::io::Result<String> {
        let mut logs = String::new();
        for entry in fs::read_dir(log_dir)? {
            logs.push_str(&fs::read_to_string(entry?.path())?);
        }
        Ok(logs)
    }
}
