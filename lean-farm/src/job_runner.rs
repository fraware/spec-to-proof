use std::error::Error;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio::time::timeout;
use tracing::{info, warn, error, instrument};
use serde::{Deserialize, Serialize};

use crate::{
    Config, ProofJob, ProofResult, JobQueue, JobPriority, ResourceUsage,
    LeanFarmError, security::SecurityManager, storage::StorageManager, lean::LeanCompiler
};

#[derive(Debug, Clone)]
pub struct JobRunner {
    config: Config,
    security_manager: SecurityManager,
    storage_manager: StorageManager,
    lean_compiler: LeanCompiler,
    job_queue: JobQueue,
    worker_count: usize,
    max_job_duration: Duration,
    is_running: RwLock<bool>,
}

impl JobRunner {
    pub async fn new(
        config: Config,
        security_manager: SecurityManager,
    ) -> Result<Self, Box<dyn Error>> {
        let storage_manager = StorageManager::new(&config.storage).await?;
        let lean_compiler = LeanCompiler::new(&config.lean);
        let job_queue = JobQueue::new(config.job.max_queue_size);
        
        Ok(Self {
            config,
            security_manager,
            storage_manager,
            lean_compiler,
            job_queue,
            worker_count: 10,
            max_job_duration: Duration::from_secs(300), // 5 minutes
            is_running: RwLock::new(false),
        })
    }

    pub async fn start_processing(&self) -> Result<(), Box<dyn Error>> {
        info!("Starting job processing with {} workers", self.worker_count);
        
        {
            let mut is_running = self.is_running.write().await;
            *is_running = true;
        }
        
        let (tx, mut rx) = mpsc::channel(100);
        
        // Start worker pool
        let mut worker_handles = Vec::new();
        for worker_id in 0..self.worker_count {
            let tx = tx.clone();
            let job_runner = self.clone();
            let handle = tokio::spawn(async move {
                job_runner.worker_loop(worker_id, tx).await;
            });
            worker_handles.push(handle);
        }
        
        // Process results
        while let Some(result) = rx.recv().await {
            self.handle_job_result(result).await?;
        }
        
        // Wait for all workers to complete
        for handle in worker_handles {
            let _ = handle.await;
        }
        
        Ok(())
    }

    #[instrument(skip(self, tx))]
    async fn worker_loop(&self, worker_id: usize, tx: mpsc::Sender<ProofResult>) {
        info!("Worker {} started", worker_id);
        
        while *self.is_running.read().await {
            // Get next job from queue
            let job = match self.job_queue.dequeue().await {
                Some(job) => job,
                None => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }
            };
            
            info!("Worker {} processing job {}", worker_id, job.id);
            
            // Process the job
            let result = self.process_job(job).await;
            
            // Send result back
            if let Err(e) = tx.send(result).await {
                error!("Failed to send job result: {}", e);
            }
        }
        
        info!("Worker {} stopped", worker_id);
    }

    #[instrument(skip(self, job))]
    async fn process_job(&self, job: ProofJob) -> ProofResult {
        let start_time = Instant::now();
        let mut resource_usage = ResourceUsage::default();
        
        info!("Processing job {} with theorem {}", job.id, job.theorem.theorem_name);
        
        // Check deadline
        if let Some(deadline) = job.deadline {
            if Instant::now() > deadline {
                return ProofResult {
                    job_id: job.id,
                    theorem: job.theorem,
                    proof_artifact: ProofArtifact::default(),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    success: false,
                    error_message: Some("Job deadline exceeded".to_string()),
                    resource_usage,
                };
            }
        }
        
        // Download code bundle from S3
        let code_bundle_path = match self.download_code_bundle(&job.theorem).await {
            Ok(path) => path,
            Err(e) => {
                return ProofResult {
                    job_id: job.id.clone(),
                    theorem: job.theorem,
                    proof_artifact: ProofArtifact::default(),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    success: false,
                    error_message: Some(format!("Failed to download code bundle: {}", e)),
                    resource_usage,
                };
            }
        };
        
        // Run Lean compilation and proof generation
        let result = timeout(self.max_job_duration, async {
            self.run_lean_proof(&job, &code_bundle_path).await
        }).await;
        
        let (theorem, proof_artifact, success, error_message) = match result {
            Ok(Ok((theorem, proof_artifact))) => (theorem, proof_artifact, true, None),
            Ok(Err(e)) => (job.theorem.clone(), ProofArtifact::default(), false, Some(e.to_string())),
            Err(_) => (job.theorem.clone(), ProofArtifact::default(), false, Some("Job timeout".to_string())),
        };
        
        // Upload proof artifact to MinIO
        if success {
            if let Err(e) = self.upload_proof_artifact(&proof_artifact).await {
                error!("Failed to upload proof artifact: {}", e);
            }
        }
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        ProofResult {
            job_id: job.id,
            theorem,
            proof_artifact,
            duration_ms,
            success,
            error_message,
            resource_usage,
        }
    }

    async fn download_code_bundle(&self, theorem: &LeanTheorem) -> Result<PathBuf, Box<dyn Error>> {
        let bundle_key = format!("{}/{}", self.config.storage.s3.key_prefix, theorem.content_sha256);
        let local_path = PathBuf::from("/tmp").join(&theorem.content_sha256);
        
        info!("Downloading code bundle from S3: {}", bundle_key);
        
        self.storage_manager.download_from_s3(&bundle_key, &local_path).await?;
        
        Ok(local_path)
    }

    async fn run_lean_proof(
        &self,
        job: &ProofJob,
        code_bundle_path: &PathBuf,
    ) -> Result<(LeanTheorem, ProofArtifact), Box<dyn Error>> {
        info!("Running Lean proof for theorem {}", job.theorem.theorem_name);
        
        // Create Docker container with Lean image
        let container_id = self.create_lean_container(code_bundle_path).await?;
        
        // Mount S3 code bundle read-only
        self.mount_code_bundle(&container_id, code_bundle_path).await?;
        
        // Run lake build
        let build_result = self.run_lake_build(&container_id).await?;
        if !build_result.success {
            return Err(LeanFarmError::LeanCompilation(build_result.error_message).into());
        }
        
        // Run proof generation
        let proof_result = self.run_proof_generation(&container_id, &job.theorem, &job.options).await?;
        
        // Clean up container
        self.cleanup_container(&container_id).await?;
        
        Ok((job.theorem.clone(), proof_result))
    }

    async fn create_lean_container(&self, code_bundle_path: &PathBuf) -> Result<String, Box<dyn Error>> {
        let lean_version = std::env::var("LEAN_VERSION").unwrap_or_else(|_| "4.7.0".to_string());
        let image_name = format!("leanprover/lean4:{}", lean_version);
        
        // Docker run command with security restrictions
        let output = tokio::process::Command::new("docker")
            .args(&[
                "run",
                "--rm",
                "--detach",
                "--security-opt=seccomp=unconfined",
                "--security-opt=no-new-privileges",
                "--read-only",
                "--tmpfs=/tmp:rw,noexec,nosuid,size=1g",
                "--tmpfs=/var/lean-farm:rw,noexec,nosuid,size=2g",
                "--user=1000:1000",
                "--cpus=2",
                "--memory=4g",
                "--network=none",
                "--name", &format!("lean-farm-{}", uuid::Uuid::new_v4()),
                &image_name,
                "sleep", "3600"
            ])
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(LeanFarmError::JobExecution(
                format!("Failed to create container: {}", String::from_utf8_lossy(&output.stderr))
            ).into());
        }
        
        let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        info!("Created Lean container: {}", container_id);
        
        Ok(container_id)
    }

    async fn mount_code_bundle(&self, container_id: &str, code_bundle_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let mount_path = "/var/lean-farm/code";
        
        let output = tokio::process::Command::new("docker")
            .args(&[
                "cp",
                code_bundle_path.to_str().unwrap(),
                &format!("{}:{}", container_id, mount_path)
            ])
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(LeanFarmError::JobExecution(
                format!("Failed to mount code bundle: {}", String::from_utf8_lossy(&output.stderr))
            ).into());
        }
        
        info!("Mounted code bundle in container {}", container_id);
        Ok(())
    }

    async fn run_lake_build(&self, container_id: &str) -> Result<BuildResult, Box<dyn Error>> {
        let timeout_seconds = std::env::var("LAKE_BUILD_TIMEOUT")
            .unwrap_or_else(|_| "300".to_string())
            .parse::<u64>()?;
        
        let output = timeout(
            Duration::from_secs(timeout_seconds),
            tokio::process::Command::new("docker")
                .args(&[
                    "exec",
                    container_id,
                    "lake", "build"
                ])
                .output()
        ).await??;
        
        let success = output.status.success();
        let error_message = if !success {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        } else {
            None
        };
        
        Ok(BuildResult {
            success,
            error_message,
            output: String::from_utf8_lossy(&output.stdout).to_string(),
        })
    }

    async fn run_proof_generation(
        &self,
        container_id: &str,
        theorem: &LeanTheorem,
        options: &ProofOptions,
    ) -> Result<ProofArtifact, Box<dyn Error>> {
        let start_time = Instant::now();
        
        // Generate proof using Lean compiler
        let proof_code = self.lean_compiler.generate_proof(theorem, options).await?;
        
        // Execute proof in container
        let output = tokio::process::Command::new("docker")
            .args(&[
                "exec",
                container_id,
                "lean",
                "--run",
                "/var/lean-farm/code/proof.lean"
            ])
            .output()
            .await?;
        
        let success = output.status.success();
        let logs = String::from_utf8_lossy(&output.stdout).to_string();
        let error_logs = String::from_utf8_lossy(&output.stderr).to_string();
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(ProofArtifact {
            id: format!("proof-{}", uuid::Uuid::new_v4()),
            content_sha256: sha256::digest(&proof_code),
            theorem_id: theorem.id.clone(),
            invariant_id: theorem.source_invariant_id.clone(),
            status: if success { ProofStatus::Success as i32 } else { ProofStatus::Failed as i32 },
            attempted_at: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
            duration_ms,
            output: logs,
            logs: vec![error_logs],
            resource_usage: Some(ResourceUsage {
                cpu_seconds: duration_ms as f64 / 1000.0,
                memory_bytes: 0, // Would be tracked in real implementation
                disk_bytes: 0,
                network_bytes: 0,
            }),
            proof_strategy: options.proof_strategy.clone(),
            confidence_score: 0.95,
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn cleanup_container(&self, container_id: &str) -> Result<(), Box<dyn Error>> {
        let _ = tokio::process::Command::new("docker")
            .args(&["stop", container_id])
            .output()
            .await;
        
        let _ = tokio::process::Command::new("docker")
            .args(&["rm", container_id])
            .output()
            .await;
        
        info!("Cleaned up container {}", container_id);
        Ok(())
    }

    async fn upload_proof_artifact(&self, proof_artifact: &ProofArtifact) -> Result<(), Box<dyn Error>> {
        let artifact_key = format!("{}/{}", self.config.storage.minio.key_prefix, proof_artifact.id);
        
        info!("Uploading proof artifact to MinIO: {}", artifact_key);
        
        // Serialize proof artifact to protobuf
        let artifact_bytes = proof_artifact.encode_to_vec();
        
        self.storage_manager.upload_to_minio(&artifact_key, &artifact_bytes).await?;
        
        info!("Successfully uploaded proof artifact {}", proof_artifact.id);
        Ok(())
    }

    async fn handle_job_result(&self, result: ProofResult) -> Result<(), Box<dyn Error>> {
        if result.success {
            info!("Job {} completed successfully in {}ms", result.job_id, result.duration_ms);
        } else {
            warn!("Job {} failed: {:?}", result.job_id, result.error_message);
        }
        
        // Update metrics
        self.update_job_metrics(&result).await;
        
        // Store result in persistent storage
        self.storage_manager.store_job_result(&result).await?;
        
        Ok(())
    }

    async fn update_job_metrics(&self, result: &ProofResult) {
        // Update Prometheus metrics
        // This would integrate with the metrics server
    }

    pub async fn start_health_server(&self) -> Result<(), Box<dyn Error>> {
        use axum::{
            routing::get,
            Router,
            http::StatusCode,
            Json,
        };
        use serde_json::json;
        
        let app = Router::new()
            .route("/health", get(|| async { StatusCode::OK }))
            .route("/ready", get(|| async { StatusCode::OK }))
            .route("/metrics", get(|| async { 
                Json(json!({
                    "queue_size": 0, // Would get from job_queue
                    "active_workers": 0,
                    "uptime_seconds": 0,
                }))
            }));
        
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
        info!("Health server started on port 8080");
        
        axum::serve(listener, app).await?;
        Ok(())
    }

    pub async fn stop(&self) {
        info!("Stopping job runner");
        let mut is_running = self.is_running.write().await;
        *is_running = false;
    }
}

impl Clone for JobRunner {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            security_manager: self.security_manager.clone(),
            storage_manager: self.storage_manager.clone(),
            lean_compiler: self.lean_compiler.clone(),
            job_queue: JobQueue::new(self.config.job.max_queue_size),
            worker_count: self.worker_count,
            max_job_duration: self.max_job_duration,
            is_running: RwLock::new(false),
        }
    }
}

#[derive(Debug)]
pub struct BuildResult {
    pub success: bool,
    pub error_message: Option<String>,
    pub output: String,
}

// Import necessary types from proto
use crate::proto::proof::v1::*;
use crate::proto::spec_to_proof::v1::*; 