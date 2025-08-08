.PHONY: help run once health validate version release test lint clean docker-dev docker-build docker-run

help:
	@echo "Dimension Bridge - PKI Certificate Auto-Renewal System"
	@echo ""
	@echo "Available targets:"
	@echo ""
	@echo "Setup & Running:"
	@echo "  run                          Run in daemon mode (default)"
	@echo "  once                         Run certificate renewal once and exit"
	@echo "  health                       Check health status"
	@echo "  validate                     Validate configuration"
	@echo "  version                      Show version information"
	@echo "  docker-dev                   Run container dev env with bash"
	@echo "  docker-build                 Build Docker image"
	@echo "  docker-run                   Run Docker container"
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

docker-dev:
	@mkdir -p .cargo-cache
	docker run -it --rm --name dimension-bridge-dev --memory="4g" --cpus="2.5" \
		-v "$(PWD)":/workspace \
		-v "$(PWD)/.cargo-cache":/usr/local/cargo/registry \
		-w /workspace \
		rust:1.75 \
		/bin/bash

docker-build:
	docker build -t appleparan/dimension-bridge:latest .

docker-run:
	docker run --rm --name dimension-bridge-test \
		-e SERVER_IP=test.example.com \
		-e STEP_CA_URL=https://ca.example.com:9000 \
		-e RELOAD_COMMAND="echo Certificate updated" \
		-v cert-test:/certs \
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
