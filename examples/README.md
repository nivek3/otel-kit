## Examples

- [otel](./otel/README.md) - OpenTelemetry tracing for axum
- [microservices](./microservices/README.md) - Microservices example
  - [users](./microservices/users/Cargo.toml) - Users microservice
  - [articles](./microservices/articles/Cargo.toml) - Articles microservice

## Span lifecycle logs

All examples read logging configuration from environment variables. To disable
span lifecycle logs such as `new` and `close`, set the following in the `.env`
file or the service's environment file:

```dotenv
LOG_SPAN_EVENTS=FMT::NONE
```

This only disables span lifecycle messages in console and file output.
OpenTelemetry spans and trace export remain enabled, and explicit log events,
including the HTTP `response` log, are still emitted according to the log filter.
