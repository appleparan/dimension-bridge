# Docker Deployment Guide

Complete deployment guide for Dimension Bridge Docker containers.

## 🏗️ Architecture

```text
docker/
├── cert-manager/     # Certificate manager container build & deployment
├── step-ca/          # Step CA PKI infrastructure
└── examples/         # Real-world usage examples
    ├── nginx/        # Nginx web server example
    ├── authentik/    # Authentik SSO example
    └── api-service/  # API service example
```

## 🚀 Quick Start

### Step 1: Start PKI Infrastructure

```bash
cd step-ca/
cp .env.example .env
# Edit .env file (domain names, passwords, etc.)
docker-compose up -d
```

### Step 2: Build Cert-Manager

```bash
cd ../cert-manager/
docker-compose build
```

### Step 3: Run Service Example

```bash
cd ../examples/nginx/
# Modify domain names in nginx.conf
docker-compose up -d
```

## 📋 Detailed Guide

### [Step CA Infrastructure](./step-ca/)

- Deploy Step CA server, the core of PKI
- ACME, API-based certificate issuance
- Optional web UI included

### [Cert-Manager](./cert-manager/)

- Automatic certificate renewal container build
- Health check and monitoring included
- Service reload via Docker socket access

### [Usage Examples](./examples/)

- **Nginx**: Web server SSL certificate management
- **Authentik**: SSO server certificate management
- **API Service**: REST API service certificate management

## 🔧 Environment Variables

### Common Configuration

```bash
# Step CA configuration (required)
export STEP_CA_FINGERPRINT="$(docker exec step-ca step ca fingerprint)"

# Optional
export SLACK_WEBHOOK_URL="https://hooks.slack.com/services/..."
```

### Per-Service Configuration

Configure the following for each service:

- `SERVER_IP`: Domain/IP to include in certificate
- `SERVICE_NAME`: Certificate filename prefix
- `RELOAD_COMMAND`: Command to execute after certificate renewal

## 🔍 Monitoring

```bash
# Check all cert-manager logs
docker logs -f $(docker ps --filter name=cert-manager --format "{{.Names}}")

# Check specific service status
docker exec nginx-cert-manager ./dimension-bridge version
```

## 🛠️ Troubleshooting

### Step CA Connection Issues

```bash
# Check CA status
docker exec step-ca step ca health

# Check network connectivity
docker exec nginx-cert-manager curl -k https://step-ca:9000/health
```

### Certificate Generation Issues

```bash
# Test manual certificate generation
docker exec nginx-cert-manager ./dimension-bridge once

# Check logs
docker logs nginx-cert-manager
```

## 📦 Production Deployment

1. **Security Settings**:
   - Change Step CA password
   - Restrict Docker socket access
   - Network isolation

2. **Monitoring**:
   - Configure Slack/email notifications
   - Integrate with log collection systems
   - Monitor health check endpoints

3. **Backup**:
   - Regular Step CA data backup
   - Certificate backup policy configuration
