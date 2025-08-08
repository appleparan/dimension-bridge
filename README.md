# Dimension Bridge

> Internal PKI Certificate Auto-Renewal System

Automated SSL/TLS certificate management system for internal services using centralized PKI infrastructure.

## 🚀 Quick Start

### Using Make (Recommended)

```bash
# 1. Setup Step CA infrastructure (one-time)
make step-ca-setup

# 2. Build the application
make release

# 3. Run once to test
make once

# 4. Run in daemon mode
make run

# 5. Check health status
make health

# 6. View all available commands
make help
```

#### PKI Infrastructure Management

```bash
# Setup Step CA with automatic bootstrap
make step-ca-setup

# Manage Step CA service
make step-ca-start    # Start Step CA
make step-ca-stop     # Stop Step CA
make step-ca-logs     # View logs
```

### Docker Deployment

> **⚠️ Important**: Before deployment, ensure you have the correct Dimension Bridge image available.
> See [DEPLOYMENT.md](DEPLOYMENT.md) for detailed image management instructions.

#### Image Requirements

```bash
# Verify required image exists
docker images | grep dimension-bridge

# Expected output:
# dimension-bridge    v1.0.0    abc123def456    2 hours ago    45.2MB
```

If the image is missing, contact your administrator or see [DEPLOYMENT.md](DEPLOYMENT.md).

1. **Deploy Step CA Infrastructure**

   ```bash
   cd docker/step-ca/
   cp .env.example .env
   # Edit .env file with your configuration
   docker-compose up -d

   # Bootstrap Step CLI client (auto-install + setup)
   ./bootstrap.sh

   # Verify connection
   step ca health
   ```

2. **Build Certificate Manager**

   ```bash
   # Build using Docker
   make docker-build

   # Or build using docker-compose
   cd docker/cert-manager/
   docker-compose build
   ```

3. **Deploy with Your Service**

   ```bash
   cd docker/examples/nginx/
   # Edit nginx.conf with your domain names
   docker-compose up -d
   ```

## 🏗️ Architecture

### System Overview

Dimension Bridge implements a **centralized PKI architecture** with **distributed certificate agents**:

```text
                    ┌─────────────────────────┐
                    │      Step CA Server      │
                    │  (Central PKI Authority) │
                    │   - Issues certificates  │
                    │   - Manages revocation   │
                    │   - ACME protocol        │
                    └─────────────┬───────────┘
                                  │ HTTPS/ACME
                    ┌─────────────┼───────────┐
                    │             │           │
            ┌───────▼───────┐ ┌───▼─────┐ ┌─▼─────────┐
            │  Cert-Manager │ │Cert-Mgr │ │Cert-Mgr   │
            │   (Agent)     │ │(Agent)  │ │(Agent)    │
            │               │ │         │ │           │
            └───────┬───────┘ └───┬─────┘ └─┬─────────┘
                    │ Mount Vol     │ Vol     │ Vol
            ┌───────▼───────┐ ┌───▼─────┐ ┌─▼─────────┐
            │  Nginx Server │ │API Svc  │ │Database   │
            │               │ │         │ │           │
            └───────────────┘ └─────────┘ └───────────┘
```

### Component Architecture

#### 🏛️ Step CA (Central Authority)

- **Single instance** serving multiple services
- **Certificate issuance** via ACME protocol
- **Centralized policy management**
- **Certificate revocation** and validation

#### 🤖 Cert-Manager (Sidecar Agents)

- **One agent per service** (sidecar pattern)
- **Automatic certificate renewal**
- **Service-specific reload commands**
- **Shared volume with target service**

### Deployment Patterns

#### Pattern 1: Sidecar Container (Recommended)

```yaml
# docker-compose.yml
services:
  nginx:                          # Your service
    image: nginx:alpine
    volumes:
      - web_certs:/etc/ssl/certs  # Shared certificate volume

  web-cert-agent:                 # Certificate agent (sidecar)
    image: appleparan/dimension-bridge:latest
    environment:
      - CERT_DOMAINS=web.company.internal
      - STEP_CA_URL=https://ca.company.internal:9000
      - RELOAD_COMMAND=docker exec nginx nginx -s reload
    volumes:
      - web_certs:/certs          # Same volume as service
```

#### Pattern 2: Multiple Services, Multiple Agents

```text
Step CA Server (ca.company.internal:9000)
├── nginx-cert-agent     → nginx (web.company.internal)
├── api-cert-agent       → api-service (api.company.internal)
├── db-cert-agent        → postgres (db.company.internal)
└── auth-cert-agent      → authentik (auth.company.internal)
```

### Benefits of This Architecture

#### 🔒 Security

- **Certificate isolation** per service
- **Minimal privilege** per agent
- **No shared secrets** between services

#### 📈 Scalability

- **Horizontal scaling** of agents
- **Independent lifecycle** per service
- **Service-specific policies**

#### 🛠️ Flexibility

- **Custom reload commands** per service type
- **Different certificate validity** periods
- **Service-specific monitoring**

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

### With Make (Recommended)

```bash
# Run all tests
make test

# Run specific test suites
make test-integration     # Integration tests only
make test-cli            # CLI tests only

# Run with coverage (manual)
cargo tarpaulin --out Html
```

### With Cargo Directly

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Run CLI tests
cargo test --test cli_tests
```

## 🔨 Development

### Prerequisites

- Rust 1.70+
- Docker & Docker Compose
- Step CA for testing

### Quick Start with Make

```bash
# View all available commands
make help

# Development workflow
make format              # Format code
make lint               # Run linting (fmt + clippy)
make test               # Run tests
make release            # Build release binary

# Running the application
make run                # Run in daemon mode
make once               # Run certificate renewal once
make health             # Check health status
make validate           # Validate configuration
make version            # Show version info
```

### Building

#### Make-based Building

```bash
# Build release binary
make release

# Build Docker image
make docker-build

# Development environment
make docker-dev         # Interactive container with Rust toolchain
```

#### Direct Cargo Usage

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### Code Quality

#### Make-based Quality Checks

```bash
# Format code
make format             # Auto-format code

# Check formatting and linting
make lint               # Run fmt --check + clippy

# Individual checks
make fmt                # Check formatting only
make clippy             # Run clippy linter only
```

#### Manual Cargo Commands

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Lint code
cargo clippy -- -D warnings

# Run pre-commit hooks
pre-commit run --all-files
```

### Docker Development

```bash
# Run development container
make docker-dev

# Build and test Docker image
make docker-build
make docker-run
```

## 📖 Documentation

- **[Docker Deployment Guide](docker/README.md)** - Complete deployment guide
- **[Usage Examples](docker/examples/README.md)** - Real-world service examples
- **[Step CA Setup](docker/step-ca/README.md)** - PKI infrastructure setup
- **[CLAUDE.md](CLAUDE.md)** - Detailed specifications and patterns

## 🛡️ Security Features

- **Non-root execution**: Containers run as unprivileged user
- **Minimal attack surface**: Single-purpose container design
- **Read-only mounts**: Application containers have read-only certificate access
- **Docker socket restrictions**: Limited to necessary restart operations
- **Certificate validation**: CA fingerprint verification

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🤝 Contributing

### Development Workflow

```bash
# 1. Clone and setup
git clone https://github.com/appleparan/dimension-bridge.git
cd dimension-bridge

# 2. Development cycle
make format              # Format your code
make lint               # Check formatting and linting
make test               # Run all tests

# 3. Before committing
make test               # Ensure all tests pass
make lint               # Ensure code quality
```

### Guidelines

1. Follow [Conventional Commits](https://www.conventionalcommits.org/) specification
2. All code must pass `make lint` (includes `cargo clippy -- -D warnings`)
3. Include tests for new functionality: `make test-integration` or `make test-cli`
4. Update documentation as needed
5. Use English for all code, comments, and documentation

### CI/CD

- **Continuous Integration**: Automated testing on push/PR via GitHub Actions
- **Release**: Multi-platform binaries (Linux/macOS, x86_64/ARM64) automatically built on release
- **Docker**: Official images built and pushed to registry

## 📞 Support

- Create an issue for bug reports or feature requests
- Check existing documentation before asking questions
- Include logs and configuration when reporting issues
