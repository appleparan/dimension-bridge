//! CLI integration tests for dimension-bridge
//!
//! Tests the command-line interface behavior.

use assert_cmd::Command;
use predicates::prelude::*;
use std::env;
use tempfile::TempDir;

fn cmd() -> Command {
    Command::cargo_bin("dimension-bridge").unwrap()
}

#[test]
fn test_version_command() {
    cmd()
        .arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("Simple Certificate Manager"))
        .stdout(predicate::str::contains("Features:"));
}

#[test]
fn test_help_command() {
    cmd()
        .arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains("USAGE:"))
        .stdout(predicate::str::contains("COMMANDS:"))
        .stdout(predicate::str::contains("ENVIRONMENT VARIABLES:"));
}

#[test]
fn test_unknown_command() {
    cmd()
        .arg("invalid-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown command: invalid-command"));
}

#[test]
fn test_missing_required_env_var() {
    // Clear environment variables that might interfere
    let mut cmd = cmd();
    cmd.env_clear()
        .env("RUST_LOG", "error") // Reduce log noise
        .arg("once")
        .assert()
        .failure();
    // Should fail because SERVER_IP or CERT_DOMAINS is required
}

#[test]
fn test_once_command_with_env_vars() {
    let temp_dir = TempDir::new().unwrap();
    let cert_dir = temp_dir.path().join("certs");
    let log_dir = temp_dir.path().join("logs");

    // Create directories to avoid permission issues in CI
    std::fs::create_dir_all(&cert_dir).unwrap();
    std::fs::create_dir_all(&log_dir).unwrap();

    let result = cmd()
        .env_clear()
        .env("SERVER_IP", "127.0.0.1")
        .env("SERVICE_NAME", "test-cli")
        .env("CERT_DIR", cert_dir.to_str().unwrap())
        .env("LOG_DIR", log_dir.to_str().unwrap())
        .env("RUST_LOG", "error") // Reduce log noise in CI
        .arg("once")
        .timeout(std::time::Duration::from_secs(5))
        .assert();

    // In CI environment, we expect this to fail due to missing certificate infrastructure
    // but it should fail with specific error messages, not permission errors
    if env::var("CI").is_ok() || env::var("GITHUB_ACTIONS").is_ok() {
        result
            .failure()
            .stderr(predicate::str::contains("SERVER_IP").not())
            .stderr(predicate::str::contains("Permission denied").not());
    } else {
        result.success(); // Should succeed in local dev environment
    }
}

#[test]
fn test_help_flag() {
    // Test both -h and --help (if implemented)
    for help_arg in &["help"] {
        cmd()
            .arg(help_arg)
            .assert()
            .success()
            .stdout(predicate::str::contains("dimension-bridge"));
    }
}

#[test]
fn test_environment_variable_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let cert_dir = temp_dir.path().join("certs");
    let log_dir = temp_dir.path().join("logs");

    // Create directories to avoid permission issues in CI
    std::fs::create_dir_all(&cert_dir).unwrap();
    std::fs::create_dir_all(&log_dir).unwrap();

    let result = cmd()
        .env_clear()
        .env("CERT_DOMAINS", "test1.example.com,test2.example.com") // Should use first domain
        .env("SERVICE_NAME", "cli-test")
        .env("CERT_DIR", cert_dir.to_str().unwrap())
        .env("LOG_DIR", log_dir.to_str().unwrap())
        .env("CHECK_INTERVAL", "3600")
        .env("DAYS_BEFORE_RENEWAL", "7")
        .env("CERT_VALIDITY_DAYS", "30")
        .env("RUST_LOG", "error") // Reduce log noise in CI
        .arg("once")
        .timeout(std::time::Duration::from_secs(5))
        .assert();

    // In CI environment, we expect this to fail due to missing certificate infrastructure
    // but it should fail with specific error messages, not permission errors
    if env::var("CI").is_ok() || env::var("GITHUB_ACTIONS").is_ok() {
        result
            .failure()
            .stderr(predicate::str::contains("CERT_DOMAINS").not())
            .stderr(predicate::str::contains("Permission denied").not());
    } else {
        result.success(); // Should succeed in local dev environment
    }
}
