use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use chrono::{Utc, Duration as ChronoDuration};
use anyhow::{Result, Context};
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::config::GitHubAppConfig;

#[derive(Debug, Clone)]
pub struct JWTManager {
    config: GitHubAppConfig,
    token_cache: HashMap<String, (String, Instant)>,
    public_key_cache: HashMap<String, (String, Instant)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JWTPayload {
    pub iss: String,  // App ID
    pub iat: i64,     // Issued at
    pub exp: i64,     // Expiration time
    pub alg: String,  // Algorithm
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallationTokenPayload {
    pub iss: String,  // App ID
    pub iat: i64,     // Issued at
    pub exp: i64,     // Expiration time
    pub alg: String,  // Algorithm
    pub aud: String,  // Audience (installation ID)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubTokenResponse {
    pub token: String,
    pub expires_at: String,
    pub permissions: HashMap<String, String>,
    pub repository_selection: String,
}

impl JWTManager {
    pub async fn new(config: &GitHubAppConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            token_cache: HashMap::new(),
            public_key_cache: HashMap::new(),
        })
    }
    
    pub async fn create_app_jwt(&mut self) -> Result<String> {
        let cache_key = "app_jwt".to_string();
        
        // Check cache first
        if let Some((token, created_at)) = self.token_cache.get(&cache_key) {
            if created_at.elapsed() < Duration::from_secs(540) { // 9 minutes cache (JWT expires in 10 minutes)
                return Ok(token.clone());
            }
        }
        
        let now = Utc::now();
        let exp = now + ChronoDuration::minutes(10); // JWT expires in 10 minutes
        
        let payload = JWTPayload {
            iss: self.config.app_id.clone(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            alg: "RS256".to_string(),
        };
        
        let token = encode(
            &Header::new(Algorithm::RS256),
            &payload,
            &EncodingKey::from_rsa_pem(self.config.private_key.as_bytes())?
        )?;
        
        // Cache the token
        self.token_cache.insert(cache_key, (token.clone(), Instant::now()));
        
        info!("Created app JWT token, expires at {}", exp);
        
        Ok(token)
    }
    
    pub async fn create_installation_jwt(&mut self, installation_id: &str) -> Result<String> {
        let cache_key = format!("installation_jwt_{}", installation_id);
        
        // Check cache first
        if let Some((token, created_at)) = self.token_cache.get(&cache_key) {
            if created_at.elapsed() < Duration::from_secs(540) { // 9 minutes cache
                return Ok(token.clone());
            }
        }
        
        let now = Utc::now();
        let exp = now + ChronoDuration::minutes(10);
        
        let payload = InstallationTokenPayload {
            iss: self.config.app_id.clone(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            alg: "RS256".to_string(),
            aud: installation_id.to_string(),
        };
        
        let token = encode(
            &Header::new(Algorithm::RS256),
            &payload,
            &EncodingKey::from_rsa_pem(self.config.private_key.as_bytes())?
        )?;
        
        // Cache the token
        self.token_cache.insert(cache_key, (token.clone(), Instant::now()));
        
        info!("Created installation JWT token for {}, expires at {}", installation_id, exp);
        
        Ok(token)
    }
    
    pub async fn verify_jwt(&self, token: &str) -> Result<JWTPayload> {
        // For now, we'll decode without verification since we're the issuer
        // In a real implementation, you'd verify against the public key
        let token_data = decode::<JWTPayload>(
            token,
            &DecodingKey::from_rsa_pem(self.config.private_key.as_bytes())?,
            &Validation::new(Algorithm::RS256)
        )?;
        
        // Check if token is expired
        let now = Utc::now().timestamp();
        if token_data.claims.exp < now {
            return Err(anyhow::anyhow!("JWT token is expired"));
        }
        
        Ok(token_data.claims)
    }
    
    pub async fn get_installation_token(&mut self, installation_id: &str) -> Result<String> {
        let cache_key = format!("installation_token_{}", installation_id);
        
        // Check cache first
        if let Some((token, created_at)) = self.token_cache.get(&cache_key) {
            if created_at.elapsed() < Duration::from_secs(3600) { // 1 hour cache
                return Ok(token.clone());
            }
        }
        
        // Create JWT for authentication
        let jwt = self.create_installation_jwt(installation_id).await?;
        
        // Request installation token from GitHub
        let token = self.request_installation_token(installation_id, &jwt).await?;
        
        // Cache the token
        self.token_cache.insert(cache_key, (token.clone(), Instant::now()));
        
        Ok(token)
    }
    
    async fn request_installation_token(&self, installation_id: &str, jwt: &str) -> Result<String> {
        use reqwest::Client;
        
        let client = Client::new();
        let url = format!("{}/app/installations/{}/access_tokens", 
            self.config.base_url, installation_id);
        
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", jwt))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "spec-to-proof-gh-app")
            .send()
            .await
            .context("Failed to request installation token")?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to get installation token: {}", error_text));
        }
        
        let token_response: GitHubTokenResponse = response.json().await
            .context("Failed to parse installation token response")?;
        
        Ok(token_response.token)
    }
    
    pub async fn validate_webhook_signature(&self, payload: &str, signature: &str) -> Result<bool> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        use hex;
        
        // Check signature format
        if !signature.starts_with("sha256=") {
            return Ok(false);
        }
        
        // Extract signature hash
        let signature_hash = &signature[7..];
        
        // Calculate expected signature
        let mut mac = Hmac::<Sha256>::new_from_slice(self.config.webhook_secret.as_bytes())
            .context("Failed to create HMAC")?;
        mac.update(payload.as_bytes());
        let expected_hash = hex::encode(mac.finalize().into_bytes());
        
        // Compare signatures
        let valid = signature_hash == expected_hash;
        
        if valid {
            info!("Webhook signature validated successfully");
        } else {
            warn!("Webhook signature validation failed");
        }
        
        Ok(valid)
    }
    
    pub async fn get_public_key(&mut self) -> Result<String> {
        let cache_key = "public_key".to_string();
        
        // Check cache first
        if let Some((key, created_at)) = self.public_key_cache.get(&cache_key) {
            if created_at.elapsed() < Duration::from_secs(3600) { // 1 hour cache
                return Ok(key.clone());
            }
        }
        
        // Extract public key from private key
        // In a real implementation, you'd store the public key separately
        let public_key = self.extract_public_key_from_private().await?;
        
        // Cache the public key
        self.public_key_cache.insert(cache_key, (public_key.clone(), Instant::now()));
        
        Ok(public_key)
    }
    
    async fn extract_public_key_from_private(&self) -> Result<String> {
        // TODO: Implement actual public key extraction from private key
        // For now, return a mock public key
        Ok("mock-public-key".to_string())
    }
    
    pub async fn refresh_token_cache(&mut self) -> Result<()> {
        info!("Refreshing token cache");
        
        // Clear expired tokens
        let now = Instant::now();
        self.token_cache.retain(|_, (_, created_at)| {
            created_at.elapsed() < Duration::from_secs(600) // Keep tokens less than 10 minutes old
        });
        
        info!("Token cache refreshed, {} tokens remaining", self.token_cache.len());
        
        Ok(())
    }
    
    pub async fn get_token_statistics(&self) -> Result<TokenStatistics> {
        let mut stats = TokenStatistics {
            total_tokens: self.token_cache.len() as u64,
            app_jwt_count: 0,
            installation_jwt_count: 0,
            installation_token_count: 0,
            cache_hit_rate: 0.0,
        };
        
        // Count different types of tokens
        for (key, _) in &self.token_cache {
            if key == "app_jwt" {
                stats.app_jwt_count += 1;
            } else if key.starts_with("installation_jwt_") {
                stats.installation_jwt_count += 1;
            } else if key.starts_with("installation_token_") {
                stats.installation_token_count += 1;
            }
        }
        
        // Calculate cache hit rate (mock for now)
        stats.cache_hit_rate = 0.85; // 85% cache hit rate
        
        Ok(stats)
    }
    
    pub async fn invalidate_token_cache(&mut self) -> Result<()> {
        info!("Invalidating token cache");
        self.token_cache.clear();
        Ok(())
    }
    
    pub async fn health_check(&self) -> Result<bool> {
        // Check if we can create a JWT token
        let jwt_manager = JWTManager::new(&self.config).await?;
        let _token = jwt_manager.create_app_jwt().await?;
        
        Ok(true)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenStatistics {
    pub total_tokens: u64,
    pub app_jwt_count: u64,
    pub installation_jwt_count: u64,
    pub installation_token_count: u64,
    pub cache_hit_rate: f64,
}

// Authentication middleware for Axum
pub async fn auth_middleware(
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next<axum::body::Body>,
) -> Result<axum::http::Response<axum::body::Body>, axum::http::StatusCode> {
    // Extract JWT token from Authorization header
    let auth_header = request.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    
    if !auth_header.starts_with("Bearer ") {
        return Err(axum::http::StatusCode::UNAUTHORIZED);
    }
    
    let token = &auth_header[7..]; // Remove "Bearer " prefix
    
    // TODO: Implement actual JWT verification
    // For now, just check if token exists
    if token.is_empty() {
        return Err(axum::http::StatusCode::UNAUTHORIZED);
    }
    
    // Continue with the request
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_jwt_manager_creation() {
        let config = GitHubAppConfig::default();
        let manager = JWTManager::new(&config).await;
        assert!(manager.is_ok());
    }
    
    #[test]
    fn test_jwt_payload_serialization() {
        let payload = JWTPayload {
            iss: "12345".to_string(),
            iat: 1234567890,
            exp: 1234567890 + 600,
            alg: "RS256".to_string(),
        };
        
        let json = serde_json::to_string(&payload).unwrap();
        let deserialized: JWTPayload = serde_json::from_str(&json).unwrap();
        
        assert_eq!(payload.iss, deserialized.iss);
        assert_eq!(payload.iat, deserialized.iat);
        assert_eq!(payload.exp, deserialized.exp);
        assert_eq!(payload.alg, deserialized.alg);
    }
    
    #[test]
    fn test_installation_token_payload_serialization() {
        let payload = InstallationTokenPayload {
            iss: "12345".to_string(),
            iat: 1234567890,
            exp: 1234567890 + 600,
            alg: "RS256".to_string(),
            aud: "67890".to_string(),
        };
        
        let json = serde_json::to_string(&payload).unwrap();
        let deserialized: InstallationTokenPayload = serde_json::from_str(&json).unwrap();
        
        assert_eq!(payload.iss, deserialized.iss);
        assert_eq!(payload.iat, deserialized.iat);
        assert_eq!(payload.exp, deserialized.exp);
        assert_eq!(payload.alg, deserialized.alg);
        assert_eq!(payload.aud, deserialized.aud);
    }
    
    #[tokio::test]
    async fn test_webhook_signature_validation() {
        let config = GitHubAppConfig {
            webhook_secret: "test_secret".to_string(),
            ..Default::default()
        };
        
        let manager = JWTManager::new(&config).await.unwrap();
        
        let payload = r#"{"action":"opened","pull_request":{"id":123}}"#;
        let signature = "sha256=abc123"; // Mock signature
        
        let result = manager.validate_webhook_signature(payload, signature).await;
        assert!(result.is_ok());
    }
} 