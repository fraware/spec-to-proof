#![no_main]

use libfuzzer_sys::fuzz_target;
use spec_to_proof_proto::{
    SpecDocumentModel, InvariantModel, InvariantSetModel, LeanTheoremModel, 
    ProofArtifactModel, BadgeStatusModel, ToProto, FromProto, calculate_sha256, generate_id
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

fuzz_target!(|data: &[u8]| {
    // Generate test data from fuzz input
    if data.len() < 10 {
        return;
    }

    // Create test SpecDocument
    let spec_doc = SpecDocumentModel {
        id: generate_id(),
        content_sha256: calculate_sha256(&String::from_utf8_lossy(data)),
        source_system: "jira".to_string(),
        source_id: format!("PROJ-{}", data[0] % 1000),
        title: format!("Test Document {}", data[1]),
        content: String::from_utf8_lossy(data).to_string(),
        url: format!("https://example.com/doc/{}", data[2]),
        author: format!("user{}@example.com", data[3]),
        created_at: Utc::now(),
        modified_at: Utc::now(),
        metadata: HashMap::new(),
        version: (data[4] % 10 + 1) as i32,
        status: spec_to_proof_proto::DocumentStatus::Published,
    };

    // Test round-trip for SpecDocument
    let proto_doc = spec_doc.to_proto();
    let round_trip_doc = SpecDocumentModel::from_proto(proto_doc).unwrap();
    
    assert_eq!(spec_doc.id, round_trip_doc.id);
    assert_eq!(spec_doc.content_sha256, round_trip_doc.content_sha256);
    assert_eq!(spec_doc.title, round_trip_doc.title);

    // Create test Invariant
    let invariant = InvariantModel {
        id: generate_id(),
        content_sha256: calculate_sha256(&String::from_utf8_lossy(data)),
        description: format!("Test invariant {}", data[5]),
        formal_expression: format!("x >= {}", data[6]),
        natural_language: "Test natural language".to_string(),
        variables: vec![
            spec_to_proof_proto::VariableModel {
                name: "x".to_string(),
                var_type: "int".to_string(),
                description: "Test variable".to_string(),
                unit: "units".to_string(),
                constraints: vec!["positive".to_string()],
            }
        ],
        units: HashMap::new(),
        confidence_score: (data[7] % 100) as f64 / 100.0,
        source_document_id: spec_doc.id.clone(),
        extracted_at: Utc::now(),
        status: spec_to_proof_proto::InvariantStatus::Extracted,
        tags: vec!["test".to_string()],
        priority: spec_to_proof_proto::Priority::Medium,
    };

    // Test round-trip for Invariant
    let proto_invariant = invariant.to_proto();
    let round_trip_invariant = InvariantModel::from_proto(proto_invariant).unwrap();
    
    assert_eq!(invariant.id, round_trip_invariant.id);
    assert_eq!(invariant.content_sha256, round_trip_invariant.content_sha256);
    assert_eq!(invariant.description, round_trip_invariant.description);

    // Create test InvariantSet
    let invariant_set = InvariantSetModel {
        id: generate_id(),
        content_sha256: calculate_sha256(&String::from_utf8_lossy(data)),
        name: format!("Test Set {}", data[8]),
        description: "Test invariant set".to_string(),
        invariants: vec![invariant],
        source_document_ids: vec![spec_doc.id.clone()],
        created_at: Utc::now(),
        modified_at: Utc::now(),
        status: spec_to_proof_proto::InvariantSetStatus::Draft,
    };

    // Create test LeanTheorem
    let lean_theorem = LeanTheoremModel {
        id: generate_id(),
        content_sha256: calculate_sha256(&String::from_utf8_lossy(data)),
        theorem_name: format!("test_theorem_{}", data[9]),
        lean_code: "theorem test_theorem : x ≥ 0 := by simp".to_string(),
        source_invariant_id: invariant.id.clone(),
        generated_at: Utc::now(),
        status: spec_to_proof_proto::TheoremStatus::Generated,
        compilation_errors: vec![],
        proof_strategy: "simp".to_string(),
        metadata: HashMap::new(),
    };

    // Create test ProofArtifact
    let proof_artifact = ProofArtifactModel {
        id: generate_id(),
        content_sha256: calculate_sha256(&String::from_utf8_lossy(data)),
        theorem_id: lean_theorem.id.clone(),
        invariant_id: invariant.id.clone(),
        status: spec_to_proof_proto::ProofStatus::Success,
        attempted_at: Utc::now(),
        duration_ms: (data[0] % 10000) as i64,
        output: "Proof successful".to_string(),
        logs: vec!["Compiling...".to_string(), "Proof found".to_string()],
        resource_usage: spec_to_proof_proto::ResourceUsageModel {
            cpu_seconds: (data[1] % 100) as f64,
            memory_bytes: (data[2] % 1000000) as i64,
            disk_bytes: (data[3] % 100000) as i64,
            network_bytes: (data[4] % 10000) as i64,
        },
        proof_strategy: "simp".to_string(),
        confidence_score: 1.0,
        metadata: HashMap::new(),
    };

    // Create test BadgeStatus
    let badge_status = BadgeStatusModel {
        id: generate_id(),
        content_sha256: calculate_sha256(&String::from_utf8_lossy(data)),
        repo_owner: "test-owner".to_string(),
        repo_name: "test-repo".to_string(),
        pr_number: (data[5] % 1000 + 1) as i32,
        commit_sha: "a".repeat(40),
        state: spec_to_proof_proto::BadgeState::Success,
        description: "Test badge".to_string(),
        target_url: "https://example.com/badge".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        proof_artifact_ids: vec![proof_artifact.id.clone()],
        coverage_percentage: 100.0,
        invariants_proven: 1,
        total_invariants: 1,
    };

    // Test SHA256 consistency
    let content = String::from_utf8_lossy(data);
    let hash1 = calculate_sha256(&content);
    let hash2 = calculate_sha256(&content);
    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 64);

    // Test ID generation uniqueness
    let id1 = generate_id();
    let id2 = generate_id();
    assert_ne!(id1, id2);
}); 