# Multi-stage build for Simple Certificate Manager
FROM rust:1.88-trixie AS builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /usr/src/app

# Copy manifests first for better layer caching
COPY Cargo.toml ./
COPY Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Runtime image
FROM debian:trixie-slim AS runtime

# Install runtime dependencies including Step CLI and OpenSSL
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    openssl \
    wget \
    && wget -O step.deb https://dl.step.sm/gh-release/cli/docs-cli-install/v0.25.2/step-cli_0.25.2_amd64.deb \
    && dpkg -i step.deb \
    && rm step.deb \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN groupadd -r certmanager && useradd -r -g certmanager certmanager

# Create directories
RUN mkdir -p /certs /logs /app && \
    chown -R certmanager:certmanager /certs /logs /app

# Copy the binary from builder stage
COPY --from=builder /usr/src/app/target/release/dimension-bridge /app/cert-manager

# Make binary executable
RUN chmod +x /app/cert-manager

# Create simple health check script
COPY <<'EOF' /app/health-check.sh
#!/bin/bash
set -e

# Check if the binary is running
pgrep -f cert-manager > /dev/null || exit 1

# Check if certificate directory is accessible
[ -d "$CERT_DIR" ] || exit 1

echo "Health check passed"
exit 0
EOF

RUN chmod +x /app/health-check.sh

# Create entrypoint script
COPY <<'EOF' /app/entrypoint.sh
#!/bin/bash
set -e

# Print version information
echo "Starting Simple Certificate Manager"
/app/cert-manager version

# Validate required environment variables
if [ -z "$SERVER_IP" ] && [ -z "$CERT_DOMAINS" ]; then
    echo "ERROR: SERVER_IP or CERT_DOMAINS environment variable is required"
    exit 1
fi

# Start the application
echo "Starting certificate manager with command: ${1:-run}"
exec /app/cert-manager "$@"
EOF

RUN chmod +x /app/entrypoint.sh && \
    chown -R certmanager:certmanager /app

# Set up volumes
VOLUME ["/certs", "/logs"]

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD /app/health-check.sh

# Switch to non-root user
USER certmanager

# Set working directory
WORKDIR /app

# Set environment variables with defaults
ENV CERT_DIR=/certs
ENV LOG_DIR=/logs
ENV CHECK_INTERVAL=86400
ENV DAYS_BEFORE_RENEWAL=5
ENV CERT_VALIDITY_DAYS=15
ENV SERVICE_NAME=cert-agent
ENV RUST_LOG=info

# Use the entrypoint script
ENTRYPOINT ["/app/entrypoint.sh"]

# Default command is to run in daemon mode
CMD ["run"]
