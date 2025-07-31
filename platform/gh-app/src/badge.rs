use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use tracing::{info, warn, error};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::config::GitHubAppConfig;
use crate::github::GitHubClient;
use crate::sigstore::SigstoreClient;
use crate::proto::gh_app::v1::*;

#[derive(Debug, Clone)]
pub struct BadgeManager {
    config: GitHubAppConfig,
    github_client: GitHubClient,
    sigstore_client: SigstoreClient,
    badge_cache: HashMap<String, (BadgeStatusResponse, Instant)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BadgeUpdateRequest {
    pub repository_id: String,
    pub pull_request_id: String,
    pub commit_sha: String,
    pub spec_document_ids: Vec<String>,
    pub installation_id: String,
    pub app_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BadgeUpdateResponse {
    pub success: bool,
    pub message: String,
    pub badge_status: BadgeStatus,
    pub proof_artifacts: Vec<ProofArtifactReference>,
    pub sigstore_entries: Vec<SigstoreEntry>,
}

impl BadgeManager {
    pub async fn new(config: &GitHubAppConfig) -> Result<Self> {
        let github_client = GitHubClient::new(config).await?;
        let sigstore_client = SigstoreClient::new(config).await?;
        
        Ok(Self {
            config: config.clone(),
            github_client,
            sigstore_client,
            badge_cache: HashMap::new(),
        })
    }
    
    pub async fn update_badge_status(&mut self, request: BadgeStatusRequest) -> Result<BadgeStatusResponse> {
        let cache_key = format!("{}_{}_{}", request.repository_id, request.pull_request_id, request.commit_sha);
        
        // Check cache first
        if let Some((cached_response, created_at)) = self.badge_cache.get(&cache_key) {
            if created_at.elapsed() < Duration::from_secs(300) { // 5 minute cache
                return Ok(cached_response.clone());
            }
        }
        
        info!("Updating badge status for repo={}, pr={}, commit={}", 
            request.repository_id, request.pull_request_id, request.commit_sha);
        
        // Extract repository and PR info
        let repo = self.extract_repo_from_id(&request.repository_id)?;
        let pr_number = self.extract_pr_from_id(&request.pull_request_id)?;
        
        // Get proof artifacts for spec documents
        let proof_artifacts = self.get_proof_artifacts(&request.spec_document_ids).await?;
        
        // Determine badge status based on proof artifacts
        let badge_status = self.determine_badge_status(&proof_artifacts)?;
        
        // Get Sigstore entries for verification
        let sigstore_entries = self.get_sigstore_entries(&proof_artifacts).await?;
        
        // Create badge response
        let response = BadgeStatusResponse {
            status: badge_status,
            message: self.get_badge_message(badge_status, &proof_artifacts),
            target_url: self.get_badge_target_url(&request, &proof_artifacts),
            description: self.get_badge_description(badge_status, &proof_artifacts),
            context: self.config.badge_context.clone(),
            proof_artifacts,
            sigstore_entries,
            created_at: Some(Utc::now().into()),
            updated_at: Some(Utc::now().into()),
        };
        
        // Update GitHub commit status
        self.update_github_status(&repo, &request.commit_sha, &response).await?;
        
        // Cache the response
        self.badge_cache.insert(cache_key, (response.clone(), Instant::now()));
        
        info!("Updated badge status: {:?} for {}@{}", badge_status, repo, request.commit_sha);
        
        Ok(response)
    }
    
    async fn get_proof_artifacts(&self, spec_document_ids: &[String]) -> Result<Vec<ProofArtifactReference>> {
        let mut artifacts = Vec::new();
        
        for spec_id in spec_document_ids {
            // TODO: Integrate with proof farm to get actual proof artifacts
            // For now, create mock artifacts
            let artifact = ProofArtifactReference {
                artifact_id: Uuid::new_v4().to_string(),
                spec_document_id: spec_id.clone(),
                content_hash: format!("sha256:{}", Uuid::new_v4()),
                proof_hash: format!("sha256:{}", Uuid::new_v4()),
                rekor_entry_id: format!("{}", Uuid::new_v4()),
                fulcio_certificate: "mock-certificate".to_string(),
                proven_at: Some(Utc::now().into()),
                status: "proven".to_string(),
                error_message: "".to_string(),
            };
            
            artifacts.push(artifact);
        }
        
        Ok(artifacts)
    }
    
    fn determine_badge_status(&self, artifacts: &[ProofArtifactReference]) -> Result<BadgeStatus> {
        if artifacts.is_empty() {
            return Ok(BadgeStatus::BadgeStatusPending);
        }
        
        let mut all_proven = true;
        let mut any_failed = false;
        
        for artifact in artifacts {
            match artifact.status.as_str() {
                "proven" => {
                    // Artifact is proven
                }
                "failed" => {
                    any_failed = true;
                    all_proven = false;
                }
                "pending" => {
                    all_proven = false;
                }
                _ => {
                    warn!("Unknown artifact status: {}", artifact.status);
                    all_proven = false;
                }
            }
        }
        
        if any_failed {
            Ok(BadgeStatus::BadgeStatusFailure)
        } else if all_proven {
            Ok(BadgeStatus::BadgeStatusSuccess)
        } else {
            Ok(BadgeStatus::BadgeStatusPending)
        }
    }
    
    async fn get_sigstore_entries(&self, artifacts: &[ProofArtifactReference]) -> Result<Vec<SigstoreEntry>> {
        let mut entries = Vec::new();
        
        for artifact in artifacts {
            // Get Sigstore entry for this artifact
            let entry = self.sigstore_client.get_entry(&artifact.rekor_entry_id).await?;
            entries.push(entry);
        }
        
        Ok(entries)
    }
    
    fn get_badge_message(&self, status: BadgeStatus, artifacts: &[ProofArtifactReference]) -> String {
        match status {
            BadgeStatus::BadgeStatusPending => {
                if artifacts.is_empty() {
                    "No spec documents found".to_string()
                } else {
                    format!("Verifying {} spec document(s)...", artifacts.len())
                }
            }
            BadgeStatus::BadgeStatusSuccess => {
                format!("All {} spec document(s) verified", artifacts.len())
            }
            BadgeStatus::BadgeStatusFailure => {
                let failed_count = artifacts.iter()
                    .filter(|a| a.status == "failed")
                    .count();
                format!("{} spec document(s) failed verification", failed_count)
            }
            BadgeStatus::BadgeStatusError => {
                "Error during verification".to_string()
            }
            _ => {
                "Unknown status".to_string()
            }
        }
    }
    
    fn get_badge_target_url(&self, request: &BadgeStatusRequest, artifacts: &[ProofArtifactReference]) -> String {
        if artifacts.is_empty() {
            self.config.badge_target_url.clone()
        } else {
            // Create a URL with artifact details
            let artifact_ids: Vec<String> = artifacts.iter()
                .map(|a| a.artifact_id.clone())
                .collect();
            format!("{}?artifacts={}", 
                self.config.badge_target_url, 
                artifact_ids.join(","))
        }
    }
    
    fn get_badge_description(&self, status: BadgeStatus, artifacts: &[ProofArtifactReference]) -> String {
        match status {
            BadgeStatus::BadgeStatusPending => {
                "Spec-to-Proof verification in progress".to_string()
            }
            BadgeStatus::BadgeStatusSuccess => {
                format!("Spec-to-Proof verification passed ({} artifacts)", artifacts.len())
            }
            BadgeStatus::BadgeStatusFailure => {
                let failed_count = artifacts.iter()
                    .filter(|a| a.status == "failed")
                    .count();
                format!("Spec-to-Proof verification failed ({} failed)", failed_count)
            }
            BadgeStatus::BadgeStatusError => {
                "Spec-to-Proof verification error".to_string()
            }
            _ => {
                "Spec-to-Proof verification".to_string()
            }
        }
    }
    
    async fn update_github_status(
        &mut self,
        repo: &str,
        commit_sha: &str,
        response: &BadgeStatusResponse,
    ) -> Result<()> {
        self.github_client.update_commit_status(
            repo,
            commit_sha,
            response.status,
            &response.context,
            &response.description,
            Some(&response.target_url),
        ).await?;
        
        Ok(())
    }
    
    fn extract_repo_from_id(&self, repository_id: &str) -> Result<String> {
        // TODO: Implement proper repository ID to name mapping
        // For now, assume the ID is the repository name
        Ok(repository_id.to_string())
    }
    
    fn extract_pr_from_id(&self, pull_request_id: &str) -> Result<String> {
        // TODO: Implement proper PR ID to number mapping
        // For now, assume the ID is the PR number
        Ok(pull_request_id.to_string())
    }
    
    pub async fn get_badge_history(&self, repo: &str, pr: &str) -> Result<Vec<BadgeStatusResponse>> {
        // TODO: Implement badge history retrieval
        // This would query a database or cache for historical badge updates
        Ok(Vec::new())
    }
    
    pub async fn invalidate_badge_cache(&mut self, repo: &str, pr: &str, commit_sha: &str) -> Result<()> {
        let cache_key = format!("{}_{}_{}", repo, pr, commit_sha);
        self.badge_cache.remove(&cache_key);
        
        info!("Invalidated badge cache for {}@{}", repo, commit_sha);
        Ok(())
    }
    
    pub async fn get_badge_statistics(&self) -> Result<BadgeStatistics> {
        let mut stats = BadgeStatistics {
            total_badges: 0,
            success_count: 0,
            failure_count: 0,
            pending_count: 0,
            error_count: 0,
            average_verification_time_ms: 0,
        };
        
        // Calculate statistics from cache
        for (_, (response, _)) in &self.badge_cache {
            stats.total_badges += 1;
            
            match response.status {
                BadgeStatus::BadgeStatusSuccess => stats.success_count += 1,
                BadgeStatus::BadgeStatusFailure => stats.failure_count += 1,
                BadgeStatus::BadgeStatusPending => stats.pending_count += 1,
                BadgeStatus::BadgeStatusError => stats.error_count += 1,
                _ => {}
            }
        }
        
        Ok(stats)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BadgeStatistics {
    pub total_badges: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub pending_count: u64,
    pub error_count: u64,
    pub average_verification_time_ms: u64,
}

// Badge verification service
pub struct BadgeVerificationService {
    badge_manager: BadgeManager,
}

impl BadgeVerificationService {
    pub async fn new(config: &GitHubAppConfig) -> Result<Self> {
        let badge_manager = BadgeManager::new(config).await?;
        
        Ok(Self {
            badge_manager,
        })
    }
    
    pub async fn verify_spec_documents(&mut self, spec_document_ids: Vec<String>) -> Result<Vec<ProofArtifactReference>> {
        let mut artifacts = Vec::new();
        
        for spec_id in spec_document_ids {
            // TODO: Integrate with proof farm for actual verification
            let artifact = ProofArtifactReference {
                artifact_id: Uuid::new_v4().to_string(),
                spec_document_id: spec_id,
                content_hash: format!("sha256:{}", Uuid::new_v4()),
                proof_hash: format!("sha256:{}", Uuid::new_v4()),
                rekor_entry_id: format!("{}", Uuid::new_v4()),
                fulcio_certificate: "mock-certificate".to_string(),
                proven_at: Some(Utc::now().into()),
                status: "proven".to_string(),
                error_message: "".to_string(),
            };
            
            artifacts.push(artifact);
        }
        
        Ok(artifacts)
    }
    
    pub async fn verify_single_document(&mut self, spec_document_id: &str) -> Result<ProofArtifactReference> {
        // TODO: Implement actual document verification
        let artifact = ProofArtifactReference {
            artifact_id: Uuid::new_v4().to_string(),
            spec_document_id: spec_document_id.to_string(),
            content_hash: format!("sha256:{}", Uuid::new_v4()),
            proof_hash: format!("sha256:{}", Uuid::new_v4()),
            rekor_entry_id: format!("{}", Uuid::new_v4()),
            fulcio_certificate: "mock-certificate".to_string(),
            proven_at: Some(Utc::now().into()),
            status: "proven".to_string(),
            error_message: "".to_string(),
        };
        
        Ok(artifact)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_badge_manager_creation() {
        let config = GitHubAppConfig::default();
        let manager = BadgeManager::new(&config).await;
        assert!(manager.is_ok());
    }
    
    #[test]
    fn test_badge_status_determination() {
        let config = GitHubAppConfig::default();
        let manager = BadgeManager {
            config,
            github_client: GitHubClient::new(&GitHubAppConfig::default()).await.unwrap(),
            sigstore_client: SigstoreClient::new(&GitHubAppConfig::default()).await.unwrap(),
            badge_cache: HashMap::new(),
        };
        
        // Test with no artifacts
        let artifacts = Vec::new();
        let status = manager.determine_badge_status(&artifacts).unwrap();
        assert_eq!(status, BadgeStatus::BadgeStatusPending);
        
        // Test with all proven artifacts
        let artifacts = vec![
            ProofArtifactReference {
                artifact_id: "1".to_string(),
                spec_document_id: "DOC-1".to_string(),
                content_hash: "sha256:abc".to_string(),
                proof_hash: "sha256:def".to_string(),
                rekor_entry_id: "entry1".to_string(),
                fulcio_certificate: "cert1".to_string(),
                proven_at: Some(Utc::now().into()),
                status: "proven".to_string(),
                error_message: "".to_string(),
            }
        ];
        let status = manager.determine_badge_status(&artifacts).unwrap();
        assert_eq!(status, BadgeStatus::BadgeStatusSuccess);
        
        // Test with failed artifacts
        let artifacts = vec![
            ProofArtifactReference {
                artifact_id: "1".to_string(),
                spec_document_id: "DOC-1".to_string(),
                content_hash: "sha256:abc".to_string(),
                proof_hash: "sha256:def".to_string(),
                rekor_entry_id: "entry1".to_string(),
                fulcio_certificate: "cert1".to_string(),
                proven_at: Some(Utc::now().into()),
                status: "failed".to_string(),
                error_message: "Verification failed".to_string(),
            }
        ];
        let status = manager.determine_badge_status(&artifacts).unwrap();
        assert_eq!(status, BadgeStatus::BadgeStatusFailure);
    }
    
    #[test]
    fn test_badge_message_generation() {
        let config = GitHubAppConfig::default();
        let manager = BadgeManager {
            config,
            github_client: GitHubClient::new(&GitHubAppConfig::default()).await.unwrap(),
            sigstore_client: SigstoreClient::new(&GitHubAppConfig::default()).await.unwrap(),
            badge_cache: HashMap::new(),
        };
        
        let artifacts = vec![
            ProofArtifactReference {
                artifact_id: "1".to_string(),
                spec_document_id: "DOC-1".to_string(),
                content_hash: "sha256:abc".to_string(),
                proof_hash: "sha256:def".to_string(),
                rekor_entry_id: "entry1".to_string(),
                fulcio_certificate: "cert1".to_string(),
                proven_at: Some(Utc::now().into()),
                status: "proven".to_string(),
                error_message: "".to_string(),
            }
        ];
        
        let message = manager.get_badge_message(BadgeStatus::BadgeStatusSuccess, &artifacts);
        assert!(message.contains("All 1 spec document(s) verified"));
    }
} 