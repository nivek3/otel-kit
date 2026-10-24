//! Tracing subscriber setup and output layer management.

use anyhow::{Context, Result};
use opentelemetry::KeyValue;
use otel_init::{
    BoxLayer, OtelGuard, get_resource, init_env_filter, init_logger_provider, init_meter_provider,
    init_tracer_provider, init_tracing_subscriber,
};
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    Layer, Registry,
    fmt::{
        self, MakeWriter,
        format::FmtSpan,
        writer::{BoxMakeWriter, MakeWriterExt},
    },
};

use crate::{LogFormat, Logger};

pub(crate) struct OutputLayers {
    pub(crate) layers: Vec<BoxLayer>,
    pub(crate) worker_guard: Option<WorkerGuard>,
}

/// Apply the specified format to a tracing layer
fn apply_layer_format<N, W>(
    layer: fmt::Layer<Registry, N, fmt::format::Format, W>,
    format: &LogFormat,
) -> Box<dyn Layer<Registry> + Sync + Send>
where
    N: for<'writer> fmt::format::FormatFields<'writer> + Sync + Send + 'static,
    W: for<'writer> MakeWriter<'writer> + Sync + Send + 'static,
{
    match format {
        LogFormat::Compact => layer.compact().boxed(),
        LogFormat::Pretty => layer.pretty().boxed(),
        LogFormat::Json => layer.json().flatten_event(true).boxed(),
    }
}

/// Initialize a format layer with the given writer and format
pub fn init_layer<W2>(
    writer: W2,
    format: &LogFormat,
    span_events: FmtSpan,
    ansi: bool,
) -> Box<dyn Layer<Registry> + Sync + Send>
where
    W2: for<'writer> MakeWriter<'writer> + Sync + Send + 'static,
{
    let layer = fmt::Layer::new()
        .with_writer(writer)
        .with_ansi(ansi)
        .with_span_events(span_events);
    apply_layer_format(layer, format)
}

/// Create a single formatting layer for all enabled outputs.
pub(crate) fn create_output_layers(logger: &Logger) -> Result<OutputLayers> {
    let mut worker_guard = None;
    let file_writer = if let Some(config) = &logger.file_appender
        && config.enable
    {
        let appender = tracing_appender::rolling::Builder::new()
            .max_log_files(config.max_log_files)
            .rotation(config.get_rolling_rotation())
            .filename_prefix(config.filename_prefix_or_default())
            .filename_suffix(config.filename_suffix_or_default())
            .build(config.dir_or_default())
            .context("Failed to build file appender")?;

        Some(if config.non_blocking {
            let (writer, guard) = tracing_appender::non_blocking(appender);
            worker_guard = Some(guard);
            BoxMakeWriter::new(writer)
        } else {
            BoxMakeWriter::new(appender)
        })
    } else {
        None
    };

    let layers = if let Some(file_writer) = file_writer {
        let console_enabled = logger.console_enabled;
        let console_writer = std::io::stdout.with_filter(move |_| console_enabled);
        // Format once so both destinations share the same bytes and span-field cache.
        vec![init_layer(
            console_writer.and(file_writer),
            &logger.format,
            logger.span_events.clone(),
            logger.ansi,
        )]
    } else if logger.console_enabled {
        vec![init_layer(
            std::io::stdout,
            &logger.format,
            logger.span_events.clone(),
            logger.ansi,
        )]
    } else {
        vec![]
    };
    Ok(OutputLayers {
        layers,
        worker_guard,
    })
}

/// Initializes the complete tracing stack with OpenTelemetry integration.
/// The service name identifies the tracer's instrumentation scope; resource
/// attributes use SDK defaults merged with `attributes`.
///
/// On failure, already-created providers are shut down. Global provider and
/// propagator registrations are not restored; initialization is not transactional.
pub fn setup_tracing(
    service_name: &str,
    attributes: &[KeyValue],
    level: Level,
    layers: Vec<BoxLayer>,
    otel_logs_enabled: bool,
) -> Result<OtelGuard> {
    let env_filter = init_env_filter(&level);
    let resource = get_resource(attributes);
    let tracer_provider = init_tracer_provider(&resource)?;
    let meter_provider = match init_meter_provider(&resource) {
        Ok(provider) => provider,
        Err(error) => {
            drop(OtelGuard::new(Some(tracer_provider), None, None));
            return Err(error);
        }
    };
    let logger_provider = if otel_logs_enabled {
        match init_logger_provider(&resource) {
            Ok(provider) => Some(provider),
            Err(error) => {
                drop(OtelGuard::new(
                    Some(tracer_provider),
                    Some(meter_provider),
                    None,
                ));
                return Err(error);
            }
        }
    } else {
        None
    };

    let guard = init_tracing_subscriber(
        service_name,
        env_filter,
        layers,
        tracer_provider,
        meter_provider,
        logger_provider,
    )?;

    Ok(guard)
}
