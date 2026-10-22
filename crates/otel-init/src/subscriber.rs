use crate::guard::OtelGuard;
use anyhow::{Context, Result};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::{
    logs::SdkLoggerProvider, metrics::SdkMeterProvider, trace::SdkTracerProvider,
};
use tracing::Level;
use tracing_subscriber::{
    EnvFilter, Layer, Registry, layer::SubscriberExt, util::SubscriberInitExt,
};

pub type BoxLayer = Box<dyn Layer<Registry> + Sync + Send>;

/// Creates an environment filter for tracing based on the given level.
///
/// This function attempts to create a filter from environment variables first,
/// falling back to the provided level if no environment configuration is found.
///
/// # Arguments
///
/// * `level` - The default tracing level to use if no environment configuration is found
///
/// # Examples
///
/// ```rust
/// use otel_init::init_env_filter;
/// use tracing::Level;
///
/// let filter = init_env_filter(&Level::INFO);
/// ```
pub fn init_env_filter(level: &Level) -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level.to_string()))
}

/// Install the global tracing subscriber and take ownership of provider shutdown.
///
/// Returns an error if installation fails, including when a global subscriber
/// already exists. The supplied providers are shut down on failure. Global
/// registrations are not rolled back; installation is not transactional.
pub fn init_tracing_subscriber(
    name: &str,
    env_filter: EnvFilter,
    mut layers: Vec<BoxLayer>,
    tracer_provider: SdkTracerProvider,
    meter_provider: SdkMeterProvider,
    logger_provider: Option<SdkLoggerProvider>,
) -> Result<OtelGuard> {
    use opentelemetry::trace::TracerProvider as _;
    // Set up telemetry layer with tracer
    let tracer = tracer_provider.tracer(name.to_string());
    let metrics_layer = tracing_opentelemetry::MetricsLayer::new(meter_provider.clone());
    let otel_layer = tracing_opentelemetry::OpenTelemetryLayer::new(tracer);

    let mut extended_layers: Vec<BoxLayer> = vec![Box::new(metrics_layer), Box::new(otel_layer)];

    // Add OpenTelemetry logs bridge layer if logger_provider is provided
    if let Some(ref logger_provider) = logger_provider {
        let otel_logs_layer = OpenTelemetryTracingBridge::new(logger_provider);
        extended_layers.push(Box::new(otel_logs_layer));
    }

    layers.extend(extended_layers);

    let guard = OtelGuard::new(
        Some(tracer_provider),
        Some(meter_provider),
        logger_provider,
    );
    tracing_subscriber::registry()
        .with(layers)
        .with(env_filter)
        .try_init()
        .context("Failed to install global tracing subscriber")?;
    Ok(guard)
}
