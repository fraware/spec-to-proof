pub mod config;
pub mod job_runner;
pub mod security;
pub mod metrics;
pub mod storage;
pub mod lean;
pub mod proto;

use std::error::Error;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, error};

use crate::proto::proof::v1::*;
use crate::proto::spec_to_proof::v1::*;

/// Lean Farm Job Runner
/// A secure, horizontally scalable Kubernetes job runner for Lean theorem proving
pub struct LeanFarm {
    config: config::Config,
    job_runner: job_runner::JobRunner,
    security_manager: security::SecurityManager,
    metrics_server: metrics::MetricsServer,
    start_time: Instant,
}

impl LeanFarm {
    pub async fn new(config: config::Config) -> Result<Self, Box<dyn Error>> {
        let security_manager = security::SecurityManager::new(&config.security)?;
        let job_runner = job_runner::JobRunner::new(config.clone(), security_manager.clone()).await?;
        let metrics_server = metrics::MetricsServer::new(
            config.metrics.port,
            config.metrics.path.clone(),
        );

        Ok(Self {
            config,
            job_runner,
            security_manager,
            metrics_server,
            start_time: Instant::now(),
        })
    }

    pub async fn start(&self) -> Result<(), Box<dyn Error>> {
        info!("Starting Lean Farm with configuration: {:?}", self.config);
        
        // Validate security requirements
        self.security_manager.validate_environment().await?;
        info!("Security validation passed");
        
        // Start metrics server
        let metrics_handle = tokio::spawn(self.metrics_server.start());
        info!("Metrics server started");
        
        // Start health check server
        let health_handle = tokio::spawn(self.job_runner.start_health_server());
        info!("Health check server started");
        
        // Start job processing
        let job_handle = tokio::spawn(self.job_runner.start_processing());
        info!("Job processing started");
        
        // Wait for all tasks to complete
        tokio::try_join!(
            metrics_handle,
            health_handle,
            job_handle
        )?;
        
        Ok(())
    }

    pub fn get_uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

/// Proof Job represents a single Lean theorem proving job
#[derive(Debug, Clone)]
pub struct ProofJob {
    pub id: String,
    pub theorem: LeanTheorem,
    pub options: ProofOptions,
    pub priority: JobPriority,
    pub created_at: Instant,
    pub deadline: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl From<i32> for JobPriority {
    fn from(value: i32) -> Self {
        match value {
            0 => JobPriority::Low,
            1 => JobPriority::Normal,
            2 => JobPriority::High,
            3 => JobPriority::Critical,
            _ => JobPriority::Normal,
        }
    }
}

/// Proof Result represents the outcome of a proof job
#[derive(Debug, Clone)]
pub struct ProofResult {
    pub job_id: String,
    pub theorem: LeanTheorem,
    pub proof_artifact: ProofArtifact,
    pub duration_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
    pub resource_usage: ResourceUsage,
}

/// Resource usage tracking
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub cpu_seconds: f64,
    pub memory_bytes: u64,
    pub disk_bytes: u64,
    pub network_bytes: u64,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_seconds: 0.0,
            memory_bytes: 0,
            disk_bytes: 0,
            network_bytes: 0,
        }
    }
}

/// Job Queue for managing proof jobs
pub struct JobQueue {
    jobs: RwLock<Vec<ProofJob>>,
    max_queue_size: usize,
}

impl JobQueue {
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            jobs: RwLock::new(Vec::new()),
            max_queue_size,
        }
    }

    pub async fn enqueue(&self, job: ProofJob) -> Result<(), Box<dyn Error>> {
        let mut jobs = self.jobs.write().await;
        
        if jobs.len() >= self.max_queue_size {
            return Err("Job queue is full".into());
        }
        
        jobs.push(job);
        jobs.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        info!("Job {} enqueued with priority {:?}", job.id, job.priority);
        Ok(())
    }

    pub async fn dequeue(&self) -> Option<ProofJob> {
        let mut jobs = self.jobs.write().await;
        jobs.pop()
    }

    pub async fn size(&self) -> usize {
        self.jobs.read().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.jobs.read().await.is_empty()
    }
}

/// Error types for the Lean Farm
#[derive(Debug, thiserror::Error)]
pub enum LeanFarmError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Security validation failed: {0}")]
    Security(String),
    
    #[error("Job execution failed: {0}")]
    JobExecution(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Lean compilation failed: {0}")]
    LeanCompilation(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Network error: {0}")]
    Network(String),
}

impl From<LeanFarmError> for Box<dyn Error> {
    fn from(error: LeanFarmError) -> Self {
        Box::new(error)
    }
} 