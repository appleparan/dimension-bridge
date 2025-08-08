# Cert Manager - PKI Certificate Auto-Renewal Container
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false -m -d /app cert-manager

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src/ src/

# Build the application
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Step CLI
RUN curl -sL https://github.com/smallstep/cli/releases/download/v0.25.2/step-cli_0.25.2_amd64.deb \
    -o step-cli.deb \
    && dpkg -i step-cli.deb \
    && rm step-cli.deb

# Create app user
RUN useradd -r -s /bin/false -m -d /app cert-manager \
    && mkdir -p /certs /logs \
    && chown -R cert-manager:cert-manager /app /certs /logs

# Copy binary from builder
COPY --from=builder /app/target/release/dimension-bridge /app/dimension-bridge

# Add health check script
COPY docker/cert-manager/health-check.sh /health-check.sh
RUN chmod +x /health-check.sh

# Switch to non-root user
USER cert-manager
WORKDIR /app

# Default environment variables
ENV RUST_LOG=info
ENV CERT_DIR=/certs
ENV LOG_DIR=/logs
ENV CHECK_INTERVAL=86400
ENV DAYS_BEFORE_RENEWAL=5
ENV CERT_VALIDITY_DAYS=15

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ["/health-check.sh"]

# Default command (daemon mode)
CMD ["./dimension-bridge"]
