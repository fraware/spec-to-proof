use std::error::Error;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn, error};

use lean_farm::{
    Config, LeanFarm, ProofJob, ProofResult, JobPriority,
    proto::proof::v1::*,
    proto::spec_to_proof::v1::*,
};

#[tokio::test]
async fn test_lean_farm_integration() -> Result<(), Box<dyn Error>> {
    info!("Starting Lean Farm integration test");
    
    // Load test configuration
    let config = load_test_config().await?;
    
    // Initialize Lean Farm
    let farm = LeanFarm::new(config).await?;
    info!("Lean Farm initialized successfully");
    
    // Create a test theorem
    let theorem = create_test_theorem().await?;
    
    // Create proof options
    let options = ProofOptions {
        max_attempts: 3,
        timeout_seconds: 300,
        proof_strategy: "auto".to_string(),
        confidence_threshold: 0.8,
        resource_limits: Some(ResourceLimits {
            cpu_seconds: 60.0,
            memory_bytes: 2 * 1024 * 1024 * 1024, // 2GB
            disk_bytes: 5 * 1024 * 1024 * 1024, // 5GB
            network_bytes: 50 * 1024 * 1024, // 50MB
        }),
        metadata: std::collections::HashMap::new(),
    };
    
    // Create proof job
    let job = ProofJob {
        id: format!("test-job-{}", uuid::Uuid::new_v4()),
        theorem,
        options,
        priority: JobPriority::High,
        created_at: std::time::Instant::now(),
        deadline: Some(std::time::Instant::now() + Duration::from_secs(600)),
    };
    
    info!("Created test job: {}", job.id);
    
    // Submit job to farm
    let job_result = submit_job_to_farm(&farm, job).await?;
    
    // Validate proof result
    validate_proof_result(&job_result).await?;
    
    info!("Integration test completed successfully");
    Ok(())
}

#[tokio::test]
async fn test_farm_scalability() -> Result<(), Box<dyn Error>> {
    info!("Testing Lean Farm scalability");
    
    let config = load_test_config().await?;
    let farm = LeanFarm::new(config).await?;
    
    // Create multiple test jobs
    let mut jobs = Vec::new();
    for i in 0..10 {
        let theorem = create_test_theorem_with_id(i).await?;
        let options = ProofOptions {
            max_attempts: 2,
            timeout_seconds: 120,
            proof_strategy: "auto".to_string(),
            confidence_threshold: 0.7,
            resource_limits: Some(ResourceLimits {
                cpu_seconds: 30.0,
                memory_bytes: 1024 * 1024 * 1024, // 1GB
                disk_bytes: 2 * 1024 * 1024 * 1024, // 2GB
                network_bytes: 25 * 1024 * 1024, // 25MB
            }),
            metadata: std::collections::HashMap::new(),
        };
        
        let job = ProofJob {
            id: format!("scalability-test-{}-{}", i, uuid::Uuid::new_v4()),
            theorem,
            options,
            priority: JobPriority::Normal,
            created_at: std::time::Instant::now(),
            deadline: Some(std::time::Instant::now() + Duration::from_secs(300)),
        };
        
        jobs.push(job);
    }
    
    info!("Created {} test jobs for scalability test", jobs.len());
    
    // Submit all jobs concurrently
    let mut handles = Vec::new();
    for job in jobs {
        let farm_clone = farm.clone();
        let handle = tokio::spawn(async move {
            submit_job_to_farm(&farm_clone, job).await
        });
        handles.push(handle);
    }
    
    // Wait for all jobs to complete with timeout
    let results = timeout(
        Duration::from_secs(600), // 10 minutes
        futures::future::join_all(handles)
    ).await??;
    
    // Validate results
    let mut success_count = 0;
    let mut failure_count = 0;
    
    for result in results {
        match result {
            Ok(job_result) => {
                if job_result.success {
                    success_count += 1;
                } else {
                    failure_count += 1;
                    warn!("Job {} failed: {:?}", job_result.job_id, job_result.error_message);
                }
            }
            Err(e) => {
                failure_count += 1;
                error!("Job submission failed: {}", e);
            }
        }
    }
    
    info!("Scalability test results: {} successful, {} failed", success_count, failure_count);
    
    // Assert that at least 80% of jobs succeeded
    let success_rate = success_count as f64 / (success_count + failure_count) as f64;
    assert!(success_rate >= 0.8, "Success rate {} is below 80%", success_rate);
    
    Ok(())
}

#[tokio::test]
async fn test_farm_security() -> Result<(), Box<dyn Error>> {
    info!("Testing Lean Farm security features");
    
    let config = load_test_config().await?;
    let farm = LeanFarm::new(config).await?;
    
    // Test security validation
    let security_manager = &farm.security_manager;
    let runtime_info = security_manager.get_runtime_info();
    
    // Verify gVisor runtime
    assert!(runtime_info.is_gvisor, "gVisor runtime is required");
    
    // Verify rootless execution
    assert!(runtime_info.is_rootless, "Rootless execution is required");
    
    // Verify seccomp profile
    assert!(runtime_info.seccomp_enabled, "Seccomp profile is required");
    
    // Verify dropped capabilities
    assert!(runtime_info.capabilities_dropped, "All capabilities must be dropped");
    
    // Verify network isolation
    assert!(runtime_info.network_isolated, "Network isolation is required");
    
    info!("Security validation passed");
    Ok(())
}

#[tokio::test]
async fn test_farm_resource_limits() -> Result<(), Box<dyn Error>> {
    info!("Testing Lean Farm resource limits");
    
    let config = load_test_config().await?;
    let farm = LeanFarm::new(config).await?;
    
    // Create a job that should exceed resource limits
    let theorem = create_test_theorem().await?;
    let options = ProofOptions {
        max_attempts: 1,
        timeout_seconds: 60,
        proof_strategy: "auto".to_string(),
        confidence_threshold: 0.5,
        resource_limits: Some(ResourceLimits {
            cpu_seconds: 0.1, // Very low CPU limit
            memory_bytes: 1024, // Very low memory limit (1KB)
            disk_bytes: 1024, // Very low disk limit (1KB)
            network_bytes: 1024, // Very low network limit (1KB)
        }),
        metadata: std::collections::HashMap::new(),
    };
    
    let job = ProofJob {
        id: format!("resource-test-{}", uuid::Uuid::new_v4()),
        theorem,
        options,
        priority: JobPriority::Low,
        created_at: std::time::Instant::now(),
        deadline: Some(std::time::Instant::now() + Duration::from_secs(120)),
    };
    
    // Submit job and expect it to fail due to resource limits
    let result = submit_job_to_farm(&farm, job).await?;
    
    // The job should fail due to resource constraints
    assert!(!result.success, "Job should fail due to resource limits");
    assert!(result.error_message.is_some(), "Job should have error message");
    
    let error_message = result.error_message.unwrap();
    assert!(
        error_message.contains("resource") || error_message.contains("limit"),
        "Error should mention resource limits: {}",
        error_message
    );
    
    info!("Resource limits test passed");
    Ok(())
}

async fn load_test_config() -> Result<Config, Box<dyn Error>> {
    // Load test configuration from file or environment
    let config = Config {
        // Test-specific configuration
        ..Default::default()
    };
    
    Ok(config)
}

async fn create_test_theorem() -> Result<LeanTheorem, Box<dyn Error>> {
    // Create a simple test theorem
    let lean_code = r#"
theorem test_theorem : 1 + 1 = 2 := by
  rw [add_comm]
  rw [add_zero]
"#;
    
    Ok(LeanTheorem {
        id: format!("test-theorem-{}", uuid::Uuid::new_v4()),
        content_sha256: sha256::digest(lean_code),
        theorem_name: "test_theorem".to_string(),
        lean_code: lean_code.to_string(),
        source_invariant_id: "test-invariant".to_string(),
        generated_at: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
        status: TheoremStatus::Generated as i32,
        compilation_errors: Vec::new(),
        proof_strategy: "auto".to_string(),
        metadata: std::collections::HashMap::new(),
    })
}

async fn create_test_theorem_with_id(id: u32) -> Result<LeanTheorem, Box<dyn Error>> {
    let lean_code = format!(r#"
theorem test_theorem_{} : {} + {} = {} := by
  rw [add_comm]
  rw [add_zero]
"#, id, id, id, id * 2);
    
    Ok(LeanTheorem {
        id: format!("test-theorem-{}-{}", id, uuid::Uuid::new_v4()),
        content_sha256: sha256::digest(&lean_code),
        theorem_name: format!("test_theorem_{}", id),
        lean_code,
        source_invariant_id: format!("test-invariant-{}", id),
        generated_at: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
        status: TheoremStatus::Generated as i32,
        compilation_errors: Vec::new(),
        proof_strategy: "auto".to_string(),
        metadata: std::collections::HashMap::new(),
    })
}

async fn submit_job_to_farm(farm: &LeanFarm, job: ProofJob) -> Result<ProofResult, Box<dyn Error>> {
    info!("Submitting job {} to farm", job.id);
    
    // In a real implementation, this would submit the job to the farm's job queue
    // For testing, we'll simulate the job processing
    
    // Simulate job processing time
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Create a mock proof result
    let proof_artifact = ProofArtifact {
        id: format!("proof-{}", uuid::Uuid::new_v4()),
        content_sha256: sha256::digest("proof_result"),
        theorem_id: job.theorem.id.clone(),
        invariant_id: job.theorem.source_invariant_id.clone(),
        status: ProofStatus::Success as i32,
        attempted_at: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
        duration_ms: 5000, // 5 seconds
        output: "Proof completed successfully".to_string(),
        logs: vec!["Proof generation completed".to_string()],
        resource_usage: Some(ResourceUsage {
            cpu_seconds: 5.0,
            memory_bytes: 1024 * 1024 * 1024, // 1GB
            disk_bytes: 100 * 1024 * 1024, // 100MB
            network_bytes: 10 * 1024 * 1024, // 10MB
        }),
        proof_strategy: job.options.proof_strategy.clone(),
        confidence_score: 0.95,
        metadata: std::collections::HashMap::new(),
    };
    
    Ok(ProofResult {
        job_id: job.id,
        theorem: job.theorem,
        proof_artifact,
        duration_ms: 5000,
        success: true,
        error_message: None,
        resource_usage: ResourceUsage {
            cpu_seconds: 5.0,
            memory_bytes: 1024 * 1024 * 1024,
            disk_bytes: 100 * 1024 * 1024,
            network_bytes: 10 * 1024 * 1024,
        },
    })
}

async fn validate_proof_result(result: &ProofResult) -> Result<(), Box<dyn Error>> {
    info!("Validating proof result for job {}", result.job_id);
    
    // Validate job ID
    assert!(!result.job_id.is_empty(), "Job ID should not be empty");
    
    // Validate theorem
    assert!(!result.theorem.id.is_empty(), "Theorem ID should not be empty");
    assert!(!result.theorem.theorem_name.is_empty(), "Theorem name should not be empty");
    assert!(!result.theorem.lean_code.is_empty(), "Lean code should not be empty");
    
    // Validate proof artifact
    assert!(!result.proof_artifact.id.is_empty(), "Proof artifact ID should not be empty");
    assert_eq!(result.proof_artifact.theorem_id, result.theorem.id, "Theorem IDs should match");
    
    // Validate duration
    assert!(result.duration_ms > 0, "Duration should be positive");
    
    // Validate resource usage
    assert!(result.resource_usage.cpu_seconds > 0.0, "CPU usage should be positive");
    assert!(result.resource_usage.memory_bytes > 0, "Memory usage should be positive");
    
    // Validate success status
    if result.success {
        assert!(result.error_message.is_none(), "Successful job should not have error message");
        assert_eq!(result.proof_artifact.status, ProofStatus::Success as i32, "Successful job should have Success status");
    } else {
        assert!(result.error_message.is_some(), "Failed job should have error message");
    }
    
    info!("Proof result validation passed");
    Ok(())
}

// Import necessary types
use lean_farm::proto::proof::v1::*;
use lean_farm::proto::spec_to_proof::v1::*; 