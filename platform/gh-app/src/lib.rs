pub mod config;
pub mod github;
pub mod webhook;
pub mod badge;
pub mod sigstore;
pub mod auth;
pub mod proto;
pub mod error;
pub mod server;
pub mod workflows;
pub mod webhook_handlers;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    routing::{post, get},
    Router,
    http::{HeaderMap, StatusCode},
    Json,
    extract::{State, Path},
};
use serde::{Deserialize, Serialize};
use tracing::{info, error, warn};
use anyhow::Result;

use crate::config::GitHubAppConfig;
use crate::github::GitHubClient;
use crate::webhook::WebhookProcessor;
use crate::badge::BadgeManager;
use crate::sigstore::SigstoreClient;
use crate::auth::JWTManager;
use crate::proto::gh_app::v1::*;

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: GitHubAppConfig,
    pub github_client: Arc<GitHubClient>,
    pub webhook_processor: Arc<WebhookProcessor>,
    pub badge_manager: Arc<BadgeManager>,
    pub sigstore_client: Arc<SigstoreClient>,
    pub jwt_manager: Arc<JWTManager>,
    pub metrics: Arc<RwLock<HashMap<String, u64>>>,
}

impl AppState {
    pub async fn new(config: GitHubAppConfig) -> Result<Self> {
        let github_client = Arc::new(GitHubClient::new(&config).await?);
        let webhook_processor = Arc::new(WebhookProcessor::new(&config).await?);
        let badge_manager = Arc::new(BadgeManager::new(&config).await?);
        let sigstore_client = Arc::new(SigstoreClient::new(&config).await?);
        let jwt_manager = Arc::new(JWTManager::new(&config).await?);
        let metrics = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            config,
            github_client,
            webhook_processor,
            badge_manager,
            sigstore_client,
            jwt_manager,
            metrics,
        })
    }
}

pub async fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/webhook", post(handle_webhook))
        .route("/badge/:repo/:pr", post(update_badge))
        .route("/health", get(health_check))
        .route("/metrics", get(get_metrics))
        .with_state(Arc::new(state))
}

async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Result<Json<ProcessWebhookResponse>, (StatusCode, String)> {
    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    let signature = headers
        .get("X-Hub-Signature-256")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    let delivery_id = headers
        .get("X-GitHub-Delivery")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    info!("Received webhook: event={}, delivery_id={}", event_type, delivery_id);

    // Verify webhook signature
    let verification_request = WebhookVerificationRequest {
        payload: body.clone(),
        signature: signature.to_string(),
        webhook_secret: state.config.webhook_secret.clone(),
    };

    let verification_response = state.webhook_processor.verify_webhook(verification_request).await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Webhook verification failed: {}", e)))?;

    if !verification_response.valid {
        return Err((StatusCode::UNAUTHORIZED, "Invalid webhook signature".to_string()));
    }

    // Process webhook
    let request = ProcessWebhookRequest {
        payload: body,
        signature: signature.to_string(),
        event_type: event_type.to_string(),
        delivery_id: delivery_id.to_string(),
        installation_id: "".to_string(), // Will be extracted from payload
    };

    let response = state.webhook_processor.process_webhook(request).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Webhook processing failed: {}", e)))?;

    // Update metrics
    {
        let mut metrics = state.metrics.write().await;
        *metrics.entry(format!("webhook_{}", event_type)).or_insert(0) += 1;
        *metrics.entry("webhook_total".to_string()).or_insert(0) += 1;
    }

    Ok(Json(response))
}

async fn update_badge(
    State(state): State<Arc<AppState>>,
    Path((repo, pr)): Path<(String, String)>,
    Json(request): Json<BadgeStatusRequest>,
) -> Result<Json<BadgeStatusResponse>, (StatusCode, String)> {
    info!("Updating badge for repo={}, pr={}", repo, pr);

    let response = state.badge_manager.update_badge_status(request).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Badge update failed: {}", e)))?;

    // Update metrics
    {
        let mut metrics = state.metrics.write().await;
        *metrics.entry(format!("badge_{:?}", response.status)).or_insert(0) += 1;
        *metrics.entry("badge_total".to_string()).or_insert(0) += 1;
    }

    Ok(Json(response))
}

async fn health_check(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HealthCheckResponse>, (StatusCode, String)> {
    let checks = HashMap::new(); // TODO: Implement actual health checks

    let response = HealthCheckResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: "0s".to_string(), // TODO: Track actual uptime
        checks: checks.into_iter().map(|(k, v)| (k, v.to_string())).collect(),
    };

    Ok(Json(response))
}

async fn get_metrics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HashMap<String, u64>>, (StatusCode, String)> {
    let metrics = state.metrics.read().await;
    Ok(Json(metrics.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GitHubAppConfig;

    #[tokio::test]
    async fn test_app_state_creation() {
        let config = GitHubAppConfig::default();
        let state = AppState::new(config).await;
        assert!(state.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = GitHubAppConfig::default();
        let state = AppState::new(config).await.unwrap();
        let response = health_check(State(Arc::new(state))).await;
        assert!(response.is_ok());
    }
} 