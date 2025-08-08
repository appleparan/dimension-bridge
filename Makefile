.PHONY: help run once health validate version release test lint clean docker-dev docker-build docker-build-multiarch docker-run step-ca-setup step-ca-start step-ca-stop step-ca-logs

help:
	@echo "Dimension Bridge - PKI Certificate Auto-Renewal System"
	@echo ""
	@echo "Available targets:"
	@echo ""
	@echo "PKI Infrastructure:"
	@echo "  step-ca-setup                Setup Step CA with bootstrap client"
	@echo "  step-ca-start                Start Step CA infrastructure"
	@echo "  step-ca-stop                 Stop Step CA infrastructure"
	@echo "  step-ca-logs                 View Step CA logs"
	@echo ""
	@echo "Setup & Running:"
	@echo "  run                          Run in daemon mode (default)"
	@echo "  once                         Run certificate renewal once and exit"
	@echo "  health                       Check health status"
	@echo "  validate                     Validate configuration"
	@echo "  version                      Show version information"
	@echo ""
	@echo "Docker:"
	@echo "  docker-dev                   Run container dev env with bash"
	@echo "  docker-build                 Build Docker image"
	@echo "  docker-build-multiarch       Build multi-architecture Docker image"
	@echo "  docker-run                   Run Docker container for testing"
	@echo ""
	@echo "Quality & Testing:"
	@echo "  test                         Run all tests"
	@echo "  test-integration             Run integration tests"
	@echo "  test-cli                     Run CLI tests"
	@echo "  lint                         Run linting (fmt + clippy)"
	@echo "  fmt                          Format code"
	@echo "  clippy                       Run clippy linter"
	@echo ""
	@echo "Deployment:"
	@echo "  release                      Build release binary"
	@echo "  clean                        Clean build artifacts"

run:
	cargo run --bin dimension-bridge -- run

once:
	cargo run --bin dimension-bridge -- once

health:
	cargo run --bin dimension-bridge -- health

validate:
	cargo run --bin dimension-bridge -- validate

version:
	cargo run --bin dimension-bridge -- version

# PKI Infrastructure Management
step-ca-setup:
	@echo "Setting up Step CA infrastructure..."
	@cd docker/step-ca && \
	if [ ! -f .env ]; then \
		echo "Creating .env from template..."; \
		cp .env.example .env; \
		echo "Please edit docker/step-ca/.env with your configuration"; \
	fi
	@cd docker/step-ca && docker-compose up -d
	@echo "Waiting for Step CA to be ready..."
	@sleep 10
	@cd docker/step-ca && ./bootstrap.sh
	@echo "Step CA setup complete!"

step-ca-start:
	@cd docker/step-ca && docker-compose up -d
	@echo "Step CA started"

step-ca-stop:
	@cd docker/step-ca && docker-compose down
	@echo "Step CA stopped"

step-ca-logs:
	@cd docker/step-ca && docker-compose logs -f

docker-dev:
	@mkdir -p .cargo-cache
	docker run -it --rm --name dimension-bridge-dev --memory="4g" --cpus="2.5" \
		-v "$(PWD)":/workspace \
		-v "$(PWD)/.cargo-cache":/usr/local/cargo/registry \
		-w /workspace \
		rust:1.88-trixie \
		/bin/bash

docker-build:
	@echo "Building Docker image..."
	docker build -t appleparan/dimension-bridge:latest .
	@echo "Docker image built successfully"

docker-build-multiarch:
	@echo "Building multi-architecture Docker image..."
	docker buildx build --platform linux/amd64,linux/arm64 \
		-t appleparan/dimension-bridge:latest \
		--push .

docker-run:
	@echo "Running Docker container for testing..."
	docker run --rm --name dimension-bridge-test \
		-e SERVER_IP=test.example.com \
		-e STEP_CA_URL=https://localhost:9000 \
		-e RELOAD_COMMAND="echo Certificate updated" \
		-e RUST_LOG=info \
		-v cert-test:/certs \
		-v logs-test:/logs \
		appleparan/dimension-bridge:latest once

release:
	cargo build --release

test:
	cargo test --all

test-integration:
	cargo test --test integration_tests

test-cli:
	cargo test --test cli_tests

lint: fmt clippy

fmt:
	cargo fmt -- --check

clippy:
	cargo clippy -- -D warnings

format:
	cargo fmt

clean:
	cargo clean
