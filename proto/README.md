# Spec-to-Proof Protocol Buffers

This module contains the core domain models and protocol buffer definitions for the Spec-to-Proof platform.

## Overview

The Spec-to-Proof platform uses Protocol Buffers (protobuf) as the primary data serialization format for all domain models. This ensures:

- **Type Safety**: Strong typing across all language implementations
- **Versioning**: Backward compatibility through buf breaking change detection
- **Content Addressing**: SHA256 hashes for all content-based addressing
- **Cross-Language Support**: Generated code for Rust, TypeScript, and other languages

## Domain Models

### Core Entities

1. **SpecDocument**: Represents specification documents from any source (Jira, Confluence, Google Docs)
2. **Invariant**: Formal invariants extracted from specifications
3. **InvariantSet**: Collections of related invariants
4. **LeanTheorem**: Lean 4 theorems generated from invariants
5. **ProofArtifact**: Results of proof attempts
6. **BadgeStatus**: GitHub PR status badges

### Key Features

- **SHA256 Content Addressing**: All entities include SHA256 hashes for content-based addressing
- **Comprehensive Metadata**: Rich metadata support for extensibility
- **Status Tracking**: Detailed status enums for workflow management
- **Resource Usage**: Detailed resource consumption tracking for proofs
- **Validation**: Zod schemas for TypeScript and JSON Schema for downstream validation

## File Structure

```
proto/
├── spec_to_proof.proto          # Main protobuf definitions
├── buf.yaml                     # Buf configuration
├── buf.gen.yaml                 # Code generation configuration
├── Cargo.toml                   # Rust dependencies
├── build.rs                     # Tonic build script
├── src/
│   └── lib.rs                   # Rust domain models and traits
├── fuzz/
│   ├── Cargo.toml              # Fuzz testing dependencies
│   └── fuzz_targets/
│       └── round_trip.rs       # Round-trip encode/decode fuzz tests
├── schemas/
│   └── spec-to-proof.json      # JSON Schema for validation
└── Makefile                    # Build and test targets
```

## Usage

### Rust

```rust
use spec_to_proof_proto::{
    SpecDocumentModel, InvariantModel, ToProto, FromProto, calculate_sha256, generate_id
};

// Create a spec document
let doc = SpecDocumentModel {
    id: generate_id(),
    content_sha256: calculate_sha256("document content"),
    source_system: "jira".to_string(),
    // ... other fields
};

// Convert to protobuf
let proto = doc.to_proto();

// Convert back from protobuf
let round_trip = SpecDocumentModel::from_proto(proto).unwrap();
```

### TypeScript

```typescript
import { 
    validateSpecDocument, 
    validateInvariant, 
    fromApiSpecDocument,
    toApiSpecDocument 
} from '../types/spec-to-proof';

// Validate a spec document
const doc = validateSpecDocument({
    id: '123e4567-e89b-12d3-a456-426614174000',
    contentSha256: 'a'.repeat(64),
    // ... other fields
});

// Convert from API format
const apiDoc = fromApiSpecDocument(apiResponse);

// Convert to API format
const apiFormat = toApiSpecDocument(doc);
```

## Testing

### Rust Tests

- **Unit Tests**: Comprehensive test coverage for all domain models
- **Property-Based Tests**: Using proptest for thorough testing
- **Fuzz Tests**: Round-trip encode/decode testing with ≥100 fuzz seeds
- **Integration Tests**: End-to-end workflow testing

### TypeScript Tests

- **Unit Tests**: Validation and conversion testing
- **Property-Based Tests**: Using fast-check for ≥95% property coverage
- **Type Guards**: Runtime type checking
- **API Conversion**: Bidirectional API format conversion

### Validation

- **Buf Breaking**: Protobuf backward compatibility checks
- **JSON Schema**: Comprehensive schema validation
- **Zod Schemas**: TypeScript runtime validation
- **Linting**: Code quality and style checks

## Development

### Prerequisites

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install buf
curl -sSL \
  "https://github.com/bufbuild/buf/releases/download/v1.28.1/buf-$(uname -s)-$(uname -m)" \
  -o "$(go env GOPATH)/bin/buf" && chmod +x "$(go env GOPATH)/bin/buf"

# Install cargo-fuzz
cargo install cargo-fuzz

# Install Node.js dependencies
npm install
```

### Build and Test

```bash
# Build the proto library
make build

# Run all tests
make test

# Run fuzz tests
make fuzz

# Run property-based tests
make property-test

# Validate schemas
make validate

# Full CI check
make ci
```

### Code Generation

```bash
# Generate code from protobuf
make buf-generate

# Check for breaking changes
make buf-breaking

# Lint protobuf files
make lint
```

## Schema Validation

### JSON Schema

The `schemas/spec-to-proof.json` file provides comprehensive JSON Schema validation for all domain models. This enables:

- **Runtime Validation**: Validate data at API boundaries
- **Documentation**: Self-documenting schemas
- **IDE Support**: Autocomplete and validation in IDEs
- **Testing**: Schema-based test data generation

### Zod Schemas

TypeScript validation using Zod provides:

- **Type Safety**: Runtime type checking
- **Error Messages**: Detailed validation error messages
- **Transformation**: Automatic data transformation
- **Composition**: Reusable schema components

## Performance

### Benchmarks

- **Serialization**: < 1ms for typical documents
- **Validation**: < 0.1ms for Zod validation
- **SHA256**: < 0.01ms for content hashing
- **Round-trip**: < 2ms for encode/decode cycles

### Memory Usage

- **Rust**: Zero-copy deserialization where possible
- **TypeScript**: Efficient object creation and validation
- **Protobuf**: Compact binary serialization

## Security

### Content Integrity

- **SHA256 Hashing**: All content is SHA256 hashed for integrity
- **Immutable References**: Content-based addressing prevents tampering
- **Audit Trail**: All changes are tracked through hashes

### Input Validation

- **Schema Validation**: All inputs validated against schemas
- **Type Safety**: Strong typing prevents injection attacks
- **Sanitization**: Input sanitization at boundaries

## Contributing

### Adding New Fields

1. **Update Protobuf**: Add fields to `spec_to_proof.proto`
2. **Update Rust Models**: Add corresponding fields to Rust structs
3. **Update TypeScript**: Add fields to TypeScript interfaces
4. **Update Schemas**: Update JSON Schema and Zod schemas
5. **Add Tests**: Add comprehensive tests for new fields
6. **Check Breaking**: Ensure no breaking changes with `make buf-breaking`

### Testing Guidelines

1. **Unit Tests**: Test all new functionality
2. **Property Tests**: Use property-based testing for complex logic
3. **Fuzz Tests**: Add fuzz tests for serialization/deserialization
4. **Integration Tests**: Test end-to-end workflows
5. **Performance Tests**: Benchmark new features

### Code Quality

1. **Formatting**: Use `make format` for consistent formatting
2. **Linting**: Fix all linting issues with `make lint`
3. **Security**: Run security checks with `make security`
4. **Documentation**: Update documentation for all changes

## License

MIT License - see LICENSE file for details. 