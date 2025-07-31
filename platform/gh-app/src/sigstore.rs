use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use reqwest::{Client, header::{HeaderMap, HeaderValue, CONTENT_TYPE, ACCEPT}};
use anyhow::{Result, Context};
use tracing::{info, warn, error};
use chrono::{DateTime, Utc};
use base64::{Engine as _, engine::general_purpose};
use sha2::{Sha256, Digest};
use hex;

use crate::config::GitHubAppConfig;
use crate::proto::gh_app::v1::*;

#[derive(Debug, Clone)]
pub struct SigstoreClient {
    config: GitHubAppConfig,
    http_client: Client,
    rekor_url: String,
    fulcio_url: String,
    oidc_issuer: String,
    entry_cache: HashMap<String, (SigstoreEntry, Instant)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RekorEntry {
    pub uuid: String,
    pub log_index: u64,
    pub integrated_time: u64,
    pub log_id: String,
    pub body: String,
    pub verification: RekorVerification,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RekorVerification {
    pub inclusion_proof: Option<RekorInclusionProof>,
    pub signed_entry_timestamp: String,
    pub signed_tree_head: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RekorInclusionProof {
    pub hashes: Vec<String>,
    pub log_index: u64,
    pub root_hash: String,
    pub tree_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FulcioCertificate {
    pub cert: String,
    pub chain: Vec<String>,
    pub sct: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OIDCToken {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: u64,
    pub iat: u64,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
}

impl SigstoreClient {
    pub async fn new(config: &GitHubAppConfig) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        
        let http_client = Client::builder()
            .timeout(Duration::from_secs(config.request_timeout))
            .default_headers(headers)
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(Self {
            config: config.clone(),
            http_client,
            rekor_url: config.sigstore_rekor_url.clone(),
            fulcio_url: config.sigstore_fulcio_url.clone(),
            oidc_issuer: config.sigstore_oidc_issuer.clone(),
            entry_cache: HashMap::new(),
        })
    }
    
    pub async fn get_entry(&mut self, entry_id: &str) -> Result<SigstoreEntry> {
        // Check cache first
        if let Some((entry, created_at)) = self.entry_cache.get(entry_id) {
            if created_at.elapsed() < Duration::from_secs(3600) { // 1 hour cache
                return Ok(entry.clone());
            }
        }
        
        // Fetch entry from Rekor
        let rekor_entry = self.fetch_rekor_entry(entry_id).await?;
        
        // Convert to SigstoreEntry
        let entry = self.convert_rekor_entry(rekor_entry).await?;
        
        // Cache the entry
        self.entry_cache.insert(entry_id.to_string(), (entry.clone(), Instant::now()));
        
        Ok(entry)
    }
    
    async fn fetch_rekor_entry(&self, entry_id: &str) -> Result<RekorEntry> {
        let url = format!("{}/api/v1/log/entries/{}", self.rekor_url, entry_id);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch Rekor entry")?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to fetch Rekor entry: {}", error_text));
        }
        
        let rekor_entry: RekorEntry = response.json().await
            .context("Failed to parse Rekor entry")?;
        
        Ok(rekor_entry)
    }
    
    async fn convert_rekor_entry(&self, rekor_entry: RekorEntry) -> Result<SigstoreEntry> {
        // Parse the body to extract additional information
        let body_bytes = general_purpose::STANDARD.decode(&rekor_entry.body)
            .context("Failed to decode Rekor entry body")?;
        
        let body_json: serde_json::Value = serde_json::from_slice(&body_bytes)
            .context("Failed to parse Rekor entry body")?;
        
        // Extract signature and public key
        let signature = body_json.get("signature")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let public_key = body_json.get("publicKey")
            .and_then(|v| v.get("content"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        // Extract artifact hash
        let artifact_hash = body_json.get("spec")
            .and_then(|v| v.get("data"))
            .and_then(|v| v.get("hash"))
            .and_then(|v| v.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        // Extract artifact type
        let artifact_type = body_json.get("spec")
            .and_then(|v| v.get("data"))
            .and_then(|v| v.get("hash"))
            .and_then(|v| v.get("algorithm"))
            .and_then(|v| v.as_str())
            .unwrap_or("sha256")
            .to_string();
        
        // Get Fulcio certificate
        let fulcio_cert = self.get_fulcio_certificate(&rekor_entry.uuid).await?;
        
        // Extract OIDC information from certificate
        let (oidc_issuer, oidc_identity) = self.extract_oidc_info(&fulcio_cert).await?;
        
        let entry = SigstoreEntry {
            entry_id: rekor_entry.uuid,
            log_index: rekor_entry.log_index.to_string(),
            integrated_time: rekor_entry.integrated_time.to_string(),
            log_id: rekor_entry.log_id,
            rekor_entry_url: format!("{}/api/v1/log/entries/{}", self.rekor_url, rekor_entry.uuid),
            fulcio_certificate_url: format!("{}/api/v1/signingCert", self.fulcio_url),
            oidc_issuer,
            oidc_identity,
            signature,
            public_key,
            artifact_hash,
            artifact_type,
        };
        
        Ok(entry)
    }
    
    async fn get_fulcio_certificate(&self, entry_id: &str) -> Result<FulcioCertificate> {
        // For now, return a mock certificate
        // TODO: Implement actual Fulcio certificate retrieval
        let cert = FulcioCertificate {
            cert: "mock-certificate".to_string(),
            chain: vec!["mock-chain-cert".to_string()],
            sct: Some("mock-sct".to_string()),
        };
        
        Ok(cert)
    }
    
    async fn extract_oidc_info(&self, cert: &FulcioCertificate) -> Result<(String, String)> {
        // For now, return mock OIDC information
        // TODO: Extract actual OIDC information from certificate
        Ok((
            self.oidc_issuer.clone(),
            "mock-identity".to_string(),
        ))
    }
    
    pub async fn verify_signature(&self, artifact_hash: &str, signature: &str, public_key: &str) -> Result<bool> {
        // TODO: Implement actual signature verification
        // This would verify the signature using the public key
        info!("Verifying signature for artifact: {}", artifact_hash);
        
        // Mock verification for now
        Ok(true)
    }
    
    pub async fn verify_inclusion_proof(&self, entry_id: &str, log_index: u64, tree_size: u64) -> Result<bool> {
        // TODO: Implement actual inclusion proof verification
        // This would verify that the entry is included in the log
        info!("Verifying inclusion proof for entry: {}", entry_id);
        
        // Mock verification for now
        Ok(true)
    }
    
    pub async fn create_signed_entry(
        &self,
        artifact_hash: &str,
        artifact_type: &str,
        signature: &str,
        public_key: &str,
    ) -> Result<SigstoreEntry> {
        // TODO: Implement actual entry creation
        // This would create a signed entry in Rekor
        info!("Creating signed entry for artifact: {}", artifact_hash);
        
        let entry = SigstoreEntry {
            entry_id: uuid::Uuid::new_v4().to_string(),
            log_index: "0".to_string(),
            integrated_time: Utc::now().timestamp().to_string(),
            log_id: "mock-log-id".to_string(),
            rekor_entry_url: format!("{}/api/v1/log/entries/{}", self.rekor_url, uuid::Uuid::new_v4()),
            fulcio_certificate_url: format!("{}/api/v1/signingCert", self.fulcio_url),
            oidc_issuer: self.oidc_issuer.clone(),
            oidc_identity: "mock-identity".to_string(),
            signature: signature.to_string(),
            public_key: public_key.to_string(),
            artifact_hash: artifact_hash.to_string(),
            artifact_type: artifact_type.to_string(),
        };
        
        Ok(entry)
    }
    
    pub async fn get_log_info(&self) -> Result<LogInfo> {
        let url = format!("{}/api/v1/log", self.rekor_url);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .context("Failed to get log info")?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to get log info: {}", error_text));
        }
        
        let log_info: LogInfo = response.json().await
            .context("Failed to parse log info")?;
        
        Ok(log_info)
    }
    
    pub async fn get_public_key(&self) -> Result<String> {
        let url = format!("{}/api/v1/log/publicKey", self.rekor_url);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .context("Failed to get public key")?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to get public key: {}", error_text));
        }
        
        let public_key: serde_json::Value = response.json().await
            .context("Failed to parse public key")?;
        
        let key = public_key.get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("No key found in response"))?;
        
        Ok(key.to_string())
    }
    
    pub async fn verify_entry_consistency(&self, entry_id: &str) -> Result<bool> {
        // TODO: Implement actual consistency verification
        // This would verify that the entry is consistent with the log
        info!("Verifying entry consistency for: {}", entry_id);
        
        // Mock verification for now
        Ok(true)
    }
    
    pub async fn get_entry_by_artifact_hash(&self, artifact_hash: &str) -> Result<Option<SigstoreEntry>> {
        let url = format!("{}/api/v1/log/entries/retrieve", self.rekor_url);
        
        let request_body = serde_json::json!({
            "hash": artifact_hash,
            "logIndex": null
        });
        
        let response = self.http_client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to retrieve entry by artifact hash")?;
        
        if response.status().is_success() {
            let rekor_entry: RekorEntry = response.json().await
                .context("Failed to parse Rekor entry")?;
            
            let entry = self.convert_rekor_entry(rekor_entry).await?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogInfo {
    pub root_hash: String,
    pub tree_size: u64,
    pub signed_tree_head: String,
    pub tree_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_sigstore_client_creation() {
        let config = GitHubAppConfig::default();
        let client = SigstoreClient::new(&config).await;
        assert!(client.is_ok());
    }
    
    #[test]
    fn test_oidc_token_serialization() {
        let token = OIDCToken {
            iss: "https://oauth2.sigstore.dev/auth".to_string(),
            sub: "test-subject".to_string(),
            aud: "sigstore".to_string(),
            exp: 1234567890,
            iat: 1234567890 - 3600,
            email: Some("test@example.com".to_string()),
            email_verified: Some(true),
        };
        
        let json = serde_json::to_string(&token).unwrap();
        let deserialized: OIDCToken = serde_json::from_str(&json).unwrap();
        
        assert_eq!(token.iss, deserialized.iss);
        assert_eq!(token.sub, deserialized.sub);
        assert_eq!(token.aud, deserialized.aud);
        assert_eq!(token.exp, deserialized.exp);
        assert_eq!(token.iat, deserialized.iat);
        assert_eq!(token.email, deserialized.email);
        assert_eq!(token.email_verified, deserialized.email_verified);
    }
    
    #[test]
    fn test_rekor_entry_serialization() {
        let entry = RekorEntry {
            uuid: "test-uuid".to_string(),
            log_index: 123,
            integrated_time: 1234567890,
            log_id: "test-log-id".to_string(),
            body: "test-body".to_string(),
            verification: RekorVerification {
                inclusion_proof: None,
                signed_entry_timestamp: "test-timestamp".to_string(),
                signed_tree_head: None,
            },
        };
        
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: RekorEntry = serde_json::from_str(&json).unwrap();
        
        assert_eq!(entry.uuid, deserialized.uuid);
        assert_eq!(entry.log_index, deserialized.log_index);
        assert_eq!(entry.integrated_time, deserialized.integrated_time);
        assert_eq!(entry.log_id, deserialized.log_id);
        assert_eq!(entry.body, deserialized.body);
    }
} 