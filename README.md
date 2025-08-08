# Dimension Bridge

> Internal PKI Certificate Auto-Renewal System

Automated SSL/TLS certificate management system for internal services using centralized PKI infrastructure.

## 🚀 Quick Start

1. **Deploy Step CA Infrastructure**

   ```bash
   cd docker/step-ca/
   cp .env.example .env
   # Edit .env file with your configuration
   docker-compose up -d
   ```

2. **Build Certificate Manager**

   ```bash
   cd ../cert-manager/
   docker-compose build
   ```

3. **Deploy with Your Service**

   ```bash
   cd ../examples/nginx/
   # Edit nginx.conf with your domain names
   docker-compose up -d
   ```

## 🏗️ Architecture

```text
[Step CA Server]
    ↓ ACME/API
[Cert Manager Container] ← Deploy per service
    ↓ Volume Mount
[Application Containers] ← Shared certificate files
```

## 📁 Project Structure

```text
dimension-bridge/
├── src/                    # Rust source code
│   ├── lib.rs             # Core library
│   └── main.rs            # CLI entry point
├── tests/                 # Test suite
│   ├── integration_tests.rs
│   └── cli_tests.rs
├── docker/                # Docker deployments
│   ├── cert-manager/      # Container build & deployment
│   ├── step-ca/           # PKI infrastructure
│   └── examples/          # Usage examples
│       ├── nginx/         # Web server example
│       ├── authentik/     # SSO example
│       └── api-service/   # API service example
└── scripts/               # Build scripts
```

## 🔧 Core Features

- **Automatic Certificate Renewal**: Monitors certificate expiry and renews before expiration
- **Service Integration**: Graceful reload of services after certificate renewal
- **Multiple Deployment Patterns**: Docker Compose, Kubernetes, standalone
- **Health Monitoring**: Built-in health checks and metrics
- **Notification Support**: Slack webhook integration
- **Security Focused**: Minimal attack surface, non-root execution

## 🎯 Supported Services

- **Web Servers**: Nginx, Apache, Traefik
- **API Services**: REST APIs with HTTP reload endpoints
- **Databases**: PostgreSQL, MySQL with certificate support
- **Identity Providers**: Authentik, Keycloak
- **Monitoring**: Grafana, Prometheus

## 📋 Configuration

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `SERVER_IP` | ✅ | Domain/IP for certificate |
| `STEP_CA_URL` | ✅ | Step CA server URL |
| `RELOAD_COMMAND` | ✅ | Service reload command |
| `SERVICE_NAME` | ❌ | Service identifier |
| `CHECK_INTERVAL` | ❌ | Check frequency (default: 6h) |
| `DAYS_BEFORE_RENEWAL` | ❌ | Renewal threshold (default: 7) |
| `SLACK_WEBHOOK_URL` | ❌ | Notification webhook |

### Example Usage

```yaml
services:
  nginx:
    image: nginx:alpine
    volumes:
      - certs:/etc/ssl/certs:ro

  cert-manager:
    image: appleparan/dimension-bridge:latest
    environment:
      - SERVER_IP=web.company.internal
      - STEP_CA_URL=https://ca.company.internal:9000
      - RELOAD_COMMAND=docker exec nginx nginx -s reload
    volumes:
      - certs:/certs:rw
      - /var/run/docker.sock:/var/run/docker.sock:ro
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Run CLI tests
cargo test --test cli_tests

# Test with coverage
cargo tarpaulin --out Html
```

## 🔨 Development

### Prerequisites

- Rust 1.70+
- Docker & Docker Compose
- Step CA for testing

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Build Docker image
./scripts/build.sh
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Run pre-commit hooks
pre-commit run --all-files
```

## 📖 Documentation

- **[Docker Deployment Guide](docker/README.md)** - Complete deployment guide
- **[Usage Examples](docker/examples/README.md)** - Real-world service examples
- **[Step CA Setup](docker/step-ca/README.md)** - PKI infrastructure setup
- **[CLAUDE.md](CLAUDE.md)** - Detailed specifications and patterns

## 🔒 Security

- **Non-root execution**: Containers run as unprivileged user
- **Minimal attack surface**: Single-purpose container design
- **Read-only mounts**: Application containers have read-only certificate access
- **Docker socket restrictions**: Limited to necessary restart operations
- **Certificate validation**: CA fingerprint verification

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🤝 Contributing

1. Follow [Conventional Commits](https://www.conventionalcommits.org/) specification
2. All code must pass `cargo clippy -- -D warnings`
3. Include tests for new functionality
4. Update documentation as needed
5. Use English for all code, comments, and documentation

## 📞 Support

- Create an issue for bug reports or feature requests
- Check existing documentation before asking questions
- Include logs and configuration when reporting issues
