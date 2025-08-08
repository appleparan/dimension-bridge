# Internal PKI Certificate Auto-Renewal System

## 📋 Overview

Automated SSL/TLS certificate management system for internal services using
centralized PKI infrastructure.

## 🏗️ Architecture

```text
[Step CA (GCP)]
    ↓ ACME/API
[Cert Agent Container] ← Deploy per service
    ↓ Volume Mount
[Application Containers] ← Shared certificate files
```

## 🎯 Core Components

### 1. Step CA Server (GCP)

- **Purpose**: Internal PKI Root Certificate Authority
- **Location**: GCP instance
- **Features**:
  - ACME/API-based certificate issuance
  - Automatic renewal support
  - Policy-based management

### 2. Generic Cert Agent (Docker Image)

- **Image**: `appleparan/cert-agent:latest`
- **Purpose**: Per-service certificate auto-renewal
- **Pattern**: Sidecar container

### 3. Service Integration

- **Method**: Shared volume mount
- **Access**: Read-only certificate files
- **Reload**: Automatic via agent

## 🔧 Usage Pattern

### Basic Service Setup

```yaml
services:
  # Your application
  web-service:
    image: nginx:latest
    volumes:
      - certs:/etc/ssl/certs:ro

  # Certificate agent (add this)
  cert-agent:
    image: company/cert-agent:latest
    environment:
      CERT_DOMAINS: "web.company.internal"
      RELOAD_COMMAND: "docker exec web-service nginx -s reload"
    volumes:
      - certs:/certs:rw
      - /var/run/docker.sock:/var/run/docker.sock

volumes:
  certs:
```

## 📊 Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `CERT_DOMAINS` | ✅ | - | Comma-separated domain list |
| `STEP_CA_URL` | ✅ | - | Step CA server URL |
| `RELOAD_COMMAND` | ✅ | - | Service reload command |
| `RENEWAL_DAYS` | ❌ | 7 | Days before expiry to renew |
| `CHECK_INTERVAL` | ❌ | 6h | Check frequency |
| `CERT_VALIDITY` | ❌ | 15d | Certificate validity period |
| `SLACK_WEBHOOK_URL` | ❌ | - | Notification webhook |
| `SERVICE_NAME` | ❌ | - | Service identifier for logging |

### Reload Command Examples

```bash
# Nginx graceful reload
RELOAD_COMMAND="docker exec nginx nginx -s reload"

# Apache graceful restart
RELOAD_COMMAND="docker exec apache httpd -k graceful"

# API endpoint call
RELOAD_COMMAND="curl -X POST http://api:3000/reload-certs"

# Container restart
RELOAD_COMMAND="docker restart api-server"

# Multiple commands
RELOAD_COMMAND="docker exec nginx nginx -s reload && \
  docker exec api curl -X POST localhost:8080/reload"
```

## 🔄 Workflow

### 1. Certificate Lifecycle

```text
Generate → Deploy → Monitor → Renew → Reload → Repeat
```

### 2. Renewal Process

1. **Check**: Monitor certificate expiry (every 6h default)
2. **Renew**: Request new certificate (7 days before expiry default)
3. **Deploy**: Atomic file replacement in shared volume
4. **Reload**: Execute configured reload command
5. **Verify**: Confirm new certificate is valid
6. **Notify**: Send success/failure notifications

### 3. File Structure

```text
/certs/
├── {domain}.crt          # Certificate file
├── {domain}.key          # Private key
├── ca.crt                # CA certificate
└── .metadata/
    ├── last_renewal.json
    └── status.json
```

## 🚨 Failure Handling

### Backup Strategy

- **Auto-backup**: Previous certificate kept as `.old`
- **Rollback**: Automatic rollback on reload failure
- **Manual override**: Emergency certificate replacement

### Monitoring & Alerts

- **Health checks**: Container health endpoint
- **Notifications**: Slack/webhook integration
- **Metrics**: Prometheus metrics export
- **Logs**: Structured JSON logging

## 📦 Deployment Strategies

### Option 1: Docker Compose (Recommended)

- Sidecar pattern with shared volumes
- Simple configuration via environment variables

### Option 2: Kubernetes

- Cert-manager integration possible
- ConfigMap/Secret management

### Option 3: Standalone

- Direct host deployment
- Systemd service integration

## 🔒 Security

### Access Control

- **Read-only**: Application containers (certificate files)
- **Read-write**: Cert agent only (certificate management)
- **Docker socket**: Limited to restart operations

### Certificate Security

- **Key permissions**: 600 (owner read-only)
- **Certificate permissions**: 644 (world-readable)
- **CA validation**: Fingerprint verification

## 🎛️ Service Patterns

### Pattern 1: Web Servers

```yaml
# Nginx, Apache, Traefik
cert-agent:
  environment:
    RELOAD_COMMAND: "docker exec nginx nginx -s reload"
```

### Pattern 2: API Services

```yaml
# REST APIs, microservices
cert-agent:
  environment:
    RELOAD_COMMAND: "curl -X POST http://api:8080/reload-ssl"
```

### Pattern 3: Databases

```yaml
# PostgreSQL, MySQL
cert-agent:
  environment:
    RELOAD_COMMAND: "docker restart postgres"
```

### Pattern 4: Auto-reload Services

```yaml
# Services that auto-detect file changes (Traefik, etc.)
cert-agent:
  environment:
    RELOAD_COMMAND: "echo 'Auto-reload enabled'"
```

## 🏢 Common Internal Services

### Authentik (Identity Provider)

```yaml
services:
  authentik-server:
    image: ghcr.io/goauthentik/server:latest
    volumes:
      - authentik_certs:/certs:ro
    environment:
      AUTHENTIK_WEB__HTTPS: "true"
      AUTHENTIK_WEB__TLS_CERT: "/certs/authentik.company.internal.crt"
      AUTHENTIK_WEB__TLS_KEY: "/certs/authentik.company.internal.key"

  authentik-ldap:
    image: ghcr.io/goauthentik/ldap:latest
    volumes:
      - authentik_certs:/certs:ro
    environment:
      AUTHENTIK_LDAP_TLS_CERT_FILE: "/certs/authentik.company.internal.crt"
      AUTHENTIK_LDAP_TLS_KEY_FILE: "/certs/authentik.company.internal.key"

  authentik-cert-agent:
    image: company/cert-agent:latest
    environment:
      CERT_DOMAINS: "authentik.company.internal,ldap.company.internal"
      RELOAD_COMMAND: "docker exec authentik-server kill -HUP 1 && \
        docker exec authentik-ldap kill -HUP 1"
      SERVICE_NAME: "authentik"
      STEP_CA_URL: "https://ca.company.internal:9000"
    volumes:
      - authentik_certs:/certs:rw
      - /var/run/docker.sock:/var/run/docker.sock

volumes:
  authentik_certs:
```

### Monitoring Stack (Grafana, Prometheus)

```yaml
services:
  grafana:
    image: grafana/grafana:latest
    volumes:
      - monitoring_certs:/etc/ssl/certs:ro
    environment:
      GF_SERVER_PROTOCOL: "https"
      GF_SERVER_CERT_FILE: "/etc/ssl/certs/grafana.company.internal.crt"
      GF_SERVER_CERT_KEY: "/etc/ssl/certs/grafana.company.internal.key"

  prometheus:
    image: prom/prometheus:latest
    volumes:
      - monitoring_certs:/etc/ssl/certs:ro

  monitoring-cert-agent:
    image: company/cert-agent:latest
    environment:
      CERT_DOMAINS: "grafana.company.internal,prometheus.company.internal"
      RELOAD_COMMAND: "docker restart grafana && docker restart prometheus"
      SERVICE_NAME: "monitoring"
    volumes:
      - monitoring_certs:/certs:rw
      - /var/run/docker.sock:/var/run/docker.sock

volumes:
  monitoring_certs:
```

### Internal API Gateway

```yaml
services:
  api-gateway:
    image: nginx:latest
    volumes:
      - gateway_certs:/etc/ssl/certs:ro

  internal-api:
    image: company/api:latest
    volumes:
      - gateway_certs:/etc/ssl/certs:ro

  api-cert-agent:
    image: company/cert-agent:latest
    environment:
      CERT_DOMAINS: "api.company.internal,gateway.company.internal"
      RELOAD_COMMAND: "docker exec api-gateway nginx -s reload && \
        curl -X POST http://internal-api:8080/reload-ssl"
      SERVICE_NAME: "api-gateway"
    volumes:
      - gateway_certs:/certs:rw
      - /var/run/docker.sock:/var/run/docker.sock

volumes:
  gateway_certs:
```

### Database Services

```yaml
services:
  postgres:
    image: postgres:15
    volumes:
      - db_certs:/var/lib/postgresql/certs:ro
    environment:
      POSTGRES_SSL_CERT_FILE: \
        "/var/lib/postgresql/certs/db.company.internal.crt"
      POSTGRES_SSL_KEY_FILE: "/var/lib/postgresql/certs/db.company.internal.key"

  db-cert-agent:
    image: company/cert-agent:latest
    environment:
      CERT_DOMAINS: "db.company.internal"
      RELOAD_COMMAND: "docker exec postgres pg_ctl reload"
      SERVICE_NAME: "database"
    volumes:
      - db_certs:/certs:rw
      - /var/run/docker.sock:/var/run/docker.sock

volumes:
  db_certs:
```

## 📈 Monitoring

### Health Checks

- Certificate expiry status
- Renewal success rate
- Service availability

### Metrics

- Days until expiry
- Renewal frequency
- Failure count
- Response times

### Dashboards

- Certificate inventory
- Expiry timeline
- Alert history

## 🚀 Getting Started

### 1. Setup Step CA

```bash
# Deploy Step CA to GCP
docker run -p 9000:9000 smallstep/step-ca:latest
```

### 2. Build Cert Agent

```bash
# Build and push cert agent image
docker build -t company/cert-agent:latest .
docker push company/cert-agent:latest
```

### 3. Deploy to Services

```bash
# Add cert-agent section to existing docker-compose.yml
# Configure environment variables
# Start containers
docker-compose up -d
```

### 4. Verify Operation

```bash
# Check certificate status
docker exec cert-agent /health-check.sh

# Monitor logs
docker logs -f cert-agent

# Test certificate renewal
docker exec cert-agent /usr/local/bin/cert-agent --once
```

## 📝 Best Practices

### Configuration

- Use environment variables for all settings
- Set appropriate renewal thresholds (7+ days recommended)
- Configure proper reload commands for each service type
- Enable notifications for production environments
- Use meaningful service names for easier debugging

### Security

- Limit Docker socket access to necessary operations only
- Use read-only mounts for application containers
- Validate CA fingerprints in production
- Regular security audits of certificate usage
- Implement proper backup and recovery procedures

### Operations

- Monitor certificate inventory across all services
- Set up comprehensive alerting for renewal failures
- Plan for CA maintenance windows and service updates
- Document emergency certificate replacement procedures
- Implement automated testing of certificate renewal process

### Performance

- Stagger renewal checks across services to avoid load spikes
- Use appropriate check intervals based on certificate validity periods
- Monitor resource usage of cert-agent containers
- Implement proper logging and metrics collection

## 📋 Development Standards

### Commit Style

Follow [Conventional Commits](https://www.conventionalcommits.org/) specification:

```text
<type>[optional scope]: <description>

[optional body]

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

**Types:**

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `chore`: Maintenance tasks
- `refactor`: Code refactoring
- `test`: Test additions/modifications
- `ci`: CI/CD changes

**Examples:**

- `feat: add certificate auto-renewal for nginx services`
- `fix: resolve certificate validation timeout issue`
- `docs: update deployment guide with Kubernetes examples`
- `chore: update dependencies to latest versions`

### Code Style

**Language Standards:**

- **Code**: English only for variables, functions, comments, and documentation
- **Commit messages**: English only
- **Documentation**: English only
- **Error messages**: English only

**Rust Guidelines:**

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting (configured in `rustfmt.toml`)
- All code must pass `cargo clippy -- -D warnings`
- Maintain comprehensive documentation with `///` doc comments
- Use meaningful variable and function names in English

**Documentation:**

- Use clear, concise English
- Include examples for public APIs
- Document error conditions and edge cases
- Keep line length under 100 characters (markdown)

**Pre-commit Hooks:**

- Automatic formatting (Rust + Markdown)
- Lint checking (clippy + markdownlint)
- Trailing whitespace removal
- EOF normalization

## 🔗 References

- [Step CA Documentation](https://smallstep.com/docs/step-ca/)
- [Docker Compose Volumes](https://docs.docker.com/compose/compose-file/compose-file-v3/#volumes)
- [Certificate Management Best Practices](https://tools.ietf.org/html/rfc5280)
- [ACME Protocol Specification](https://tools.ietf.org/html/rfc8555)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
