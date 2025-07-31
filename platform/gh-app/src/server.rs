use std::net::SocketAddr;
use std::sync::Arc;
use axum::{
    routing::{post, get},
    Router,
    http::{HeaderMap, StatusCode},
    Json,
    extract::{State, Path},
    middleware,
    response::IntoResponse,
};
use tower_http::{
    cors::{CorsLayer, Any},
    trace::TraceLayer,
    compression::CompressionLayer,
};
use tracing::{info, warn, error};
use anyhow::Result;

use crate::config::GitHubAppConfig;
use crate::lib::{AppState, create_app};
use crate::proto::gh_app::v1::*;
use crate::error::{GitHubAppError, ErrorResponse};

pub struct Server {
    config: GitHubAppConfig,
    app: Router,
}

impl Server {
    pub async fn new(config: GitHubAppConfig) -> Result<Self> {
        let state = AppState::new(config.clone()).await?;
        let app = create_app(state).await;
        
        // Add middleware
        let app = app
            .layer(TraceLayer::new_for_http())
            .layer(CompressionLayer::new())
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any)
            );
        
        Ok(Self { config, app })
    }
    
    pub async fn run(&self, addr: SocketAddr) -> Result<()> {
        info!("Starting GitHub App server on {}", addr);
        
        let listener = tokio::net::TcpListener::bind(addr).await?;
        info!("Listening on {}", addr);
        
        axum::serve(listener, self.app.clone()).await?;
        
        Ok(())
    }
    
    pub async fn run_with_config(&self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port)
            .parse::<SocketAddr>()?;
        
        self.run(addr).await
    }
}

// Custom error handler for the application
pub async fn handle_error(err: axum::BoxError) -> impl IntoResponse {
    let error_response = ErrorResponse::new(
        "INTERNAL_ERROR",
        &err.to_string(),
        "INTERNAL_001"
    );
    
    (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
}

// Health check endpoint
pub async fn health_check_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HealthCheckResponse>, (StatusCode, Json<ErrorResponse>)> {
    let checks = HashMap::new(); // TODO: Implement actual health checks
    
    let response = HealthCheckResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: "0s".to_string(), // TODO: Track actual uptime
        checks: checks.into_iter().map(|(k, v)| (k, v.to_string())).collect(),
    };
    
    Ok(Json(response))
}

// Metrics endpoint
pub async fn metrics_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HashMap<String, u64>>, (StatusCode, Json<ErrorResponse>)> {
    let metrics = state.metrics.read().await;
    Ok(Json(metrics.clone()))
}

// Webhook handler with custom error handling
pub async fn webhook_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Result<Json<ProcessWebhookResponse>, (StatusCode, Json<ErrorResponse>)> {
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
        .map_err(|e| {
            let error_response = ErrorResponse::new(
                "WEBHOOK_ERROR",
                &format!("Webhook verification failed: {}", e),
                "WEBHOOK_001"
            );
            (StatusCode::BAD_REQUEST, Json(error_response))
        })?;

    if !verification_response.valid {
        let error_response = ErrorResponse::new(
            "WEBHOOK_ERROR",
            "Invalid webhook signature",
            "WEBHOOK_001"
        );
        return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
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
        .map_err(|e| {
            let error_response = ErrorResponse::new(
                "WEBHOOK_ERROR",
                &format!("Webhook processing failed: {}", e),
                "WEBHOOK_004"
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    // Update metrics
    {
        let mut metrics = state.metrics.write().await;
        *metrics.entry(format!("webhook_{}", event_type)).or_insert(0) += 1;
        *metrics.entry("webhook_total".to_string()).or_insert(0) += 1;
    }

    Ok(Json(response))
}

// Badge update handler
pub async fn badge_handler(
    State(state): State<Arc<AppState>>,
    Path((repo, pr)): Path<(String, String)>,
    Json(request): Json<BadgeStatusRequest>,
) -> Result<Json<BadgeStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Updating badge for repo={}, pr={}", repo, pr);

    let response = state.badge_manager.update_badge_status(request).await
        .map_err(|e| {
            let error_response = ErrorResponse::new(
                "BADGE_ERROR",
                &format!("Badge update failed: {}", e),
                "BADGE_001"
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    // Update metrics
    {
        let mut metrics = state.metrics.write().await;
        *metrics.entry(format!("badge_{:?}", response.status)).or_insert(0) += 1;
        *metrics.entry("badge_total".to_string()).or_insert(0) += 1;
    }

    Ok(Json(response))
}

// Sigstore verification handler
pub async fn sigstore_verification_handler(
    State(state): State<Arc<AppState>>,
    Path(entry_id): Path<String>,
) -> Result<Json<SigstoreEntry>, (StatusCode, Json<ErrorResponse>)> {
    info!("Verifying Sigstore entry: {}", entry_id);

    let entry = state.sigstore_client.get_entry(&entry_id).await
        .map_err(|e| {
            let error_response = ErrorResponse::new(
                "SIGSTORE_ERROR",
                &format!("Failed to get Sigstore entry: {}", e),
                "SIGSTORE_001"
            );
            (StatusCode::NOT_FOUND, Json(error_response))
        })?;

    Ok(Json(entry))
}

// Proof artifacts handler
pub async fn proof_artifacts_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GetProofArtifactsRequest>,
) -> Result<Json<GetProofArtifactsResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Getting proof artifacts for repo={}, commit={}", 
        request.repository_id, request.commit_sha);

    // TODO: Implement actual proof artifacts retrieval
    let artifacts = Vec::new();
    let total_count = artifacts.len() as i32;
    let proven_count = artifacts.iter().filter(|a| a.status == "proven").count() as i32;
    let failed_count = artifacts.iter().filter(|a| a.status == "failed").count() as i32;
    let pending_count = artifacts.iter().filter(|a| a.status == "pending").count() as i32;

    let response = GetProofArtifactsResponse {
        artifacts,
        total_count,
        proven_count,
        failed_count,
        pending_count,
    };

    Ok(Json(response))
}

// Rate limiting middleware
pub async fn rate_limit_middleware(
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next<axum::body::Body>,
) -> Result<axum::http::Response<axum::body::Body>, axum::http::StatusCode> {
    // TODO: Implement actual rate limiting
    // For now, just pass through
    Ok(next.run(request).await)
}

// Logging middleware
pub async fn logging_middleware(
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next<axum::body::Body>,
) -> axum::http::Response<axum::body::Body> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();
    
    info!("{} {} - Starting request", method, uri);
    
    let response = next.run(request).await;
    
    let duration = start.elapsed();
    info!("{} {} - Completed in {:?}", method, uri, duration);
    
    response
}

// Error handling middleware
pub async fn error_handling_middleware(
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next<axum::body::Body>,
) -> Result<axum::http::Response<axum::body::Body>, axum::http::StatusCode> {
    match next.run(request).await {
        Ok(response) => Ok(response),
        Err(_) => {
            error!("Request failed");
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Graceful shutdown handler
pub async fn graceful_shutdown() {
    info!("Shutting down server gracefully...");
    
    // TODO: Implement graceful shutdown logic
    // - Close database connections
    // - Cancel ongoing operations
    // - Wait for in-flight requests to complete
    
    info!("Server shutdown complete");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_server_creation() {
        let config = GitHubAppConfig::default();
        let server = Server::new(config).await;
        assert!(server.is_ok());
    }
    
    #[test]
    fn test_error_response_creation() {
        let response = ErrorResponse::new("TEST_ERROR", "Test message", "TEST_001");
        assert_eq!(response.error, "TEST_ERROR");
        assert_eq!(response.message, "Test message");
        assert_eq!(response.code, "TEST_001");
    }
    
    #[tokio::test]
    async fn test_health_check_handler() {
        let config = GitHubAppConfig::default();
        let state = AppState::new(config).await.unwrap();
        let response = health_check_handler(State(Arc::new(state))).await;
        assert!(response.is_ok());
    }
} 