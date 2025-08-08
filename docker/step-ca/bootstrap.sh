#!/bin/bash
set -e

# Step CA Client Bootstrap Script
# This script bootstraps a step client on the host to connect to the Step CA running in Docker

echo "🚀 Step CA Client Bootstrap Script"
echo "=================================="

# Configuration
CA_URL="${STEP_CA_URL:-https://localhost:9000}"
CA_CONTAINER="${STEP_CA_CONTAINER:-step-ca}"

# Check if Step CA container is running
echo "📋 Checking Step CA container status..."
if ! docker ps | grep -q "$CA_CONTAINER"; then
    echo "❌ Error: Step CA container '$CA_CONTAINER' is not running"
    echo "   Please start the Step CA first:"
    echo "   docker-compose up -d"
    exit 1
fi

# Wait for Step CA to be healthy
echo "⏳ Waiting for Step CA to be ready..."
timeout=60
counter=0
while [ $counter -lt $timeout ]; do
    if docker exec "$CA_CONTAINER" step ca health >/dev/null 2>&1; then
        echo "✅ Step CA is healthy"
        break
    fi
    echo "   Waiting... ($counter/$timeout)"
    sleep 2
    counter=$((counter + 2))
done

if [ $counter -ge $timeout ]; then
    echo "❌ Error: Step CA did not become healthy within $timeout seconds"
    exit 1
fi

# Get the CA fingerprint
echo "🔍 Getting CA fingerprint..."
CA_FINGERPRINT=$(docker exec "$CA_CONTAINER" step certificate fingerprint certs/root_ca.crt)

if [ -z "$CA_FINGERPRINT" ]; then
    echo "❌ Error: Failed to get CA fingerprint"
    exit 1
fi

echo "📄 CA Fingerprint: $CA_FINGERPRINT"

# Check if step CLI is installed on host
if ! command -v step >/dev/null 2>&1; then
    echo "📦 Step CLI not found. Installing..."

    # Detect OS
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        if command -v brew >/dev/null 2>&1; then
            echo "🍺 Installing Step CLI via Homebrew..."
            brew install step
        else
            echo "❌ Error: Homebrew not found on macOS"
            echo "Please install Homebrew first: https://brew.sh/"
            exit 1
        fi
    elif [[ -f /etc/debian_version ]]; then
        # Ubuntu/Debian
        echo "🐧 Installing Step CLI on Debian/Ubuntu..."
        if [ "$EUID" -ne 0 ]; then
            echo "❌ Error: Root privileges required for installation"
            echo "Please run with sudo or install manually:"
            echo ""
            echo "sudo apt-get update && sudo apt-get install -y --no-install-recommends curl gpg ca-certificates"
            echo "curl -fsSL https://packages.smallstep.com/keys/apt/repo-signing-key.gpg -o /etc/apt/trusted.gpg.d/smallstep.asc"
            echo "echo 'deb [signed-by=/etc/apt/trusted.gpg.d/smallstep.asc] https://packages.smallstep.com/stable/debian debs main' | sudo tee /etc/apt/sources.list.d/smallstep.list"
            echo "sudo apt-get update && sudo apt-get -y install step-cli"
            exit 1
        fi

        apt-get update && apt-get install -y --no-install-recommends curl gpg ca-certificates
        curl -fsSL https://packages.smallstep.com/keys/apt/repo-signing-key.gpg -o /etc/apt/trusted.gpg.d/smallstep.asc
        echo 'deb [signed-by=/etc/apt/trusted.gpg.d/smallstep.asc] https://packages.smallstep.com/stable/debian debs main' | tee /etc/apt/sources.list.d/smallstep.list
        apt-get update && apt-get -y install step-cli

        echo "✅ Step CLI installed successfully"
    elif [[ -f /etc/redhat-release ]]; then
        # RedHat/CentOS/Fedora
        echo "🎩 RedHat-based system detected"
        echo "Please install Step CLI manually:"
        echo "  # Add Smallstep repository"
        echo "  curl -fsSL https://packages.smallstep.com/keys/rpm/repo-signing-key.gpg | sudo rpm --import -"
        echo "  echo '[smallstep]' | sudo tee /etc/yum.repos.d/smallstep.repo"
        echo "  echo 'name=Smallstep' | sudo tee -a /etc/yum.repos.d/smallstep.repo"
        echo "  echo 'baseurl=https://packages.smallstep.com/stable/rpm' | sudo tee -a /etc/yum.repos.d/smallstep.repo"
        echo "  echo 'enabled=1' | sudo tee -a /etc/yum.repos.d/smallstep.repo"
        echo "  echo 'gpgcheck=1' | sudo tee -a /etc/yum.repos.d/smallstep.repo"
        echo "  sudo yum install -y step-cli"
        exit 1
    else
        # Unknown OS
        echo "❌ Error: Unknown operating system"
        echo "Please install Step CLI manually:"
        echo "  Visit: https://smallstep.com/docs/step-cli/installation/"
        echo ""
        echo "Or download from: https://github.com/smallstep/cli/releases"
        exit 1
    fi
fi

# Bootstrap the step client
echo "🔧 Bootstrapping step client..."
if step ca bootstrap --ca-url "$CA_URL" --fingerprint "$CA_FINGERPRINT" --install --force; then
    echo "✅ Step client bootstrapped successfully!"
    echo ""
    echo "🎉 Setup Complete!"
    echo "=================="
    echo "CA URL: $CA_URL"
    echo "Fingerprint: $CA_FINGERPRINT"
    echo "Root CA certificate has been installed in the host trust store."
    echo ""
    echo "📝 Next Steps:"
    echo "1. Test the connection:"
    echo "   step ca health"
    echo ""
    echo "2. Generate a certificate:"
    echo "   step ca certificate myservice.local myservice.crt myservice.key"
    echo ""
    echo "3. Start your cert-manager containers to manage certificates automatically"
else
    echo "❌ Error: Failed to bootstrap step client"
    exit 1
fi
