# Step CA Infrastructure

This directory deploys the Step CA server, which is the core of the PKI infrastructure.

## Quick Start

1. Configure environment variables:

```bash
cp .env.example .env
# Edit the .env file
```

1. Start Step CA:

```bash
docker-compose up -d
```

1. Start with UI (optional):

```bash
docker-compose --profile ui up -d
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
