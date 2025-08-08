#!/bin/bash
# Health check script for cert-manager container

set -e

# Check if the binary exists and is executable
if [[ ! -x "/app/dimension-bridge" ]]; then
    echo "❌ Binary not found or not executable"
    exit 1
fi

# Check if certificate directory exists
if [[ ! -d "${CERT_DIR:-/certs}" ]]; then
    echo "❌ Certificate directory not found: ${CERT_DIR:-/certs}"
    exit 1
fi

# Check if log directory exists
if [[ ! -d "${LOG_DIR:-/logs}" ]]; then
    echo "❌ Log directory not found: ${LOG_DIR:-/logs}"
    exit 1
fi

# Test if we can run the version command
if ! timeout 5s /app/dimension-bridge version >/dev/null 2>&1; then
    echo "❌ Version command failed"
    exit 1
fi

echo "✅ Cert-manager health check passed"
exit 0
