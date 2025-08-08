#![allow(clippy::multiple_crate_versions)]
//! Simple certificate manager - Rust port of cert-manager.sh
//!
//! Automated certificate lifecycle management using Step CLI or OpenSSL.

use chrono::{DateTime, Utc};
use serde_json::json;
use std::{env, process::Command, time::Duration};
use tokio::{fs, time::sleep};
use tracing::{debug, error, info, warn};

/// Certificate manager configuration.
#[derive(Debug, Clone)]
struct Config {
    /// Certificate directory.
    cert_dir: String,
    /// Log directory.
    log_dir: String,
    /// Check interval in seconds.
    check_interval: u64,
    /// Days before expiry to renew.
    days_before_renewal: i64,
    /// Certificate validity in days.
    cert_validity_days: u32,
    /// Server IP for SAN.
    server_ip: String,
    /// Service name for the certificate.
    service_name: String,
    /// Slack webhook URL for notifications.
    slack_webhook_url: Option<String>,
}

impl Config {
    /// Load configuration from environment variables.
    fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let server_ip = env::var("SERVER_IP")
            .or_else(|_| {
                env::var("CERT_DOMAINS")
                    .map(|domains| domains.split(',').next().unwrap_or("localhost").to_string())
            })
            .map_err(|_| "SERVER_IP or CERT_DOMAINS environment variable is required")?;

        let service_name = env::var("SERVICE_NAME").unwrap_or_else(|_| "cert-agent".to_string());

        Ok(Self {
            cert_dir: env::var("CERT_DIR").unwrap_or_else(|_| "/certs".to_string()),
            log_dir: env::var("LOG_DIR").unwrap_or_else(|_| "/logs".to_string()),
            check_interval: env::var("CHECK_INTERVAL")
                .unwrap_or_else(|_| "86400".to_string())
                .parse::<u64>()
                .unwrap_or(86400),
            days_before_renewal: env::var("DAYS_BEFORE_RENEWAL")
                .unwrap_or_else(|_| "5".to_string())
                .parse::<i64>()
                .unwrap_or(5),
            cert_validity_days: env::var("CERT_VALIDITY_DAYS")
                .unwrap_or_else(|_| "15".to_string())
                .parse::<u32>()
                .unwrap_or(15),
            server_ip,
            service_name,
            slack_webhook_url: env::var("SLACK_WEBHOOK_URL").ok(),
        })
    }
}

/// Main certificate manager.
struct CertManager {
    config: Config,
    http_client: reqwest::Client,
}

impl CertManager {
    /// Create a new certificate manager.
    fn new(config: Config) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }

    /// Initialize the certificate manager.
    async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🚀 Certificate Manager Container Starting");
        info!(
            "Config: CHECK_INTERVAL={}s, DAYS_BEFORE_RENEWAL={}, VALIDITY={} days",
            self.config.check_interval,
            self.config.days_before_renewal,
            self.config.cert_validity_days
        );

        // Create required directories
        fs::create_dir_all(&self.config.cert_dir).await?;
        fs::create_dir_all(&self.config.log_dir).await?;

        info!("Server IP: {}", self.config.server_ip);
        info!("Service Name: {}", self.config.service_name);

        Ok(())
    }

    /// Check certificate expiry.
    async fn check_cert_expiry(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let cert_file = format!("{}/{}.crt", self.config.cert_dir, self.config.service_name);

        if fs::metadata(&cert_file).await.is_err() {
            warn!("Certificate file not found: {cert_file}");
            return Ok(false);
        }

        // Use openssl to check expiry
        let output = Command::new("openssl")
            .args(["x509", "-enddate", "-noout", "-in", &cert_file])
            .output()?;

        if !output.status.success() {
            error!("Failed to read certificate file: {cert_file}");
            return Ok(false);
        }

        let expiry_str = String::from_utf8_lossy(&output.stdout);
        let expiry = expiry_str.trim().strip_prefix("notAfter=").unwrap_or("");

        // Parse the date
        let expiry_date = chrono::DateTime::parse_from_str(expiry, "%b %d %H:%M:%S %Y %Z")
            .map_err(|e| format!("Failed to parse expiry date '{expiry}': {e}"))?;

        let now = Utc::now();
        let days_left = (expiry_date.with_timezone(&Utc) - now).num_days();

        debug!("Certificate expiry date: {expiry}");
        info!("Certificate days remaining: {days_left} days");

        if days_left <= self.config.days_before_renewal {
            warn!(
                "Certificate renewal required ({} days remaining)",
                days_left
            );
            Ok(false)
        } else {
            info!("Certificate status healthy ({days_left} days remaining)");
            Ok(true)
        }
    }

    /// Backup existing certificate.
    async fn backup_cert(&self) -> Result<(), Box<dyn std::error::Error>> {
        let backup_dir = format!("{}/backup", self.config.cert_dir);
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");

        fs::create_dir_all(&backup_dir).await?;

        let cert_file = format!("{}/{}.crt", self.config.cert_dir, self.config.service_name);
        let key_file = format!("{}/{}.key", self.config.cert_dir, self.config.service_name);

        if fs::metadata(&cert_file).await.is_ok() && fs::metadata(&key_file).await.is_ok() {
            fs::copy(
                &cert_file,
                format!(
                    "{}/{}.crt.{}",
                    backup_dir, self.config.service_name, timestamp
                ),
            )
            .await?;
            fs::copy(
                &key_file,
                format!(
                    "{}/{}.key.{}",
                    backup_dir, self.config.service_name, timestamp
                ),
            )
            .await?;

            info!("✅ Existing certificate backed up: {timestamp}");

            // Clean up old backups (older than 30 days)
            self.cleanup_old_backups(&backup_dir).await?;
        }

        Ok(())
    }

    /// Clean up old backup files.
    async fn cleanup_old_backups(
        &self,
        backup_dir: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cutoff = Utc::now() - chrono::Duration::days(30);

        let mut entries = fs::read_dir(backup_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if let (Ok(_metadata), Ok(modified)) = (
                entry.metadata().await,
                entry.metadata().await.and_then(|m| m.modified()),
            ) {
                let modified_dt: DateTime<Utc> = modified.into();
                if modified_dt < cutoff {
                    if let Err(e) = fs::remove_file(entry.path()).await {
                        warn!("Failed to remove old backup {:?}: {}", entry.path(), e);
                    } else {
                        debug!("Removed old backup: {:?}", entry.path());
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate new certificate.
    async fn generate_cert(&self) -> Result<bool, Box<dyn std::error::Error>> {
        info!("🔧 Generating new certificate...");

        let temp_cert = format!(
            "{}/{}-new.crt",
            self.config.cert_dir, self.config.service_name
        );
        let temp_key = format!(
            "{}/{}-new.key",
            self.config.cert_dir, self.config.service_name
        );
        let validity_hours = self.config.cert_validity_days * 24;

        // Remove existing temp files
        let _ = fs::remove_file(&temp_cert).await;
        let _ = fs::remove_file(&temp_key).await;

        // Try Step CLI first
        if self.try_step_cli(&temp_cert, &temp_key, validity_hours)? {
            return Ok(true);
        }

        // Fall back to OpenSSL
        warn!("Step CLI failed, using OpenSSL");
        self.try_openssl(&temp_cert, &temp_key)
    }

    /// Try generating certificate with Step CLI.
    fn try_step_cli(
        &self,
        cert_path: &str,
        key_path: &str,
        validity_hours: u32,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        debug!("Generating certificate with Step CLI");

        let mut cmd = Command::new("step");
        cmd.args([
            "certificate",
            "create",
            &format!("{}-server", self.config.service_name),
            cert_path,
            key_path,
            "--profile",
            "leaf",
            "--not-after",
            &format!("{validity_hours}h"),
            "--san",
            &self.config.server_ip,
            "--san",
            "localhost",
            "--san",
            "127.0.0.1",
        ]);

        let output = cmd.output()?;

        if output.status.success() {
            info!("✅ Certificate generated successfully with Step CLI");
            Ok(true)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Step CLI failed: {stderr}");
            Ok(false)
        }
    }

    /// Try generating certificate with OpenSSL.
    fn try_openssl(
        &self,
        cert_path: &str,
        key_path: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let subject = format!(
            "/C=KR/O={} Service/CN={}",
            self.config.service_name, self.config.server_ip
        );
        let san = format!("IP:{},DNS:localhost,IP:127.0.0.1", self.config.server_ip);

        let output = Command::new("openssl")
            .args([
                "req",
                "-x509",
                "-newkey",
                "rsa:2048",
                "-nodes",
                "-days",
                &self.config.cert_validity_days.to_string(),
                "-keyout",
                key_path,
                "-out",
                cert_path,
                "-subj",
                &subject,
                "-addext",
                &format!("subjectAltName={san}"),
            ])
            .output()?;

        if output.status.success() {
            info!("✅ Certificate generated successfully with OpenSSL");
            Ok(true)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("❌ OpenSSL certificate generation failed: {stderr}");
            Ok(false)
        }
    }

    /// Verify generated certificate.
    fn verify_cert(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let cert_file = format!(
            "{}/{}-new.crt",
            self.config.cert_dir, self.config.service_name
        );

        debug!("Verifying certificate...");

        // Basic format validation
        let output = Command::new("openssl")
            .args(["x509", "-noout", "-text", "-in", &cert_file])
            .output()?;

        if !output.status.success() {
            error!("❌ Certificate format is invalid");
            return Ok(false);
        }

        // Check SAN
        let text_output = String::from_utf8_lossy(&output.stdout);
        if text_output.contains(&self.config.server_ip) {
            info!("✅ Server IP found in certificate");
        } else {
            warn!("⚠️ Server IP not found in certificate SAN");
        }

        // Check expiry date
        let expiry_output = Command::new("openssl")
            .args(["x509", "-enddate", "-noout", "-in", &cert_file])
            .output()?;

        if expiry_output.status.success() {
            let expiry = String::from_utf8_lossy(&expiry_output.stdout);
            let expiry_date = expiry.trim().strip_prefix("notAfter=").unwrap_or("");
            info!("New certificate expiry date: {expiry_date}");
        }

        info!("✅ Certificate verification completed");
        Ok(true)
    }

    /// Replace certificate files.
    async fn replace_cert(&self) -> Result<bool, Box<dyn std::error::Error>> {
        info!("🔄 Replacing certificate...");

        let old_cert = format!("{}/{}.crt", self.config.cert_dir, self.config.service_name);
        let old_key = format!("{}/{}.key", self.config.cert_dir, self.config.service_name);
        let new_cert = format!(
            "{}/{}-new.crt",
            self.config.cert_dir, self.config.service_name
        );
        let new_key = format!(
            "{}/{}-new.key",
            self.config.cert_dir, self.config.service_name
        );

        if fs::metadata(&new_cert).await.is_ok() && fs::metadata(&new_key).await.is_ok() {
            // Backup old files
            let _ = fs::rename(&old_cert, format!("{old_cert}.old")).await;
            let _ = fs::rename(&old_key, format!("{old_key}.old")).await;

            // Move new files
            fs::rename(&new_cert, &old_cert).await?;
            fs::rename(&new_key, &old_key).await?;

            // Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&old_cert, std::fs::Permissions::from_mode(0o644)).await?;
                fs::set_permissions(&old_key, std::fs::Permissions::from_mode(0o600)).await?;
            }

            info!("✅ Certificate replacement completed");
            Ok(true)
        } else {
            error!("❌ New certificate files not found");
            Ok(false)
        }
    }

    /// Signal service restart.
    async fn signal_restart(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Execute reload command if provided
        if let Ok(reload_command) = env::var("RELOAD_COMMAND") {
            info!("🔃 Executing reload command: {reload_command}");

            let output = if reload_command.contains("&&") || reload_command.contains(';') {
                Command::new("sh").args(["-c", &reload_command]).output()?
            } else {
                let parts: Vec<&str> = reload_command.split_whitespace().collect();
                if let Some((cmd, args)) = parts.split_first() {
                    Command::new(cmd).args(args).output()?
                } else {
                    return Err("Empty reload command".into());
                }
            };

            if output.status.success() {
                info!("✅ Reload command executed successfully");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("⚠️ Reload command failed: {stderr}");
            }
        } else {
            // Create restart signal file
            let signal_file = format!("{}/.restart_needed", self.config.cert_dir);
            fs::write(&signal_file, "restart needed").await?;
            info!("🔃 Service restart signal created: {signal_file}");
            info!("ℹ️ Please configure your orchestrator to watch for this file");
        }

        Ok(())
    }

    /// Send notification.
    async fn send_notification(&self, message: &str, status: &str) {
        let emoji = match status {
            "success" => "✅",
            "warning" => "⚠️",
            "error" => "❌",
            _ => "ℹ️",
        };

        if let Some(ref webhook_url) = self.config.slack_webhook_url {
            let payload = json!({
                "text": format!("{} {} Certificate ({}): {}", emoji, self.config.service_name, self.config.server_ip, message)
            });

            match self
                .http_client
                .post(webhook_url)
                .json(&payload)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        debug!("Slack notification sent successfully");
                    } else {
                        warn!(
                            "Slack notification failed with status: {}",
                            response.status()
                        );
                    }
                }
                Err(e) => {
                    warn!("Failed to send Slack notification: {e}");
                }
            }
        }
    }

    /// Certificate renewal process.
    async fn renew_certificate(&self) -> Result<bool, Box<dyn std::error::Error>> {
        info!("🔄 Certificate renewal process started");

        // Backup
        self.backup_cert().await?;

        // Generate new certificate
        if !self.generate_cert().await? {
            self.send_notification("Certificate generation failed", "error")
                .await;
            return Ok(false);
        }

        // Verify
        if !self.verify_cert()? {
            self.send_notification("Certificate verification failed", "error")
                .await;
            // Clean up temp files
            let temp_cert = format!(
                "{}/{}-new.crt",
                self.config.cert_dir, self.config.service_name
            );
            let temp_key = format!(
                "{}/{}-new.key",
                self.config.cert_dir, self.config.service_name
            );
            let _ = fs::remove_file(&temp_cert).await;
            let _ = fs::remove_file(&temp_key).await;
            return Ok(false);
        }

        // Replace
        if !self.replace_cert().await? {
            self.send_notification("Certificate replacement failed", "error")
                .await;
            return Ok(false);
        }

        // Signal restart
        self.signal_restart().await?;

        // Success notification
        self.send_notification("Certificate renewal completed", "success")
            .await;
        info!("🎉 Certificate renewal process completed");

        Ok(true)
    }

    /// Check and renew certificate if needed.
    async fn check_and_renew(&self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("=== Periodic check started ===");

        let cert_valid = self.check_cert_expiry().await.unwrap_or(false);

        if !cert_valid {
            info!("Certificate renewal is required");
            if self.renew_certificate().await? {
                info!("Certificate renewal successful");
            } else {
                error!("Certificate renewal failed");
            }
        }

        debug!("=== Periodic check completed ===");
        Ok(())
    }

    /// Run the certificate manager.
    async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.initialize().await?;

        // Generate initial certificate if not exists
        let cert_file = format!("{}/{}.crt", self.config.cert_dir, self.config.service_name);
        if fs::metadata(&cert_file).await.is_err() {
            info!("Initial certificate not found. Generating...");
            self.renew_certificate().await?;
        }

        // Main loop
        loop {
            self.check_and_renew().await?;
            info!(
                "Waiting {} seconds until next check...",
                self.config.check_interval
            );
            sleep(Duration::from_secs(self.config.check_interval)).await;
        }
    }

    /// Run once and exit.
    async fn run_once(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.initialize().await?;
        self.check_and_renew().await?;
        info!("Single check completed");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt().with_env_filter(log_level).init();

    // Load configuration
    let config = Config::from_env()?;

    // Create certificate manager
    let cert_manager = CertManager::new(config);

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map_or("run", |s| s.as_str());

    match command {
        "run" => {
            info!("Starting in daemon mode");
            cert_manager.run().await
        }
        "once" | "--once" => {
            info!("Running once and exiting");
            cert_manager.run_once().await
        }
        "version" | "--version" => {
            println!("Simple Certificate Manager v{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        "help" | "--help" => {
            print_help();
            Ok(())
        }
        _ => {
            eprintln!("Unknown command: {command}");
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("Simple Certificate Manager v{}", env!("CARGO_PKG_VERSION"));
    println!("Automated certificate lifecycle management");
    println!();
    println!("USAGE:");
    println!("    {} [COMMAND]", env!("CARGO_PKG_NAME"));
    println!();
    println!("COMMANDS:");
    println!("    run         Run in daemon mode (default)");
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
