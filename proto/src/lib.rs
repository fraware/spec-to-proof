pub mod spec_to_proof {
    tonic::include_proto!("spec_to_proof.v1");
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;

pub use spec_to_proof::*;

// Re-export the generated protobuf types
pub use spec_to_proof::{
    badge_status::BadgeState,
    document_status::DocumentStatus,
    invariant::InvariantStatus,
    invariant_set::InvariantSetStatus,
    lean_theorem::TheoremStatus,
    proof_artifact::ProofStatus,
    BadgeStatus, DocumentStatus as ProtoDocumentStatus, Invariant, InvariantSet,
    InvariantSetStatus as ProtoInvariantSetStatus, InvariantStatus as ProtoInvariantStatus,
    LeanTheorem, Priority, ProofArtifact, ProofStatus as ProtoProofStatus,
    ResourceUsage, SpecDocument, TheoremStatus as ProtoTheoremStatus, Variable,
};

// Rust domain models with conversion traits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecDocumentModel {
    pub id: String,
    pub content_sha256: String,
    pub source_system: String,
    pub source_id: String,
    pub title: String,
    pub content: String,
    pub url: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
    pub version: i32,
    pub status: DocumentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantModel {
    pub id: String,
    pub content_sha256: String,
    pub description: String,
    pub formal_expression: String,
    pub natural_language: String,
    pub variables: Vec<VariableModel>,
    pub units: HashMap<String, String>,
    pub confidence_score: f64,
    pub source_document_id: String,
    pub extracted_at: DateTime<Utc>,
    pub status: InvariantStatus,
    pub tags: Vec<String>,
    pub priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableModel {
    pub name: String,
    pub var_type: String,
    pub description: String,
    pub unit: String,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantSetModel {
    pub id: String,
    pub content_sha256: String,
    pub name: String,
    pub description: String,
    pub invariants: Vec<InvariantModel>,
    pub source_document_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub status: InvariantSetStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeanTheoremModel {
    pub id: String,
    pub content_sha256: String,
    pub theorem_name: String,
    pub lean_code: String,
    pub source_invariant_id: String,
    pub generated_at: DateTime<Utc>,
    pub status: TheoremStatus,
    pub compilation_errors: Vec<String>,
    pub proof_strategy: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofArtifactModel {
    pub id: String,
    pub content_sha256: String,
    pub theorem_id: String,
    pub invariant_id: String,
    pub status: ProofStatus,
    pub attempted_at: DateTime<Utc>,
    pub duration_ms: i64,
    pub output: String,
    pub logs: Vec<String>,
    pub resource_usage: ResourceUsageModel,
    pub proof_strategy: String,
    pub confidence_score: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageModel {
    pub cpu_seconds: f64,
    pub memory_bytes: i64,
    pub disk_bytes: i64,
    pub network_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeStatusModel {
    pub id: String,
    pub content_sha256: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub pr_number: i32,
    pub commit_sha: String,
    pub state: BadgeState,
    pub description: String,
    pub target_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub proof_artifact_ids: Vec<String>,
    pub coverage_percentage: f64,
    pub invariants_proven: i32,
    pub total_invariants: i32,
}

// Rust enums for better type safety
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DocumentStatus {
    Unspecified,
    Draft,
    Published,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InvariantStatus {
    Unspecified,
    Extracted,
    Confirmed,
    Rejected,
    Proven,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InvariantSetStatus {
    Unspecified,
    Draft,
    Review,
    Approved,
    Proven,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TheoremStatus {
    Unspecified,
    Generated,
    Compiling,
    Compiled,
    Proving,
    Proven,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProofStatus {
    Unspecified,
    Pending,
    Running,
    Success,
    Failed,
    Timeout,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BadgeState {
    Unspecified,
    Pending,
    Success,
    Failure,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    Unspecified,
    Low,
    Medium,
    High,
    Critical,
}

// Conversion traits
pub trait ToProto {
    type ProtoType;
    fn to_proto(&self) -> Self::ProtoType;
}

pub trait FromProto {
    type ProtoType;
    fn from_proto(proto: Self::ProtoType) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;
}

// Implementation of conversion traits
impl ToProto for SpecDocumentModel {
    type ProtoType = SpecDocument;

    fn to_proto(&self) -> Self::ProtoType {
        SpecDocument {
            id: self.id.clone(),
            content_sha256: self.content_sha256.clone(),
            source_system: self.source_system.clone(),
            source_id: self.source_id.clone(),
            title: self.title.clone(),
            content: self.content.clone(),
            url: self.url.clone(),
            author: self.author.clone(),
            created_at: Some(self.created_at.into()),
            modified_at: Some(self.modified_at.into()),
            metadata: self.metadata.clone(),
            version: self.version,
            status: self.status.to_proto() as i32,
        }
    }
}

impl FromProto for SpecDocumentModel {
    type ProtoType = SpecDocument;

    fn from_proto(proto: Self::ProtoType) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(SpecDocumentModel {
            id: proto.id,
            content_sha256: proto.content_sha256,
            source_system: proto.source_system,
            source_id: proto.source_id,
            title: proto.title,
            content: proto.content,
            url: proto.url,
            author: proto.author,
            created_at: proto.created_at.unwrap_or_default().try_into()?,
            modified_at: proto.modified_at.unwrap_or_default().try_into()?,
            metadata: proto.metadata,
            version: proto.version,
            status: DocumentStatus::from_proto(proto.status),
        })
    }
}

impl ToProto for InvariantModel {
    type ProtoType = Invariant;

    fn to_proto(&self) -> Self::ProtoType {
        Invariant {
            id: self.id.clone(),
            content_sha256: self.content_sha256.clone(),
            description: self.description.clone(),
            formal_expression: self.formal_expression.clone(),
            natural_language: self.natural_language.clone(),
            variables: self.variables.iter().map(|v| v.to_proto()).collect(),
            units: self.units.clone(),
            confidence_score: self.confidence_score,
            source_document_id: self.source_document_id.clone(),
            extracted_at: Some(self.extracted_at.into()),
            status: self.status.to_proto() as i32,
            tags: self.tags.clone(),
            priority: self.priority.to_proto() as i32,
        }
    }
}

impl FromProto for InvariantModel {
    type ProtoType = Invariant;

    fn from_proto(proto: Self::ProtoType) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(InvariantModel {
            id: proto.id,
            content_sha256: proto.content_sha256,
            description: proto.description,
            formal_expression: proto.formal_expression,
            natural_language: proto.natural_language,
            variables: proto
                .variables
                .into_iter()
                .map(|v| VariableModel::from_proto(v))
                .collect::<Result<Vec<_>, _>>()?,
            units: proto.units,
            confidence_score: proto.confidence_score,
            source_document_id: proto.source_document_id,
            extracted_at: proto.extracted_at.unwrap_or_default().try_into()?,
            status: InvariantStatus::from_proto(proto.status),
            tags: proto.tags,
            priority: Priority::from_proto(proto.priority),
        })
    }
}

impl ToProto for VariableModel {
    type ProtoType = Variable;

    fn to_proto(&self) -> Self::ProtoType {
        Variable {
            name: self.name.clone(),
            r#type: self.var_type.clone(),
            description: self.description.clone(),
            unit: self.unit.clone(),
            constraints: self.constraints.clone(),
        }
    }
}

impl FromProto for VariableModel {
    type ProtoType = Variable;

    fn from_proto(proto: Self::ProtoType) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(VariableModel {
            name: proto.name,
            var_type: proto.r#type,
            description: proto.description,
            unit: proto.unit,
            constraints: proto.constraints,
        })
    }
}

// Enum conversion implementations
impl ToProto for DocumentStatus {
    type ProtoType = i32;

    fn to_proto(&self) -> Self::ProtoType {
        match self {
            DocumentStatus::Unspecified => 0,
            DocumentStatus::Draft => 1,
            DocumentStatus::Published => 2,
            DocumentStatus::Archived => 3,
        }
    }
}

impl FromProto for DocumentStatus {
    type ProtoType = i32;

    fn from_proto(proto: Self::ProtoType) -> Self {
        match proto {
            0 => DocumentStatus::Unspecified,
            1 => DocumentStatus::Draft,
            2 => DocumentStatus::Published,
            3 => DocumentStatus::Archived,
            _ => DocumentStatus::Unspecified,
        }
    }
}

impl ToProto for InvariantStatus {
    type ProtoType = i32;

    fn to_proto(&self) -> Self::ProtoType {
        match self {
            InvariantStatus::Unspecified => 0,
            InvariantStatus::Extracted => 1,
            InvariantStatus::Confirmed => 2,
            InvariantStatus::Rejected => 3,
            InvariantStatus::Proven => 4,
            InvariantStatus::Failed => 5,
        }
    }
}

impl FromProto for InvariantStatus {
    type ProtoType = i32;

    fn from_proto(proto: Self::ProtoType) -> Self {
        match proto {
            0 => InvariantStatus::Unspecified,
            1 => InvariantStatus::Extracted,
            2 => InvariantStatus::Confirmed,
            3 => InvariantStatus::Rejected,
            4 => InvariantStatus::Proven,
            5 => InvariantStatus::Failed,
            _ => InvariantStatus::Unspecified,
        }
    }
}

impl ToProto for Priority {
    type ProtoType = i32;

    fn to_proto(&self) -> Self::ProtoType {
        match self {
            Priority::Unspecified => 0,
            Priority::Low => 1,
            Priority::Medium => 2,
            Priority::High => 3,
            Priority::Critical => 4,
        }
    }
}

impl FromProto for Priority {
    type ProtoType = i32;

    fn from_proto(proto: Self::ProtoType) -> Self {
        match proto {
            0 => Priority::Unspecified,
            1 => Priority::Low,
            2 => Priority::Medium,
            3 => Priority::High,
            4 => Priority::Critical,
            _ => Priority::Unspecified,
        }
    }
}

// Utility functions for SHA256 hashing
pub fn calculate_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

// JSON Schema generation
pub fn generate_json_schema() -> serde_json::Value {
    serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "Spec-to-Proof Domain Models",
        "type": "object",
        "properties": {
            "spec_document": {
                "$ref": "#/definitions/SpecDocument"
            },
            "invariant": {
                "$ref": "#/definitions/Invariant"
            },
            "invariant_set": {
                "$ref": "#/definitions/InvariantSet"
            },
            "lean_theorem": {
                "$ref": "#/definitions/LeanTheorem"
            },
            "proof_artifact": {
                "$ref": "#/definitions/ProofArtifact"
            },
            "badge_status": {
                "$ref": "#/definitions/BadgeStatus"
            }
        },
        "definitions": {
            "SpecDocument": {
                "type": "object",
                "required": ["id", "content_sha256", "source_system", "title", "content"],
                "properties": {
                    "id": {"type": "string"},
                    "content_sha256": {"type": "string", "pattern": "^[a-fA-F0-9]{64}$"},
                    "source_system": {"type": "string"},
                    "source_id": {"type": "string"},
                    "title": {"type": "string"},
                    "content": {"type": "string"},
                    "url": {"type": "string", "format": "uri"},
                    "author": {"type": "string"},
                    "created_at": {"type": "string", "format": "date-time"},
                    "modified_at": {"type": "string", "format": "date-time"},
                    "metadata": {"type": "object", "additionalProperties": {"type": "string"}},
                    "version": {"type": "integer", "minimum": 1},
                    "status": {"$ref": "#/definitions/DocumentStatus"}
                }
            },
            "DocumentStatus": {
                "type": "string",
                "enum": ["unspecified", "draft", "published", "archived"]
            },
            "Invariant": {
                "type": "object",
                "required": ["id", "content_sha256", "description", "formal_expression"],
                "properties": {
                    "id": {"type": "string"},
                    "content_sha256": {"type": "string", "pattern": "^[a-fA-F0-9]{64}$"},
                    "description": {"type": "string"},
                    "formal_expression": {"type": "string"},
                    "natural_language": {"type": "string"},
                    "variables": {"type": "array", "items": {"$ref": "#/definitions/Variable"}},
                    "units": {"type": "object", "additionalProperties": {"type": "string"}},
                    "confidence_score": {"type": "number", "minimum": 0.0, "maximum": 1.0},
                    "source_document_id": {"type": "string"},
                    "extracted_at": {"type": "string", "format": "date-time"},
                    "status": {"$ref": "#/definitions/InvariantStatus"},
                    "tags": {"type": "array", "items": {"type": "string"}},
                    "priority": {"$ref": "#/definitions/Priority"}
                }
            },
            "Variable": {
                "type": "object",
                "required": ["name", "type"],
                "properties": {
                    "name": {"type": "string"},
                    "type": {"type": "string"},
                    "description": {"type": "string"},
                    "unit": {"type": "string"},
                    "constraints": {"type": "array", "items": {"type": "string"}}
                }
            },
            "InvariantStatus": {
                "type": "string",
                "enum": ["unspecified", "extracted", "confirmed", "rejected", "proven", "failed"]
            },
            "Priority": {
                "type": "string",
                "enum": ["unspecified", "low", "medium", "high", "critical"]
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_spec_document_round_trip(doc: SpecDocumentModel) {
            let proto = doc.to_proto();
            let round_trip = SpecDocumentModel::from_proto(proto).unwrap();
            assert_eq!(doc.id, round_trip.id);
            assert_eq!(doc.content_sha256, round_trip.content_sha256);
            assert_eq!(doc.title, round_trip.title);
        }

        #[test]
        fn test_invariant_round_trip(invariant: InvariantModel) {
            let proto = invariant.to_proto();
            let round_trip = InvariantModel::from_proto(proto).unwrap();
            assert_eq!(invariant.id, round_trip.id);
            assert_eq!(invariant.content_sha256, round_trip.content_sha256);
            assert_eq!(invariant.description, round_trip.description);
        }

        #[test]
        fn test_sha256_hashing(content: String) {
            let hash1 = calculate_sha256(&content);
            let hash2 = calculate_sha256(&content);
            assert_eq!(hash1, hash2);
            assert_eq!(hash1.len(), 64);
        }
    }

    // Arbitrary implementations for property-based testing
    impl Arbitrary for SpecDocumentModel {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (
                any::<String>(),
                any::<String>(),
                any::<String>(),
                any::<String>(),
                any::<String>(),
                any::<String>(),
                any::<String>(),
                any::<String>(),
                any::<DateTime<Utc>>(),
                any::<DateTime<Utc>>(),
                any::<HashMap<String, String>>(),
                any::<i32>(),
                any::<DocumentStatus>(),
            )
                .prop_map(|(id, content_sha256, source_system, source_id, title, content, url, author, created_at, modified_at, metadata, version, status)| {
                    SpecDocumentModel {
                        id,
                        content_sha256,
                        source_system,
                        source_id,
                        title,
                        content,
                        url,
                        author,
                        created_at,
                        modified_at,
                        metadata,
                        version,
                        status,
                    }
                })
                .boxed()
        }
    }

    impl Arbitrary for InvariantModel {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (
                any::<String>(),
                any::<String>(),
                any::<String>(),
                any::<String>(),
                any::<String>(),
                any::<Vec<VariableModel>>(),
                any::<HashMap<String, String>>(),
                any::<f64>(),
                any::<String>(),
                any::<DateTime<Utc>>(),
                any::<InvariantStatus>(),
                any::<Vec<String>>(),
                any::<Priority>(),
            )
                .prop_map(|(id, content_sha256, description, formal_expression, natural_language, variables, units, confidence_score, source_document_id, extracted_at, status, tags, priority)| {
                    InvariantModel {
                        id,
                        content_sha256,
                        description,
                        formal_expression,
                        natural_language,
                        variables,
                        units,
                        confidence_score,
                        source_document_id,
                        extracted_at,
                        status,
                        tags,
                        priority,
                    }
                })
                .boxed()
        }
    }

    impl Arbitrary for VariableModel {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (
                any::<String>(),
                any::<String>(),
                any::<String>(),
                any::<String>(),
                any::<Vec<String>>(),
            )
                .prop_map(|(name, var_type, description, unit, constraints)| {
                    VariableModel {
                        name,
                        var_type,
                        description,
                        unit,
                        constraints,
                    }
                })
                .boxed()
        }
    }

    impl Arbitrary for DocumentStatus {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                Just(DocumentStatus::Unspecified),
                Just(DocumentStatus::Draft),
                Just(DocumentStatus::Published),
                Just(DocumentStatus::Archived),
            ]
            .boxed()
        }
    }

    impl Arbitrary for InvariantStatus {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                Just(InvariantStatus::Unspecified),
                Just(InvariantStatus::Extracted),
                Just(InvariantStatus::Confirmed),
                Just(InvariantStatus::Rejected),
                Just(InvariantStatus::Proven),
                Just(InvariantStatus::Failed),
            ]
            .boxed()
        }
    }

    impl Arbitrary for Priority {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                Just(Priority::Unspecified),
                Just(Priority::Low),
                Just(Priority::Medium),
                Just(Priority::High),
                Just(Priority::Critical),
            ]
            .boxed()
        }
    }

    impl Arbitrary for DateTime<Utc> {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (0..253402300799i64)
                .prop_map(|timestamp| {
                    DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now())
                })
                .boxed()
        }
    }
} 