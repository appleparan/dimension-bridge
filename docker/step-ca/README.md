# Step CA Infrastructure

This directory deploys the Step CA server, which is the core of the PKI infrastructure.

## Quick Start

1. Configure environment (optional):

```bash
cp .env.example .env
# Edit .env file with your settings
```

1. Start Step CA:

```bash
docker-compose up -d
```

1. Install step CLI (if not installed):

```bash
# Option 1: Auto-install with bootstrap
./bootstrap.sh

# Option 2: Install separately
sudo ./install-step-cli.sh
./bootstrap.sh
```

1. Verify connection:

```bash
step ca health
```

1. (Optional) Configure environment variables:

```bash
# Required
export STEP_CA_NAME="My Company CA"                    # CA name (issuer name)
export STEP_CA_DNS="ca.mycompany.com,localhost"       # Hostnames/IPs CA accepts requests on

# Optional
export STEP_CA_PORT="9000"                             # Port to expose (default: 9000)
```

## Environment Variables

The Step CA container supports the following initialization variables:

### Required Variables

- **`DOCKER_STEPCA_INIT_NAME`**: The name of your CA (issuer of certificates)
- **`DOCKER_STEPCA_INIT_DNS_NAMES`**: Hostname(s) or IPs that the CA will accept requests on

### Recommended Variables

- **`DOCKER_STEPCA_INIT_REMOTE_MANAGEMENT=true`**: Enable remote provisioner management
- **`DOCKER_STEPCA_INIT_ACME=true`**: Create initial ACME provisioner for automated certificates

### Optional Variables

- **`DOCKER_STEPCA_INIT_PROVISIONER_NAME`**: Label for initial admin provisioner (default: "admin")
- **`DOCKER_STEPCA_INIT_SSH`**: Set to enable SSH certificate support
- **`DOCKER_STEPCA_INIT_PASSWORD_FILE`**: Path to password file (recommended for production)
- **`DOCKER_STEPCA_INIT_PASSWORD`**: CA password (not recommended - use password file instead)

### Docker Compose Environment Variables

These are mapped in the docker-compose.yml:

```bash
# Used by docker-compose.yml
STEP_CA_NAME="My Company CA"           # → DOCKER_STEPCA_INIT_NAME
STEP_CA_DNS="ca.company.com,localhost" # → DOCKER_STEPCA_INIT_DNS_NAMES
STEP_CA_PORT="9000"                    # → Container port mapping
```

**Note**: These variables are only evaluated once during the CA's first initialization.

### Production Configuration Example

```bash
# .env file for production
STEP_CA_NAME="Company Internal PKI"
STEP_CA_DNS="ca.company.internal,ca-backup.company.internal"
STEP_CA_PORT="9000"
STEP_CA_PROVISIONER_NAME="company-admin"
STEP_CA_SSH_SUPPORT="true"
STEP_CA_PASSWORD_FILE="/run/secrets/ca-password"  # Docker secret
```

### Development Configuration Example

```bash
# .env file for development
STEP_CA_NAME="Development CA"
STEP_CA_DNS="localhost,127.0.0.1,ca.dev.local"
STEP_CA_PORT="9000"
```

## Manual Installation (Alternative)

### Install Step CLI manually

**Ubuntu/Debian:**

```bash
sudo apt-get update && sudo apt-get install -y --no-install-recommends curl gpg ca-certificates
curl -fsSL https://packages.smallstep.com/keys/apt/repo-signing-key.gpg | sudo tee /etc/apt/trusted.gpg.d/smallstep.asc
echo 'deb [signed-by=/etc/apt/trusted.gpg.d/smallstep.asc] https://packages.smallstep.com/stable/debian debs main' | sudo tee /etc/apt/sources.list.d/smallstep.list
sudo apt-get update && sudo apt-get -y install step-cli
```

**macOS:**

```bash
brew install step
```

### Bootstrap manually

```bash
# Get CA fingerprint
CA_FINGERPRINT=$(docker exec step-ca step certificate fingerprint certs/root_ca.crt)

# Bootstrap step client
step ca bootstrap --ca-url https://localhost:9000 --fingerprint $CA_FINGERPRINT --install
```

## Access Information

- **Step CA API**: <https://localhost:9000>

## Usage

### Get CA Fingerprint

After Step CA starts, get the fingerprint for use with cert-manager clients:

```bash
docker exec step-ca step ca fingerprint
```

Use this fingerprint when deploying cert-manager.
