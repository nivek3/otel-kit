//! Tracing subscriber setup and output layer management.

use crate::{
    logger::{LogFormat, Logger},
    otel::{
        OtelGuard, get_resource, init_logger_provider, init_meter_provider, init_tracer_provider,
        init_tracing_subscriber, opentelemetry::KeyValue,
    },
};
use anyhow::{Context, Result};
use otel_init::BoxLayer;
pub use otel_init::init_env_filter;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    Layer, Registry,
    fmt::{self, MakeWriter, format::FmtSpan},
};

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

/// Create output layers based on configuration.
pub(crate) fn create_output_layers(logger: &Logger) -> Result<OutputLayers> {
    let mut layers: Vec<BoxLayer> = vec![];
    let mut worker_guard = None;

    // Add console layer if enabled
    if logger.console_enabled {
        let stdout_layer = init_layer(
            std::io::stdout,
            &logger.format,
            logger.span_events.clone(),
            logger.ansi,
        );
        layers.push(stdout_layer);
    }
    // Add file layer if configured and enabled
    if let Some(config) = &logger.file_appender
        && config.enable
    {
        let rolling_builder = tracing_appender::rolling::Builder::new()
            .max_log_files(config.max_log_files)
            .rotation(config.get_rolling_rotation());

        let file_appender = rolling_builder
            .filename_prefix(config.filename_prefix_or_default())
            .filename_suffix(config.filename_suffix_or_default())
            .build(config.dir_or_default())
            .context("Failed to build file appender")?;

        let file_appender_layer = if config.non_blocking {
            let (non_blocking_file_appender, non_blocking_worker_guard) =
                tracing_appender::non_blocking(file_appender);
            worker_guard = Some(non_blocking_worker_guard);
            init_layer(
                non_blocking_file_appender,
                &config.format_or_default(),
                logger.span_events.clone(),
                config.ansi,
            )
        } else {
            init_layer(
                file_appender,
                &config.format_or_default(),
                logger.span_events.clone(),
                config.ansi,
            )
        };
        layers.push(file_appender_layer);
    }
    Ok(OutputLayers {
        layers,
        worker_guard,
    })
}

/// Initializes the complete tracing stack with OpenTelemetry integration.
pub fn setup_tracing(
    service_name: &str,
    attributes: &[KeyValue],
    level: Level,
    layers: Vec<BoxLayer>,
    otel_logs_enabled: bool,
) -> Result<OtelGuard> {
    let env_filter = init_env_filter(&level);
    let resource = get_resource(service_name, attributes);
    let tracer_provider = init_tracer_provider(&resource)?;
    let meter_provider = init_meter_provider(&resource)?;
    let logger_provider = if otel_logs_enabled {
        Some(init_logger_provider(&resource)?)
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
