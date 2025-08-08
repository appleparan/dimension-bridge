# Dimension Bridge - Agent Specification

## 🎯 Project Goals

**Primary Objective**: Automated internal PKI certificate management system that
integrates seamlessly with existing services without manual intervention.

**Key Requirements**:

- Zero-downtime certificate renewals
- Sidecar container pattern for easy integration
- Support for diverse service reload mechanisms
- Production-ready security and monitoring
- Minimal configuration overhead

## 🏗️ System Architecture

```text
[Step CA Server] ← Centralized PKI Authority
    ↓ ACME/API Protocol
[Cert Manager] ← Per-service sidecar container
    ↓ Shared Volume Mount
[Target Service] ← Your application (nginx, api, db, etc.)
```

## 🔧 Agent Specifications

### Core Functionality

- **Certificate Lifecycle Management**: Generate → Monitor → Renew → Deploy → Reload
- **Service Integration**: Execute custom reload commands post-renewal
- **Health Monitoring**: Built-in health checks and failure notifications
- **Security**: Non-root execution, minimal permissions, read-only mounts

### Deployment Patterns

1. **Sidecar Container**: One cert-manager per service (recommended)
2. **Shared Agent**: One cert-manager for multiple related services
3. **Standalone**: Direct host deployment for legacy systems

## 📋 Integration Guide

### Essential Configuration

| Variable | Required | Purpose | Example |
|----------|----------|---------|---------|
| `SERVER_IP` | ✅ | Certificate domain/IP | `api.company.internal` |
| `STEP_CA_URL` | ✅ | PKI server endpoint | `https://ca.company.internal:9000` |
| `RELOAD_COMMAND` | ✅ | Service reload method | `docker exec nginx nginx -s reload` |
| `SERVICE_NAME` | ❌ | Identifier for logging | `nginx-web-server` |

### Service Reload Patterns

**Web Servers**: Graceful reload

```bash
RELOAD_COMMAND="docker exec nginx nginx -s reload"
```

**API Services**: HTTP endpoint trigger

```bash
RELOAD_COMMAND="curl -X POST http://api:8080/reload-ssl"
```

**Databases**: Service restart

```bash
RELOAD_COMMAND="docker restart postgres"
```

**Multiple Services**: Combined commands

```bash
RELOAD_COMMAND="docker exec nginx nginx -s reload && curl -X POST http://api:3000/reload"
```

## ⚙️ Operational Requirements

### Certificate Lifecycle

- **Monitoring Frequency**: Every 6 hours (configurable)
- **Renewal Threshold**: 7 days before expiry (configurable)
- **Certificate Validity**: 15-30 days (Step CA controlled)
- **Backup Strategy**: Previous certificate kept as `.crt.old`

### File Structure Standard

```text
/certs/
├── {service-name}.crt    # Certificate file
├── {service-name}.key    # Private key (600 permissions)
├── ca.crt                # CA certificate
└── .metadata/            # Internal agent data
    ├── last_renewal.json
    └── status.json
```

### Security Requirements

- **Container Security**: Non-root user (UID 1000)
- **File Permissions**: Keys 600, certificates 644
- **Docker Access**: Read-only socket mount for service management
- **Network**: Internal Docker networks only
- **CA Validation**: Fingerprint verification required

## 🎛️ Service Integration Patterns

### Pattern 1: Web Servers (Nginx, Apache, Traefik)

- **Reload Method**: Graceful reload signal
- **Certificate Path**: `/etc/ssl/certs/` (read-only mount)
- **Downtime**: Zero (graceful reload)

### Pattern 2: API Services (REST APIs, Microservices)

- **Reload Method**: HTTP endpoint trigger
- **Certificate Path**: Application-specific
- **Downtime**: Zero (hot reload)

### Pattern 3: Databases (PostgreSQL, MySQL)

- **Reload Method**: Service restart or reload signal
- **Certificate Path**: Database-specific directory
- **Downtime**: Brief (restart required)

### Pattern 4: Identity Providers (Authentik, Keycloak)

- **Reload Method**: Container restart
- **Certificate Path**: Application config directory
- **Downtime**: Brief (restart required)

## 🚨 Error Handling & Monitoring

### Failure Recovery

- **Automatic Rollback**: Revert to previous certificate on reload failure
- **Retry Logic**: Exponential backoff for transient failures
- **Manual Override**: Emergency certificate replacement capability
- **Health Monitoring**: HTTP endpoint at `:8080/health`

### Notification Requirements

- **Success**: Certificate renewed successfully
- **Failure**: Renewal failed (with error details)
- **Warning**: Certificate expiring soon (configurable threshold)
- **Critical**: Service reload failed after renewal

### Monitoring Integration

- **Health Checks**: Container health endpoint with detailed status
- **Metrics**: Certificate expiry days, renewal success rate, failure count
- **Logging**: Structured JSON logs with correlation IDs
- **Alerting**: Slack webhook integration (expandable)

## 🚀 Deployment Checklist

### Pre-deployment

- [ ] Step CA server deployed and accessible
- [ ] Service-specific reload commands tested
- [ ] Certificate file paths configured in target services
- [ ] Shared volumes defined in docker-compose
- [ ] Network connectivity verified between components

### Post-deployment Validation

- [ ] Certificate generation successful
- [ ] Service reload command executes without errors
- [ ] Health check endpoint responding (`:8080/health`)
- [ ] Log output shows successful certificate lifecycle
- [ ] Notification delivery tested (if configured)

### Production Readiness

- [ ] Certificate expiry monitoring configured
- [ ] Backup/rollback procedures documented
- [ ] Emergency certificate replacement process tested
- [ ] Service dependency mapping completed
- [ ] Disaster recovery plan includes certificate management

## 🎯 Implementation Priorities

### Phase 1: Core Functionality (Implemented)

- [x] Certificate generation and renewal logic
- [x] Step CA integration via ACME protocol
- [x] Service reload command execution
- [x] Basic health monitoring
- [x] Docker container deployment

### Phase 2: Production Features (In Progress)

- [ ] Comprehensive error handling and rollback
- [ ] Slack notification integration
- [ ] Prometheus metrics export
- [ ] Certificate backup and recovery
- [ ] Performance optimization

### Phase 3: Advanced Features (Planned)

- [ ] Kubernetes CRD integration
- [ ] Multi-CA support
- [ ] Certificate policy enforcement
- [ ] Automated service discovery
- [ ] Dashboard UI for certificate management

## 📖 Reference Documentation

- **[Docker Deployment Guide](docker/README.md)** - Complete deployment instructions
- **[Usage Examples](docker/examples/README.md)** - Service integration examples
- **[Step CA Setup](docker/step-ca/README.md)** - PKI infrastructure guide
- **[Step CA Documentation](https://smallstep.com/docs/step-ca/)** - Official CA documentation
- **[ACME Protocol](https://tools.ietf.org/html/rfc8555)** - Certificate automation standard

---

**Key Design Principle**: Minimize configuration complexity while maximizing
integration flexibility. The agent should "just work" with sensible defaults
while supporting diverse service architectures.
