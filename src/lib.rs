//! Dimension Bridge - PKI Certificate Management Library
//!
//! Core library for automated certificate lifecycle management.

use chrono::{DateTime, Utc};
use serde_json::json;
use std::{env, process::Command};
use tokio::fs;
use tracing::{debug, error, info, warn};

/// Certificate manager configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Certificate directory.
    pub cert_dir: String,
    /// Log directory.
    pub log_dir: String,
    /// Check interval in seconds.
    pub check_interval: u64,
    /// Days before expiry to renew.
    pub days_before_renewal: i64,
    /// Certificate validity in days.
    pub cert_validity_days: u32,
    /// Server IP for SAN.
    pub server_ip: String,
    /// Service name for the certificate.
    pub service_name: String,
    /// Slack webhook URL for notifications.
    pub slack_webhook_url: Option<String>,
}

impl Config {
    /// Create a new Config with default values for testing.
    #[cfg(test)]
    pub fn test_default() -> Self {
        Self {
            cert_dir: "/tmp/test-certs".to_owned(),
            log_dir: "/tmp/test-logs".to_owned(),
            check_interval: 3600,
            days_before_renewal: 5,
            cert_validity_days: 15,
            server_ip: "127.0.0.1".to_owned(),
            service_name: "test-service".to_owned(),
            slack_webhook_url: None,
        }
    }

    /// Create a new Config from provided values for testing.
    #[must_use]
    pub fn test_from_values(
        server_ip: &str,
        service_name: &str,
        cert_dir: &str,
        check_interval: u64,
    ) -> Self {
        Self {
            cert_dir: cert_dir.to_owned(),
            log_dir: "/tmp/test-logs".to_owned(),
            check_interval,
            days_before_renewal: 5,
            cert_validity_days: 15,
            server_ip: server_ip.to_owned(),
            service_name: service_name.to_owned(),
            slack_webhook_url: None,
        }
    }

    /// Load configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns error if `SERVER_IP` or `CERT_DOMAINS` environment variables are missing.
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let server_ip = env::var("SERVER_IP")
            .or_else(|_| {
                env::var("CERT_DOMAINS")
                    .map(|domains| domains.split(',').next().unwrap_or("localhost").to_owned())
            })
            .map_err(|_| "SERVER_IP or CERT_DOMAINS environment variable is required")?;

        let service_name = env::var("SERVICE_NAME").unwrap_or_else(|_| "cert-agent".to_owned());

        Ok(Self {
            cert_dir: env::var("CERT_DIR").unwrap_or_else(|_| "/certs".to_owned()),
            log_dir: env::var("LOG_DIR").unwrap_or_else(|_| "/logs".to_owned()),
            check_interval: env::var("CHECK_INTERVAL")
                .unwrap_or_else(|_| "86400".to_owned())
                .parse::<u64>()
                .unwrap_or(86400),
            days_before_renewal: env::var("DAYS_BEFORE_RENEWAL")
                .unwrap_or_else(|_| "5".to_owned())
                .parse::<i64>()
                .unwrap_or(5),
            cert_validity_days: env::var("CERT_VALIDITY_DAYS")
                .unwrap_or_else(|_| "15".to_owned())
                .parse::<u32>()
                .unwrap_or(15),
            server_ip,
            service_name,
            slack_webhook_url: env::var("SLACK_WEBHOOK_URL").ok(),
        })
    }
}

/// Main certificate manager.
pub struct CertManager {
    /// Configuration for the certificate manager.
    pub config: Config,
    /// HTTP client for notifications.
    pub http_client: reqwest::Client,
}

impl CertManager {
    /// Create a new certificate manager.
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }

    /// Create a new certificate manager for testing.
    #[cfg(test)]
    pub fn test_new() -> Self {
        Self::new(Config::test_default())
    }

    /// Initialize the certificate manager.
    ///
    /// # Errors
    ///
    /// Returns error if directory creation fails.
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
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
    ///
    /// # Errors
    ///
    /// Returns error if certificate reading or parsing fails.
    pub async fn check_cert_expiry(&self) -> Result<bool, Box<dyn std::error::Error>> {
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
    ///
    /// # Errors
    ///
    /// Returns error if backup operations fail.
    pub async fn backup_cert(&self) -> Result<(), Box<dyn std::error::Error>> {
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

    /// Send Slack notification.
    ///
    /// # Errors
    ///
    /// Returns error if HTTP request fails.
    pub async fn send_slack_notification(
        &self,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(webhook_url) = &self.config.slack_webhook_url else {
            debug!("No Slack webhook configured, skipping notification");
            return Ok(());
        };

        let payload = json!({
            "text": format!("[{}] {}", self.config.service_name, message),
            "username": "cert-manager",
            "icon_emoji": ":lock:"
        });

        let response = self
            .http_client
            .post(webhook_url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            info!("✅ Slack notification sent successfully");
        } else {
            warn!("⚠️ Slack notification failed: {}", response.status());
        }

        Ok(())
    }

    /// Run the certificate manager daemon.
    ///
    /// # Errors
    ///
    /// Returns error if certificate operations fail.
    #[allow(clippy::future_not_send)]
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        use std::time::Duration;
        use tokio::time::sleep;

        loop {
            let needs_renewal = !self.check_cert_expiry().await?;

            if needs_renewal {
                info!("🔄 Starting certificate renewal process");

                // Backup existing certificate
                if let Err(e) = self.backup_cert().await {
                    error!("Failed to backup certificate: {e}");
                    self.send_slack_notification(&format!("❌ Backup failed: {e}"))
                        .await?;
                }

                // Generate new certificate
                match self.generate_cert().await {
                    Ok(true) => {
                        info!("✅ Certificate renewal completed successfully");
                        self.send_slack_notification("✅ Certificate renewed successfully")
                            .await?;
                    }
                    Ok(false) => {
                        error!("❌ Certificate generation failed");
                        self.send_slack_notification("❌ Certificate generation failed")
                            .await?;
                    }
                    Err(e) => {
                        error!("❌ Certificate renewal error: {e}");
                        self.send_slack_notification(&format!("❌ Renewal error: {e}"))
                            .await?;
                    }
                }
            } else {
                debug!("Certificate is still valid, skipping renewal");
            }

            info!(
                "⏰ Sleeping for {} seconds until next check",
                self.config.check_interval
            );
            sleep(Duration::from_secs(self.config.check_interval)).await;
        }
    }

    /// Run once and exit.
    ///
    /// # Errors
    ///
    /// Returns error if certificate operations fail.
    #[allow(clippy::future_not_send)]
    pub async fn run_once(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔄 Running certificate check once");

        let needs_renewal = !self.check_cert_expiry().await?;

        if needs_renewal {
            info!("🔄 Certificate renewal required");

            // Backup existing certificate
            self.backup_cert().await?;

            // Generate new certificate
            match self.generate_cert().await {
                Ok(true) => {
                    info!("✅ Certificate renewal completed successfully");
                    self.send_slack_notification("✅ Certificate renewed successfully")
                        .await?;
                }
                Ok(false) => {
                    error!("❌ Certificate generation failed");
                    return Err("Certificate generation failed".into());
                }
                Err(e) => {
                    error!("❌ Certificate renewal error: {e}");
                    return Err(e);
                }
            }
        } else {
            info!("✅ Certificate is valid, no renewal needed");
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
            return self.deploy_cert(&temp_cert, &temp_key).await;
        }

        // Fall back to OpenSSL
        warn!("Step CLI failed, using OpenSSL");
        self.try_openssl(&temp_cert, &temp_key).await
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

    /// Check if a string is a valid IP address.
    fn is_ip_address(addr: &str) -> bool {
        addr.parse::<std::net::IpAddr>().is_ok()
    }

    /// Try generating certificate with OpenSSL.
    async fn try_openssl(
        &self,
        cert_path: &str,
        key_path: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let subject = format!(
            "/C=KR/O={} Service/CN={}",
            self.config.service_name, self.config.server_ip
        );

        // Build SAN (Subject Alternative Name) list properly
        let mut san_entries = vec!["DNS:localhost".to_owned(), "IP:127.0.0.1".to_owned()];

        if Self::is_ip_address(&self.config.server_ip) {
            san_entries.push(format!("IP:{}", self.config.server_ip));
        } else {
            san_entries.push(format!("DNS:{}", self.config.server_ip));
        }

        let san = san_entries.join(",");

        let mut child = Command::new("openssl")
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
                "-extensions",
                "v3_req",
                "-config",
                "/dev/stdin",
            ])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        // Write OpenSSL config to stdin
        let config_content = format!(
            "[req]\n\
             distinguished_name = req_distinguished_name\n\
             req_extensions = v3_req\n\
             prompt = no\n\
             \n\
             [req_distinguished_name]\n\
             \n\
             [v3_req]\n\
             basicConstraints = CA:FALSE\n\
             keyUsage = nonRepudiation, digitalSignature, keyEncipherment\n\
             subjectAltName = {san}\n"
        );

        if let Some(stdin) = child.stdin.take() {
            use std::io::Write;
            let mut stdin = std::io::BufWriter::new(stdin);
            stdin.write_all(config_content.as_bytes())?;
        }

        let result = child.wait_with_output()?;

        if result.status.success() {
            info!("✅ Certificate generated successfully with OpenSSL");
            self.deploy_cert(cert_path, key_path).await
        } else {
            let stderr = String::from_utf8_lossy(&result.stderr);
            error!("OpenSSL certificate generation failed: {stderr}");
            Ok(false)
        }
    }

    /// Deploy the new certificate.
    async fn deploy_cert(
        &self,
        temp_cert_path: &str,
        temp_key_path: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let final_cert = format!("{}/{}.crt", self.config.cert_dir, self.config.service_name);
        let final_key = format!("{}/{}.key", self.config.cert_dir, self.config.service_name);

        // Atomic move
        fs::rename(temp_cert_path, &final_cert).await?;
        fs::rename(temp_key_path, &final_key).await?;

        // Set proper permissions
        self.set_cert_permissions(&final_cert, &final_key).await?;

        info!("✅ Certificate deployed to {final_cert}");
        self.execute_reload_command()?;

        Ok(true)
    }

    /// Set certificate file permissions.
    async fn set_cert_permissions(
        &self,
        cert_path: &str,
        key_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            // Certificate: readable by all (644)
            let cert_perms = std::fs::Permissions::from_mode(0o644);
            fs::set_permissions(cert_path, cert_perms).await?;

            // Private key: readable by owner only (600)
            let key_perms = std::fs::Permissions::from_mode(0o600);
            fs::set_permissions(key_path, key_perms).await?;

            debug!("Set certificate permissions: cert=644, key=600");
        }

        Ok(())
    }

    /// Execute the reload command.
    #[allow(clippy::unused_self)]
    fn execute_reload_command(&self) -> Result<(), Box<dyn std::error::Error>> {
        let Some(reload_command) = env::var("RELOAD_COMMAND").ok() else {
            debug!("No reload command configured");
            return Ok(());
        };

        info!("🔄 Executing reload command: {reload_command}");

        let output = Command::new("sh").args(["-c", &reload_command]).output()?;

        if output.status.success() {
            info!("✅ Reload command executed successfully");
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("⚠️ Reload command failed: {stderr}");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::test_default();
        assert_eq!(config.cert_dir, "/tmp/test-certs");
        assert_eq!(config.service_name, "test-service");
        assert_eq!(config.days_before_renewal, 5);
        assert_eq!(config.cert_validity_days, 15);
        assert_eq!(config.check_interval, 3600);
    }

    #[test]
    fn test_config_from_values() {
        let config =
            Config::test_from_values("192.168.1.100", "my-test-service", "/custom/cert/dir", 7200);

        assert_eq!(config.server_ip, "192.168.1.100");
        assert_eq!(config.service_name, "my-test-service");
        assert_eq!(config.cert_dir, "/custom/cert/dir");
        assert_eq!(config.check_interval, 7200);
        assert_eq!(config.days_before_renewal, 5); // default
    }

    #[test]
    fn test_config_equality() {
        let config1 = Config::test_default();
        let config2 = Config::test_default();
        assert_eq!(config1, config2);
    }

    #[tokio::test]
    async fn test_cert_manager_new() {
        let config = Config::test_default();
        let manager = CertManager::new(config.clone());

        assert_eq!(manager.config, config);
    }

    #[tokio::test]
    async fn test_cert_manager_initialize() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let cert_dir = temp_dir.path().join("certs");
        let log_dir = temp_dir.path().join("logs");

        let mut config =
            Config::test_from_values("127.0.0.1", "test", cert_dir.to_str().unwrap(), 3600);
        config.log_dir = log_dir.to_str().unwrap().to_owned();

        let manager = CertManager::new(config);

        // This should create the directories
        let result = manager.initialize().await;
        assert!(result.is_ok());

        // Verify directories were created
        assert!(cert_dir.exists());
        assert!(log_dir.exists());
    }

    #[test]
    fn test_is_ip_address_valid_ipv4() {
        assert!(CertManager::is_ip_address("192.168.1.1"));
        assert!(CertManager::is_ip_address("127.0.0.1"));
        assert!(CertManager::is_ip_address("10.0.0.1"));
        assert!(CertManager::is_ip_address("172.16.0.1"));
        assert!(CertManager::is_ip_address("8.8.8.8"));
        assert!(CertManager::is_ip_address("255.255.255.255"));
        assert!(CertManager::is_ip_address("0.0.0.0"));
    }

    #[test]
    fn test_is_ip_address_valid_ipv6() {
        assert!(CertManager::is_ip_address("::1"));
        assert!(CertManager::is_ip_address("2001:db8::1"));
        assert!(CertManager::is_ip_address("fe80::1"));
        assert!(CertManager::is_ip_address("::"));
        assert!(CertManager::is_ip_address(
            "2001:0db8:85a3:0000:0000:8a2e:0370:7334"
        ));
    }

    #[test]
    fn test_is_ip_address_invalid_domain_names() {
        assert!(!CertManager::is_ip_address("example.com"));
        assert!(!CertManager::is_ip_address("subdomain.example.com"));
        assert!(!CertManager::is_ip_address("api.company.internal"));
        assert!(!CertManager::is_ip_address("localhost"));
        assert!(!CertManager::is_ip_address("my-service"));
        assert!(!CertManager::is_ip_address("test1.example.com"));
        assert!(!CertManager::is_ip_address("web-server.local"));
    }

    #[test]
    fn test_is_ip_address_invalid_formats() {
        assert!(!CertManager::is_ip_address(""));
        assert!(!CertManager::is_ip_address("256.256.256.256")); // Invalid IPv4
        assert!(!CertManager::is_ip_address("192.168.1")); // Incomplete IPv4
        assert!(!CertManager::is_ip_address("192.168.1.1.1")); // Too many octets
        assert!(!CertManager::is_ip_address("not-an-ip"));
        assert!(!CertManager::is_ip_address("192.168.abc.1"));
        assert!(!CertManager::is_ip_address("::1::2")); // Invalid IPv6
    }

    #[test]
    fn test_san_generation_with_ip_address() {
        let config = Config::test_from_values("192.168.1.100", "test-service", "/tmp", 3600);
        let manager = CertManager::new(config);

        // We need to test the SAN generation logic indirectly by checking the components
        let mut san_entries = vec!["DNS:localhost".to_owned(), "IP:127.0.0.1".to_owned()];

        if CertManager::is_ip_address(&manager.config.server_ip) {
            san_entries.push(format!("IP:{}", manager.config.server_ip));
        } else {
            san_entries.push(format!("DNS:{}", manager.config.server_ip));
        }

        let san = san_entries.join(",");
        assert_eq!(san, "DNS:localhost,IP:127.0.0.1,IP:192.168.1.100");
    }

    #[test]
    fn test_san_generation_with_domain_name() {
        let config = Config::test_from_values("api.example.com", "test-service", "/tmp", 3600);
        let manager = CertManager::new(config);

        // Test SAN generation logic for domain names
        let mut san_entries = vec!["DNS:localhost".to_owned(), "IP:127.0.0.1".to_owned()];

        if CertManager::is_ip_address(&manager.config.server_ip) {
            san_entries.push(format!("IP:{}", manager.config.server_ip));
        } else {
            san_entries.push(format!("DNS:{}", manager.config.server_ip));
        }

        let san = san_entries.join(",");
        assert_eq!(san, "DNS:localhost,IP:127.0.0.1,DNS:api.example.com");
    }

    #[test]
    fn test_san_generation_with_subdomain() {
        let config = Config::test_from_values("web.company.internal", "test-service", "/tmp", 3600);
        let manager = CertManager::new(config);

        // Test SAN generation logic for subdomains
        let mut san_entries = vec!["DNS:localhost".to_owned(), "IP:127.0.0.1".to_owned()];

        if CertManager::is_ip_address(&manager.config.server_ip) {
            san_entries.push(format!("IP:{}", manager.config.server_ip));
        } else {
            san_entries.push(format!("DNS:{}", manager.config.server_ip));
        }

        let san = san_entries.join(",");
        assert_eq!(san, "DNS:localhost,IP:127.0.0.1,DNS:web.company.internal");
    }
}
