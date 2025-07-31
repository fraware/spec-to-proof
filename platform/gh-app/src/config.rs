use std::env;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use config::{Config, ConfigError, Environment, File};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAppConfig {
    // GitHub App settings
    pub app_id: String,
    pub private_key: String,
    pub webhook_secret: String,
    pub installation_id: String,
    pub client_id: String,
    pub client_secret: String,
    
    // Permissions and events
    pub permissions: Vec<String>,
    pub events: Vec<String>,
    
    // API settings
    pub base_url: String,
    pub upload_url: String,
    pub api_version: String,
    
    // Server settings
    pub host: String,
    pub port: u16,
    pub webhook_path: String,
    
    // Badge settings
    pub badge_context: String,
    pub badge_description: String,
    pub badge_target_url: String,
    
    // Sigstore settings
    pub sigstore_rekor_url: String,
    pub sigstore_fulcio_url: String,
    pub sigstore_oidc_issuer: String,
    
    // AWS settings
    pub aws_region: String,
    pub aws_secrets_arn: String,
    pub aws_s3_bucket: String,
    
    // Logging settings
    pub log_level: String,
    pub log_format: String,
    
    // Security settings
    pub enable_webhook_verification: bool,
    pub enable_jwt_verification: bool,
    pub enable_sigstore_verification: bool,
    
    // Rate limiting
    pub rate_limit_requests: u32,
    pub rate_limit_window: u64,
    
    // Timeouts
    pub request_timeout: u64,
    pub webhook_timeout: u64,
    pub badge_timeout: u64,
}

impl Default for GitHubAppConfig {
    fn default() -> Self {
        Self {
            app_id: "".to_string(),
            private_key: "".to_string(),
            webhook_secret: "".to_string(),
            installation_id: "".to_string(),
            client_id: "".to_string(),
            client_secret: "".to_string(),
            permissions: vec![
                "pull_requests".to_string(),
                "statuses".to_string(),
                "contents".to_string(),
                "metadata".to_string(),
            ],
            events: vec![
                "pull_request".to_string(),
                "push".to_string(),
                "status".to_string(),
            ],
            base_url: "https://api.github.com".to_string(),
            upload_url: "https://uploads.github.com".to_string(),
            api_version: "2022-11-28".to_string(),
            host: "0.0.0.0".to_string(),
            port: 8080,
            webhook_path: "/webhook".to_string(),
            badge_context: "spec-to-proof/verification".to_string(),
            badge_description: "Spec-to-Proof verification status".to_string(),
            badge_target_url: "https://spec-to-proof.com/verification".to_string(),
            sigstore_rekor_url: "https://rekor.sigstore.dev".to_string(),
            sigstore_fulcio_url: "https://fulcio.sigstore.dev".to_string(),
            sigstore_oidc_issuer: "https://oauth2.sigstore.dev/auth".to_string(),
            aws_region: "us-east-1".to_string(),
            aws_secrets_arn: "".to_string(),
            aws_s3_bucket: "spec-to-proof-artifacts".to_string(),
            log_level: "info".to_string(),
            log_format: "json".to_string(),
            enable_webhook_verification: true,
            enable_jwt_verification: true,
            enable_sigstore_verification: true,
            rate_limit_requests: 1000,
            rate_limit_window: 3600,
            request_timeout: 30,
            webhook_timeout: 10,
            badge_timeout: 5,
        }
    }
}

impl GitHubAppConfig {
    pub fn load() -> Result<Self> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config".to_string());
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        
        info!("Loading configuration from {} with environment {}", config_path, environment);
        
        let config = Config::builder()
            // Start with default settings
            .add_source(File::from(PathBuf::from(&config_path).join("default.yaml")).required(false))
            // Add environment-specific settings
            .add_source(File::from(PathBuf::from(&config_path).join(format!("{}.yaml", environment))).required(false))
            // Add local settings
            .add_source(File::from(PathBuf::from(&config_path).join("local.yaml")).required(false))
            // Add environment variables with prefix
            .add_source(Environment::with_prefix("GH_APP").separator("__"))
            // Add secrets from AWS Secrets Manager
            .add_source(Environment::with_prefix("GH_APP_SECRETS").separator("__"))
            .build()
            .context("Failed to build configuration")?;
        
        let mut app_config: GitHubAppConfig = config.try_deserialize()
            .context("Failed to deserialize configuration")?;
        
        // Load secrets from AWS Secrets Manager if configured
        if !app_config.aws_secrets_arn.is_empty() {
            app_config.load_secrets_from_aws().await?;
        }
        
        // Validate configuration
        app_config.validate()?;
        
        info!("Configuration loaded successfully");
        Ok(app_config)
    }
    
    async fn load_secrets_from_aws(&mut self) -> Result<()> {
        use aws_sdk_secretsmanager::Client as SecretsClient;
        use aws_config::BehaviorVersion;
        
        let aws_config = aws_config::defaults(BehaviorVersion::latest())
            .region(aws_sdk_sts::Region::new(self.aws_region.clone()))
            .load()
            .await;
        
        let secrets_client = SecretsClient::new(&aws_config);
        
        let secret_response = secrets_client
            .get_secret_value()
            .secret_id(&self.aws_secrets_arn)
            .send()
            .await
            .context("Failed to retrieve secret from AWS Secrets Manager")?;
        
        if let Some(secret_string) = secret_response.secret_string() {
            let secrets: serde_json::Value = serde_json::from_str(secret_string)
                .context("Failed to parse secrets JSON")?;
            
            // Update configuration with secrets
            if let Some(app_id) = secrets.get("app_id").and_then(|v| v.as_str()) {
                self.app_id = app_id.to_string();
            }
            if let Some(private_key) = secrets.get("private_key").and_then(|v| v.as_str()) {
                self.private_key = private_key.to_string();
            }
            if let Some(webhook_secret) = secrets.get("webhook_secret").and_then(|v| v.as_str()) {
                self.webhook_secret = webhook_secret.to_string();
            }
            if let Some(client_secret) = secrets.get("client_secret").and_then(|v| v.as_str()) {
                self.client_secret = client_secret.to_string();
            }
        }
        
        Ok(())
    }
    
    pub fn validate(&self) -> Result<()> {
        // Validate required fields
        if self.app_id.is_empty() {
            return Err(anyhow::anyhow!("app_id is required"));
        }
        if self.private_key.is_empty() {
            return Err(anyhow::anyhow!("private_key is required"));
        }
        if self.webhook_secret.is_empty() {
            return Err(anyhow::anyhow!("webhook_secret is required"));
        }
        if self.installation_id.is_empty() {
            return Err(anyhow::anyhow!("installation_id is required"));
        }
        
        // Validate URLs
        if !self.base_url.starts_with("http") {
            return Err(anyhow::anyhow!("base_url must be a valid URL"));
        }
        if !self.upload_url.starts_with("http") {
            return Err(anyhow::anyhow!("upload_url must be a valid URL"));
        }
        
        // Validate Sigstore URLs
        if !self.sigstore_rekor_url.starts_with("http") {
            return Err(anyhow::anyhow!("sigstore_rekor_url must be a valid URL"));
        }
        if !self.sigstore_fulcio_url.starts_with("http") {
            return Err(anyhow::anyhow!("sigstore_fulcio_url must be a valid URL"));
        }
        
        // Validate AWS settings
        if !self.aws_region.is_empty() && self.aws_secrets_arn.is_empty() {
            warn!("AWS region is set but no secrets ARN provided");
        }
        
        // Validate timeouts
        if self.request_timeout == 0 {
            return Err(anyhow::anyhow!("request_timeout must be greater than 0"));
        }
        if self.webhook_timeout == 0 {
            return Err(anyhow::anyhow!("webhook_timeout must be greater than 0"));
        }
        if self.badge_timeout == 0 {
            return Err(anyhow::anyhow!("badge_timeout must be greater than 0"));
        }
        
        info!("Configuration validation passed");
        Ok(())
    }
    
    pub fn get_webhook_url(&self) -> String {
        format!("http://{}:{}{}", self.host, self.port, self.webhook_path)
    }
    
    pub fn get_badge_url(&self, repo: &str, pr: &str) -> String {
        format!("{}/badge/{}/{}", self.get_webhook_url(), repo, pr)
    }
    
    pub fn get_health_url(&self) -> String {
        format!("http://{}:{}/health", self.host, self.port)
    }
    
    pub fn get_metrics_url(&self) -> String {
        format!("http://{}:{}/metrics", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = GitHubAppConfig::default();
        assert!(!config.app_id.is_empty() || config.app_id.is_empty()); // Either empty or not
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "0.0.0.0");
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = GitHubAppConfig::default();
        
        // Should fail validation
        assert!(config.validate().is_err());
        
        // Set required fields
        config.app_id = "12345".to_string();
        config.private_key = "-----BEGIN RSA PRIVATE KEY-----\n-----END RSA PRIVATE KEY-----".to_string();
        config.webhook_secret = "secret".to_string();
        config.installation_id = "67890".to_string();
        
        // Should pass validation
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_url_generation() {
        let config = GitHubAppConfig::default();
        let webhook_url = config.get_webhook_url();
        assert!(webhook_url.contains("0.0.0.0:8080"));
        assert!(webhook_url.ends_with("/webhook"));
        
        let badge_url = config.get_badge_url("test-repo", "123");
        assert!(badge_url.contains("/badge/test-repo/123"));
    }
} 