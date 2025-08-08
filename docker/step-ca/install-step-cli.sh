#!/bin/bash
set -e

# Step CLI Installation Script
# This script installs the step CLI on various operating systems

echo "📦 Step CLI Installation Script"
echo "==============================="

# Check if already installed
if command -v step >/dev/null 2>&1; then
    echo "✅ Step CLI is already installed: $(step version)"
    exit 0
fi

# Detect OS
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    echo "🍎 macOS detected"
    if command -v brew >/dev/null 2>&1; then
        echo "🍺 Installing Step CLI via Homebrew..."
        brew install step
    else
        echo "❌ Error: Homebrew not found"
        echo "Please install Homebrew first: https://brew.sh/"
        echo ""
        echo "Alternative: Download from https://github.com/smallstep/cli/releases"
        exit 1
    fi

elif [[ -f /etc/debian_version ]]; then
    # Ubuntu/Debian
    echo "🐧 Debian/Ubuntu detected"
    echo "Installing Step CLI via official repository..."

    if [ "$EUID" -ne 0 ]; then
        echo "🔐 This script requires root privileges for system-wide installation"
        echo "Run with sudo: sudo $0"
        echo ""
        echo "Or install manually:"
        echo "sudo apt-get update && sudo apt-get install -y --no-install-recommends curl gpg ca-certificates"
        echo "curl -fsSL https://packages.smallstep.com/keys/apt/repo-signing-key.gpg | sudo tee /etc/apt/trusted.gpg.d/smallstep.asc"
        echo "echo 'deb [signed-by=/etc/apt/trusted.gpg.d/smallstep.asc] https://packages.smallstep.com/stable/debian debs main' | sudo tee /etc/apt/sources.list.d/smallstep.list"
        echo "sudo apt-get update && sudo apt-get -y install step-cli"
        exit 1
    fi

    # Install prerequisites
    echo "📋 Installing prerequisites..."
    apt-get update && apt-get install -y --no-install-recommends curl gpg ca-certificates

    # Add Smallstep repository
    echo "🔑 Adding Smallstep GPG key..."
    curl -fsSL https://packages.smallstep.com/keys/apt/repo-signing-key.gpg -o /etc/apt/trusted.gpg.d/smallstep.asc

    echo "📝 Adding Smallstep repository..."
    echo 'deb [signed-by=/etc/apt/trusted.gpg.d/smallstep.asc] https://packages.smallstep.com/stable/debian debs main' | tee /etc/apt/sources.list.d/smallstep.list

    # Install step-cli
    echo "⬇️ Installing step-cli..."
    apt-get update && apt-get -y install step-cli

elif [[ -f /etc/redhat-release ]]; then
    # RedHat/CentOS/Fedora
    echo "🎩 RedHat-based system detected"

    if [ "$EUID" -ne 0 ]; then
        echo "🔐 This script requires root privileges for system-wide installation"
        echo "Run with sudo: sudo $0"
        exit 1
    fi

    echo "🔑 Adding Smallstep GPG key..."
    curl -fsSL https://packages.smallstep.com/keys/rpm/repo-signing-key.gpg | rpm --import -

    echo "📝 Adding Smallstep repository..."
    cat > /etc/yum.repos.d/smallstep.repo <<EOF
[smallstep]
name=Smallstep
baseurl=https://packages.smallstep.com/stable/rpm
enabled=1
gpgcheck=1
EOF

    # Detect package manager
    if command -v dnf >/dev/null 2>&1; then
        echo "⬇️ Installing step-cli via dnf..."
        dnf install -y step-cli
    elif command -v yum >/dev/null 2>&1; then
        echo "⬇️ Installing step-cli via yum..."
        yum install -y step-cli
    else
        echo "❌ Error: Neither dnf nor yum found"
        exit 1
    fi

else
    # Unknown OS - provide manual installation options
    echo "❓ Unknown operating system detected"
    echo "Please install Step CLI manually using one of these methods:"
    echo ""
    echo "🌐 Official installation guide:"
    echo "   https://smallstep.com/docs/step-cli/installation/"
    echo ""
    echo "📦 Download binary from GitHub releases:"
    echo "   https://github.com/smallstep/cli/releases"
    echo ""
    echo "🐳 Use Docker (as alternative):"
    echo "   docker run --rm -it smallstep/step-ca step version"
    exit 1
fi

# Verify installation
echo "🔍 Verifying installation..."
if command -v step >/dev/null 2>&1; then
    echo "✅ Step CLI installed successfully!"
    echo "📄 Version: $(step version)"
    echo ""
    echo "🎉 Next steps:"
    echo "1. Return to the main directory: cd .."
    echo "2. Start Step CA: docker-compose up -d"
    echo "3. Bootstrap client: ./bootstrap.sh"
else
    echo "❌ Installation failed - step command not found"
    exit 1
fi
