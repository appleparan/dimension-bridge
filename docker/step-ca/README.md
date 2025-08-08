# Step CA Infrastructure

This directory deploys the Step CA server, which is the core of the PKI infrastructure.

## Quick Start

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
export STEP_CA_NAME="My Company CA"
export STEP_CA_DNS="ca.mycompany.com,localhost"
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
- **Step CA UI** (optional): <http://localhost:3000>

## Configuration

### Environment Variables

- `STEP_CA_NAME`: CA name
- `STEP_CA_DNS`: CA server DNS names (comma-separated)
- `STEP_CA_PORT`: CA server port (default: 9000)
- `STEP_CA_UI_PORT`: UI port (default: 3000)
- `STEP_CA_PASSWORD`: CA password

### After Initial Setup

After Step CA starts, get the fingerprint for use with cert-manager clients:

```bash
docker exec step-ca step ca fingerprint
```

Use this fingerprint when deploying cert-manager.
