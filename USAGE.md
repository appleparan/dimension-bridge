# Dimension Bridge Certificate Manager Usage Guide

## Quick Start

### 1. Build the Docker Image

```bash
# Build locally
./scripts/build.sh

# Or use pre-built image
docker pull appleparan/dimension-bridge:latest
```

### 2. Basic Usage

```bash
docker run --rm \
  -e CERT_DOMAINS=api.company.com \
  -e STEP_CA_URL=https://ca.company.com:9000 \
  -e RELOAD_COMMAND='echo "Certificate updated"' \
  -v $(pwd)/certs:/certs \
  appleparan/dimension-bridge:latest once
```

### 3. Production Deployment with Docker Compose

```yaml
services:
  nginx:
    image: nginx:alpine
    volumes:
      - web_certs:/etc/ssl/certs:ro
    ports:
      - "443:443"

  cert-agent:
    image: appleparan/dimension-bridge:latest
    environment:
      CERT_DOMAINS: "web.company.internal"
      STEP_CA_URL: "https://ca.company.internal:9000"
      RELOAD_COMMAND: "docker exec nginx nginx -s reload"
      SERVICE_NAME: "nginx-web"
    volumes:
      - web_certs:/certs:rw
      - /var/run/docker.sock:/var/run/docker.sock:ro
    restart: unless-stopped

volumes:
  web_certs:
```

## Environment Variables

### Required

| Variable | Description | Example |
|----------|-------------|---------|
| `CERT_DOMAINS` | Comma-separated domains | `api.company.com,web.company.com` |
| `STEP_CA_URL` | Step CA server URL | `https://ca.company.com:9000` |
| `RELOAD_COMMAND` | Service reload command | `docker exec nginx nginx -s reload` |

### Optional

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVICE_NAME` | - | Service identifier for logging |
| `CERT_DIR` | `/certs` | Certificate output directory |
| `RENEWAL_DAYS` | `7` | Days before expiry to renew |
| `CHECK_INTERVAL` | `6h` | Certificate check frequency |
| `CERT_VALIDITY` | `15d` | Requested certificate validity |
| `HEALTH_PORT` | `8080` | Health check server port |
| `SLACK_WEBHOOK_URL` | - | Slack notification webhook |
| `RUST_LOG` | `info` | Logging level |

## Commands

### Run Modes

```bash
# Daemon mode (default)
docker run appleparan/dimension-bridge:latest run

# Run once and exit
docker run appleparan/dimension-bridge:latest once

# Check health
docker run appleparan/dimension-bridge:latest health

# Validate configuration
docker run appleparan/dimension-bridge:latest validate

# Show version
docker run appleparan/dimension-bridge:latest version
```

## Common Patterns

### 1. Nginx Web Server

```yaml
nginx-cert-agent:
  image: appleparan/dimension-bridge:latest
  environment:
    CERT_DOMAINS: "web.company.internal"
    STEP_CA_URL: "https://ca.company.internal:9000"
    RELOAD_COMMAND: "docker exec nginx nginx -s reload"
  volumes:
    - web_certs:/certs:rw
    - /var/run/docker.sock:/var/run/docker.sock:ro
```

### 2. API Service

```yaml
api-cert-agent:
  image: appleparan/dimension-bridge:latest
  environment:
    CERT_DOMAINS: "api.company.internal"
    STEP_CA_URL: "https://ca.company.internal:9000"
    RELOAD_COMMAND: "curl -X POST http://api:8080/reload-ssl"
  volumes:
    - api_certs:/certs:rw
```

### 3. Database (PostgreSQL)

```yaml
db-cert-agent:
  image: appleparan/dimension-bridge:latest
  environment:
    CERT_DOMAINS: "db.company.internal"
    STEP_CA_URL: "https://ca.company.internal:9000"
    RELOAD_COMMAND: "docker exec postgres pg_ctl reload"
  volumes:
    - db_certs:/certs:rw
    - /var/run/docker.sock:/var/run/docker.sock:ro
```

### 4. Multiple Services

```yaml
multi-cert-agent:
  image: appleparan/dimension-bridge:latest
  environment:
    CERT_DOMAINS: "service1.company.internal,service2.company.internal"
    STEP_CA_URL: "https://ca.company.internal:9000"
    RELOAD_COMMAND: "docker exec service1 reload && docker exec service2 reload"
  volumes:
    - multi_certs:/certs:rw
    - /var/run/docker.sock:/var/run/docker.sock:ro
```

## Health Monitoring

### Health Check Endpoint

```bash
# Check health via HTTP
curl http://localhost:8080/health

# Response format
{
  "status": "Healthy",
  "timestamp": "2024-01-01T12:00:00Z",
  "details": {
    "certificates": {
      "total_certificates": 2,
      "valid_certificates": 2,
      "expiring_soon": 0,
      "expired_certificates": 0,
      "failed_renewals": 0
    },
    "step_ca": {
      "status": "Healthy",
      "last_check": "2024-01-01T12:00:00Z",
      "response_time_ms": 150
    }
  },
  "uptime_seconds": 3600,
  "version": "0.1.0"
}
```

### Docker Health Check

```bash
# Check container health
docker inspect --format='{{.State.Health.Status}}' cert-agent

# View health check logs
docker inspect --format='{{range .State.Health.Log}}{{.Output}}{{end}}' cert-agent
```

## Notifications

### Slack Integration

```bash
# Set Slack webhook URL
export SLACK_WEBHOOK_URL="https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
```

### Custom Webhooks

```bash
# Multiple webhook URLs (comma-separated)
export WEBHOOK_URLS="https://api.company.com/webhooks/cert,https://monitoring.company.com/alerts"
```

## Troubleshooting

### Common Issues

1. **Permission Denied for Docker Socket**

   ```bash
   # Ensure cert-agent can access Docker socket
   docker run -v /var/run/docker.sock:/var/run/docker.sock:ro ...
   ```

2. **Certificate Directory Permissions**

   ```bash
   # Fix certificate directory permissions
   sudo chown -R 1000:1000 /path/to/certs
   ```

3. **Step CA Connection Issues**

   ```bash
   # Test Step CA connectivity
   curl -k https://ca.company.internal:9000/health
   ```

4. **Service Reload Failures**

   ```bash
   # Test reload command manually
   docker exec cert-agent /bin/bash -c "your-reload-command"
   ```

### Debug Mode

```bash
# Enable debug logging
docker run -e RUST_LOG=debug appleparan/dimension-bridge:latest
```

### Logs

```bash
# View container logs
docker logs -f cert-agent

# View logs with timestamps
docker logs -t cert-agent
```

## Security Considerations

### File Permissions

- Certificate files: `644` (world-readable)
- Private key files: `600` (owner read-only)
- Certificate directory: `755`

### Docker Socket Access

- Use read-only Docker socket mount when possible
- Limit container capabilities
- Run as non-root user (done by default)

### Network Security

- Use internal networks for Docker containers
- Restrict health check endpoint access
- Use TLS for Step CA communication

## Performance Tuning

### Resource Limits

```yaml
cert-agent:
  image: appleparan/dimension-bridge:latest
  deploy:
    resources:
      limits:
        memory: 128M
        cpus: '0.5'
      reservations:
        memory: 64M
        cpus: '0.1'
```

### Optimization Settings

```bash
# Adjust check intervals based on certificate validity
export CHECK_INTERVAL=12h  # For longer-lived certificates

# Reduce renewal threshold for critical services
export RENEWAL_DAYS=14     # Renew 2 weeks before expiry
```

## Advanced Configuration

### Custom Configuration File

Create `config.toml`:

```toml
[step_ca]
url = "https://ca.company.internal:9000"
root_fingerprint = "sha256:abcd..."

[renewal]
days_before_expiry = 7
check_interval = "6h"
validity_period = "15d"

[health]
port = 8080
path = "/health"

[notifications]
slack_webhook_url = "https://hooks.slack.com/..."
```

Mount the config file:

```bash
docker run -v ./config.toml:/app/config.toml appleparan/dimension-bridge:latest
```

## Migration from cert-bot or similar tools

### From Let's Encrypt + certbot

1. Update renewal scripts to use Step CA
2. Modify file paths in your services
3. Update renewal frequency (Step CA certificates are typically shorter-lived)

### From manual certificate management

1. Identify all services using certificates
2. Create certificate agent containers for each service
3. Test reload commands
4. Gradually migrate services to automatic renewal
