# Certificate Manager Usage Examples

This directory provides real-world examples of using dimension-bridge cert-manager with various services.

## Prerequisites

1. Step CA must be running:

   ```bash
   cd ../step-ca
   docker-compose up -d
   ```

2. Certificate manager image must be built:

   ```bash
   cd ../cert-manager
   docker-compose build
   ```

## Examples

### 🌐 [Nginx Web Server](./nginx/)

**Description**: Web server requiring SSL certificates
**Features**:

- Nginx graceful reload (`nginx -s reload`)
- HTTPS redirect configuration
- Security headers applied

**Getting Started**:

```bash
cd nginx/
# Edit domain names in nginx.conf
docker-compose up -d
```

### 🔐 [Authentik SSO](./authentik/)

**Description**: SSO authentication server
**Features**:

- Certificate renewal via service restart
- PostgreSQL + Redis dependencies
- Complex service configuration

**Getting Started**:

```bash
cd authentik/
cp .env.example .env
# Set passwords in .env file
docker-compose up -d
```

### 🚀 [API Service](./api-service/)

**Description**: REST API service
**Features**:

- Graceful reload via HTTP endpoint
- Optional Nginx proxy
- Node.js application example

**Getting Started**:

```bash
cd api-service/
docker-compose up -d

# Start with proxy
docker-compose --profile proxy up -d
```

## Common Configuration

All examples require the following environment variables:

```bash
# Step CA configuration
export STEP_CA_FINGERPRINT="your-ca-fingerprint"

# Optional: Slack notifications
export SLACK_WEBHOOK_URL="https://hooks.slack.com/services/..."
```

### Getting Step CA Fingerprint

```bash
docker exec step-ca step ca fingerprint
```

## Customization

To adapt examples to your environment:

1. **Change domain names**: Replace `mycompany.com` with your actual domain in all configurations
2. **Update Step CA URL**: Change `STEP_CA_URL` to your actual Step CA address
3. **Adjust reload commands**: Modify `RELOAD_COMMAND` to match your service
4. **Verify certificate paths**: Ensure applications use correct certificate paths

## Monitoring

Monitor certificate status by checking cert-manager container logs:

```bash
# Check specific service's cert-manager logs
docker logs -f nginx-cert-manager

# Check all cert-manager container logs
docker logs -f $(docker ps --filter name=cert-manager --format "{{.Names}}")
```

## Troubleshooting

### Certificate not generating

1. Check Step CA connection:

   ```bash
   docker exec nginx-cert-manager curl -k https://ca.mycompany.com:9000/health
   ```

2. Verify environment variables:

   ```bash
   docker exec nginx-cert-manager env | grep STEP
   ```

3. Test manual certificate generation:

   ```bash
   docker exec nginx-cert-manager ./dimension-bridge once
   ```
