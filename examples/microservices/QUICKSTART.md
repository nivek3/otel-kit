# Quick Start Guide

This guide will help you get the microservices example running quickly.

## Prerequisites

- Docker and Docker Compose installed
- Git (to clone the repository)

## Step 1: Start the Observability Stack

First, you need to start the observability backend (Loki + Tempo + Grafana):

```bash
# Clone the dockotlp project
git clone https://github.com/nivek3/dockotlp
cd dockotlp

# Start the observability stack
docker compose up -d

# Verify services are running
docker compose ps
```

## Step 2: Start the Microservices

In a new terminal, navigate back to the otel-kit project:

```bash
cd otel-kit

# Create logs directory
mkdir -p logs/{articles,users,demo}

# Copy .env files and change the files to your own
cp .env.example demo.env
cp .env.example articles.env
cp .env.example users.env

# Start all microservices
docker compose up -d

# Check service status
docker compose ps
```

## Step 3: Test the Services

### Test the Demo Service
```bash
curl http://localhost:8080/hello
```

### Test User Management
```bash
# Create a user
curl -X POST http://localhost:8081/users \
  -H "Content-Type: application/json" \
  -d '{"name": "John Doe", "email": "john@example.com"}'

# Expected response: {"id": 1, "name": "John Doe", "email": "john@example.com"}
```

### Test Article Management
```bash
# Create an article
curl -X POST http://localhost:8082/articles \
  -H "Content-Type: application/json" \
  -d '{"title": "My First Article", "content": "This is the content", "author_id": 1}'

# Get articles by author
curl http://localhost:8082/articles/author/1
```

## Step 4: View Observability Data

### Access Grafana
- URL: http://localhost:3000
- Username: `admin`
- Password: `admin`

### View Logs in Grafana
1. Go to Explore
2. Select Loki as data source
3. Query: `{app="articles"}` or `{app="users"}` or `{app="demo"}`

### View Traces in Grafana
1. Go to Explore
2. Select Tempo as data source
3. Search for traces by service name or trace ID

## Step 5: Clean Up

```bash
# Stop microservices
docker compose down

# Stop observability stack (in dockotlp directory)
cd /path/to/dockotlp
docker compose down
```

## Troubleshooting

### Services Not Starting
```bash
# Check logs
docker compose logs

# Check specific service
docker compose logs users-service
```

### Port Conflicts
If you get port conflicts, check what's using the ports:
```bash
# Check port usage
lsof -i :8080
lsof -i :8081
lsof -i :8082
```

### Observability Stack Issues
```bash
# Check if dockotlp services are running
cd /path/to/dockotlp
docker compose ps

# Check logs
docker compose logs
```

## Next Steps

- Read the full [README.md](README.md) for detailed configuration
- Explore the source code in `articles/` and `users/` directories
- Customize the environment files for your needs
