#![cfg(feature = "subscriber")]

use opentelemetry_sdk::{
    error::OTelSdkError, logs::SdkLoggerProvider, metrics::SdkMeterProvider,
    trace::SdkTracerProvider,
};
use otel_init::init_tracing_subscriber;
use tracing_subscriber::EnvFilter;

#[test]
fn duplicate_subscriber_returns_error_and_shuts_down_rejected_providers() {
    let guard = init_tracing_subscriber(
        "first",
        EnvFilter::new("off"),
        vec![],
        SdkTracerProvider::builder().build(),
        SdkMeterProvider::builder().build(),
        None,
    )
    .unwrap();

    let tracer = SdkTracerProvider::builder().build();
    let meter = SdkMeterProvider::builder().build();
    let logger = SdkLoggerProvider::builder().build();
    let result = init_tracing_subscriber(
        "second",
        EnvFilter::new("off"),
        vec![],
        tracer.clone(),
        meter.clone(),
        Some(logger.clone()),
    );

    assert!(result.is_err());
    assert!(matches!(
        tracer.shutdown(),
        Err(OTelSdkError::AlreadyShutdown)
    ));
    assert!(matches!(
        meter.shutdown(),
        Err(OTelSdkError::AlreadyShutdown)
    ));
    assert!(matches!(
        logger.shutdown(),
        Err(OTelSdkError::AlreadyShutdown)
    ));
    guard.shutdown().unwrap();
}
