# Microservices Example

This directory contains a simple microservices example demonstrating how to use OpenTelemetry for distributed tracing and log collection.

## Project Overview

This project consists of three services:
- **users-service**: User management service (port 8081)
- **articles-service**: Article management service (port 8082) 
- **axum-otel-demo**: Demo application (port 8080)

The services communicate via HTTP calls and use OpenTelemetry for distributed tracing. All services are containerized and can be run using Docker Compose.

The articles HTTP client uses `reqwest-tracing` with `opentelemetry_0_33` to create
client spans and propagate `traceparent` and `tracestate` before retry handling.
The three reqwest middleware crates are pinned to upstream commit
`75197ef573cc5f9aec9b0c934e3114950c5cf1e4`, which adds OpenTelemetry 0.33 support.

## Quick Start

### Option 1: Using Docker Compose (Recommended)

```bash
# Start all services
docker compose up -d

# Check service status
docker compose ps
```

### Option 2: Running Services Locally

```bash
# Start user service
cargo run --package users-service

# Start article service  
cargo run --package articles-service

# Start demo service
cargo run --package axum-otel-demo
```

## Service Configuration

### Docker Compose Services

The `docker-compose.yml` includes the following services:

#### 1. Grafana Alloy (Observability Agent)
- **Image**: `grafana/alloy:v1.7.5`
- **Port**: 12345 (internal)
- **Purpose**: Collects logs and traces, forwards to Loki and Tempo
- **Configuration**: Uses `alloy/config.alloy` for log collection and processing

#### 2. Users Service
- **Image**: `ghcr.io/nivek-ph/tracing-otel-extra:main-78cf8fd`
- **Port**: 8081
- **Environment**: Uses `users.env` configuration
- **Logs**: Mounted to `./logs/users`

#### 3. Articles Service  
- **Image**: `ghcr.io/nivek-ph/tracing-otel-extra:main-78cf8fd`
- **Port**: 8082
- **Environment**: Uses `articles.env` configuration
- **Logs**: Mounted to `./logs/articles`
- **Dependencies**: Communicates with users-service

#### 4. Axum OTEL Demo
- **Image**: `ghcr.io/nivek-ph/tracing-otel-extra:main-78cf8fd`
- **Port**: 8080
- **Environment**: Uses `demo.env` configuration
- **Logs**: Mounted to `./logs/demo`

## Testing the Services

### Create a user
```bash
curl -X POST http://localhost:8081/users \
  -H "Content-Type: application/json" \
  -d '{"name": "John Doe", "email": "john@example.com"}'
```

### Create an article
```bash
curl -X POST http://localhost:8082/articles \
  -H "Content-Type: application/json" \
  -d '{"title": "My First Article", "content": "This is the content of my first article", "author_id": 1}'
```

### Get all articles by author
```bash
curl http://localhost:8082/articles/author/1
```

### Test demo service
```bash
curl http://localhost:8080/hello
```

## Monitoring Configuration

### Log Collection with Grafana Alloy

The project uses Grafana Alloy as an observability agent that:

1. **Collects Logs**: Monitors log files from all services in `./logs/` directory
2. **Processes Logs**: Extracts trace IDs and other metadata from JSON logs
3. **Forwards to Loki**: Sends processed logs to Loki for storage and querying

#### Alloy Configuration (`alloy/config.alloy`)

```alloy
// Monitors log files from all services
local.file_match "tmp" {
  path_targets = [
    {"__path__" = "/app/logs/articles/*.log", "app" = "articles"},
    {"__path__" = "/app/logs/demo/*.log", "app" = "demo"},
    {"__path__" = "/app/logs/users/*.log", "app" = "users"},
  ]
  sync_period = "1s"
}

// Processes logs to extract trace information
loki.process "extract_tracing_fields" {
  stage.json {
    expressions = {
      "trace_id" = "span.trace_id",
    }
  }
  stage.labels {
    values = {
      trace_id = "",
    }
  }
}
```

### Environment Configuration

Each service uses its own environment file. You need to create these files before starting the services:

#### Create `articles.env`
```bash
cat > articles.env << 'EOF'
OTEL_EXPORTER_OTLP_ENDPOINT="http://198.18.0.1:4317"
RUST_LOG=info
LOG_SERVICE_NAME="articles"
LOG_FORMAT=compact
LOG_FILE_ENABLE=true
LOG_FILE_FORMAT=json
LOG_FILE_DIR=./logs
EOF
```

#### Create `users.env`
```bash
cat > users.env << 'EOF'
OTEL_EXPORTER_OTLP_ENDPOINT="http://198.18.0.1:4317"
RUST_LOG=info
LOG_SERVICE_NAME="users"
LOG_FORMAT=compact
LOG_FILE_ENABLE=true
LOG_FILE_FORMAT=json
LOG_FILE_DIR=./logs
EOF
```

#### Create `demo.env`
```bash
cat > demo.env << 'EOF'
OTEL_EXPORTER_OTLP_ENDPOINT="http://198.18.0.1:4317"
RUST_LOG=info
LOG_SERVICE_NAME="demo"
LOG_FORMAT=compact
LOG_FILE_ENABLE=true
LOG_FILE_FORMAT=json
LOG_FILE_DIR=./logs
EOF
```

## Observability Stack

### Current Setup
- **Log Collection**: Grafana Alloy → Loki
- **Tracing**: OpenTelemetry → Tempo (via OTLP)
- **Visualization**: Grafana (Loki + Tempo)

### External Dependencies

To complete the observability stack, you need to run:

#### Loki + Tempo Stack
```bash
# Clone and setup dockotlp project
git clone https://github.com/nivek3/dockotlp
cd dockotlp

# Start Loki + Tempo services
docker compose up -d
```

#### Jaeger (Alternative)
```bash
docker run -d \
  -p 6831:6831/udp \
  -p 6832:6832/udp \
  -p 16686:16686 \
  -p 4317:4317 \
  jaegertracing/all-in-one:latest
```

## Documentation

- **[Quick Start Guide](QUICKSTART.md)** - Get up and running in minutes
- **[Architecture Overview](ARCHITECTURE.md)** - Detailed system architecture
- **[README.md](README.md)** - This file (detailed configuration)

## Project Structure

```
microservices/
├── articles/           # Article service source
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── users/              # User service source
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── config.alloy        # Grafana Agent configuration
├── README.md           # This file
├── QUICKSTART.md       # Quick start guide
└── ARCHITECTURE.md     # Architecture documentation

# Root level files (for Docker Compose)
├── docker-compose.yml  # Service orchestration
├── alloy/
│   └── config.alloy    # Alloy configuration
├── articles.env        # Articles service config
├── users.env          # Users service config
├── demo.env           # Demo service config
└── logs/              # Log directory
    ├── articles/      # Articles service logs
    ├── users/         # Users service logs
    └── demo/          # Demo service logs
```

## Technology Stack

- **Framework**: Axum (Rust web framework)
- **Tracing**: OpenTelemetry + Tempo/Jaeger
- **Logging**: JSON format + Loki
- **Monitoring**: Grafana + Alloy (Grafana Agent)
- **Containerization**: Docker + Docker Compose
- **Observability**: Distributed tracing with correlation IDs

## Troubleshooting

### Common Issues

1. **Port conflicts**: Ensure ports 8080, 8081, and 8082 are not in use
2. **Docker service not running**: Make sure Docker is running
3. **Log file permissions**: Ensure the `logs/` directory exists and has write permissions
4. **OTLP endpoint**: Verify the OTLP endpoint is accessible (default: `http://198.18.0.1:4317`)

### Log Levels

You can set log levels via environment variables in the `.env` files:
```bash
RUST_LOG=debug
```

### Service Communication

- Articles service communicates with Users service via `USERS_SERVICE_URL=http://users-service:8081`
- All services export traces to the same OTLP endpoint
- All services write logs to their respective directories in `./logs/`

### Viewing Logs

```bash
# View service logs
docker compose logs users-service
docker compose logs articles-service
docker compose logs axum-otel-demo

# View Alloy logs
docker compose logs alloy

# View log files directly
tail -f logs/articles/app.log
tail -f logs/users/app.log
tail -f logs/demo/app.log
```
