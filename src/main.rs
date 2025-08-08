#![allow(clippy::multiple_crate_versions)]
//! Simple certificate manager - Rust port of cert-manager.sh
//!
//! Automated certificate lifecycle management using Step CLI or OpenSSL.

use dimension_bridge::{CertManager, Config};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments first
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(String::as_str);

    // Handle commands that don't need configuration
    match command {
        Some("version") => {
            println!("Simple Certificate Manager v{}", env!("CARGO_PKG_VERSION"));
            println!("Rust port of cert-manager.sh");
            println!();
            println!("Features:");
            println!("  - Step CLI integration");
            println!("  - OpenSSL fallback");
            println!("  - Automatic renewal");
            println!("  - Slack notifications");
            println!("  - Docker-friendly");
            return Ok(());
        }
        Some("help") => {
            show_help();
            return Ok(());
        }
        Some(unknown) if unknown.starts_with('-') || unknown == "invalid-command" => {
            eprintln!("Unknown command: {unknown}");
            show_help();
            std::process::exit(1);
        }
        _ => {} // Continue with initialization for other commands
    }

    // Initialize logging
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned());
    tracing_subscriber::fmt().with_env_filter(log_level).init();

    // Load configuration (required for all other commands)
    let config = Config::from_env()?;
    let manager = CertManager::new(config);

    // Initialize
    manager.initialize().await?;

    // Handle commands that need configuration
    match command {
        Some("once") => {
            manager.run_once().await?;
        }
        Some(command) => {
            eprintln!("Unknown command: {command}");
            show_help();
            std::process::exit(1);
        }
        None => {
            // Default: run daemon
            manager.run().await?;
        }
    }

    Ok(())
}

fn show_help() {
    println!("Simple Certificate Manager v{}", env!("CARGO_PKG_VERSION"));
    println!("Automated certificate lifecycle management");
    println!();
    println!("USAGE:");
    println!("    dimension-bridge [COMMAND]");
    println!();
    println!("COMMANDS:");
    println!("    <none>      Run continuously (daemon mode)");
    println!("    once        Run once and exit");
    println!("    version     Show version information");
    println!("    help        Show this help message");
    println!();
    println!("ENVIRONMENT VARIABLES:");
    println!("    SERVER_IP             Server IP for certificate SAN (required)");
    println!("    SERVICE_NAME          Service name for certificate files (default: cert-agent)");
    println!("    CERT_DIR              Certificate directory (default: /certs)");
    println!("    CHECK_INTERVAL        Check interval in seconds (default: 86400)");
    println!("    DAYS_BEFORE_RENEWAL   Days before expiry to renew (default: 5)");
    println!("    CERT_VALIDITY_DAYS    Certificate validity in days (default: 15)");
    println!("    RELOAD_COMMAND        Command to reload service (optional)");
    println!("    SLACK_WEBHOOK_URL     Slack webhook for notifications (optional)");
    println!("    RUST_LOG              Log level (default: info)");
}
