use otel_logger::{
    BoxLayer, FmtSpan, Logger, OtelGuard, default, init_env_filter, init_layer,
    init_tracing_from_logger, setup_tracing,
};

#[test]
fn public_bootstrap_api_and_default_scope() {
    let logger = Logger::default();
    let deserialized: Logger = serde_json::from_str("{}").unwrap();
    assert_eq!(logger.service_name, "otel_logger");
    assert_eq!(deserialized.service_name, "otel_logger");
    assert_eq!(default::service_name(), "otel_logger");

    let _: BoxLayer = init_layer(std::io::sink, &logger.format, FmtSpan::NONE, false);
    let _ = (init_env_filter, init_tracing_from_logger, setup_tracing);
    let _ = OtelGuard::new(None, None, None);
}
