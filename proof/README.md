# Spec-to-Proof Lean Compiler Service

This service converts invariant specifications into Lean 4 theorems and generates formal proofs using Claude 3 Opus.

## Features

- **Invariant-to-Theorem Conversion**: Converts `InvariantSet` objects into Lean 4 theorem stubs
- **Proof Generation**: Uses Claude 3 Opus in tool mode to complete proofs
- **S3 Storage**: Streams Lean code to S3 with versioning by invariant hash
- **Deterministic Generation**: Temperature 0.0 with pinned seeds for reproducible results
- **Auto-retry Logic**: Exponential backoff with up to 3 retry attempts
- **Prompt Injection Guards**: Comprehensive protection against prompt injection attacks
- **Cost Tracking**: Token usage and cost estimation for all operations

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   InvariantSet  │───▶│  Lean Compiler  │───▶│  Lean Theorem   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │                       │
                                ▼                       ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │ Claude 3 Opus   │    │   S3 Storage    │
                       │   (Tool Mode)   │    │  (Versioned)    │
                       └─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │ Proof Artifact  │
                       └─────────────────┘
```

## Service Endpoints

### gRPC API

- `CompileInvariantSet`: Convert invariant set to Lean theorems
- `GenerateProof`: Generate complete proofs for Lean theorems
- `StreamLeanCode`: Upload Lean code to S3 with versioning
- `HealthCheck`: Service health status

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CLAUDE_API_KEY` | Required | Claude API key |
| `CLAUDE_MODEL` | `claude-3-opus-20240229` | Claude model to use |
| `MAX_TOKENS` | `8000` | Maximum tokens for Claude responses |
| `TEMPERATURE` | `0.0` | Temperature for deterministic generation |
| `MAX_RETRIES` | `3` | Maximum retry attempts |
| `RETRY_DELAY_MS` | `1000` | Base retry delay in milliseconds |
| `COST_PER_1K_TOKENS` | `0.015` | Claude 3 Opus cost per 1K tokens |
| `S3_BUCKET` | `spec-to-proof-lean` | S3 bucket for Lean code storage |
| `S3_REGION` | `us-east-1` | AWS region for S3 |
| `S3_KEY_PREFIX` | `theorems/` | S3 key prefix |
| `KMS_KEY_ID` | Optional | KMS key for encryption |

## Usage

### Running the Service

```bash
# Set required environment variables
export CLAUDE_API_KEY="your-api-key"

# Run the service
bazel run //proof:lean_compiler
```

### Example Invariant

```rust
let invariant = Invariant {
    id: "trivial_inv".to_string(),
    description: "Trivial arithmetic invariant".to_string(),
    formal_expression: "∀n ∈ ℕ, n + 0 = n".to_string(),
    natural_language: "For any natural number n, n plus zero equals n".to_string(),
    variables: vec![
        Variable {
            name: "n".to_string(),
            var_type: "Nat".to_string(),
            description: "Natural number".to_string(),
            unit: "".to_string(),
            constraints: vec![],
        }
    ],
    // ... other fields
};
```

### Generated Lean Theorem

```lean
import Mathlib.Data.Nat.Basic

theorem trivial_theorem (n : Nat) : n + 0 = n := by
    simp
```

## Testing

### Unit Tests

```bash
# Run all tests
bazel test //proof:all

# Run specific test
bazel test //proof:lean_compiler_test
```

### Benchmarks

The service includes benchmarks for:

- **Trivial Invariants**: ≤ 500ms compilation time
- **ResNet Examples**: ≤ 15K tokens, ≥ 99% success rate across 30 runs
- **Prompt Injection**: ≥ 10K seeds, zero escapes

### Fuzz Testing

```bash
# Run fuzz tests for prompt injection protection
bazel test //proof:lean_compiler_test --test_filter=test_fuzz_prompt_injection
```

## Performance Requirements

### Trivial Invariants
- Compilation time: ≤ 500ms
- Token usage: ≤ 1K tokens
- Success rate: 100%

### ResNet Examples
- Token usage: ≤ 15K tokens
- Success rate: ≥ 99% across 30 runs
- Processing time: ≤ 5 minutes total

### Security
- Prompt injection protection: 0 escapes from 10K seeds
- Input validation: All suspicious patterns detected
- Rate limiting: Exponential backoff on failures

## Error Handling

### Retry Logic
- Maximum 3 attempts per proof generation
- Exponential backoff: 1s, 2s, 4s delays
- Failed proofs routed to DLQ (Dead Letter Queue)

### Timeout Handling
- Default timeout: 30 seconds per proof
- Configurable via `ProofOptions.timeout_seconds`
- Graceful degradation on timeouts

## Security Features

### Prompt Injection Protection
- Pattern detection for common injection attempts
- Escape sequence validation
- Input length limits (max 5KB)
- Null byte detection

### S3 Security
- Server-side encryption (SSE-KMS or AES256)
- Versioning enabled
- Immutable tags for critical theorems
- Access logging

## Monitoring

### Metrics
- Compilation time per invariant
- Token usage and cost tracking
- Success/failure rates
- S3 upload performance

### Logging
- Structured logging with tracing
- Request/response correlation
- Error details with context
- Performance metrics

## Deployment

### Docker
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/lean_compiler /usr/local/bin/
CMD ["lean_compiler"]
```

### Kubernetes
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: lean-compiler
spec:
  replicas: 3
  selector:
    matchLabels:
      app: lean-compiler
  template:
    metadata:
      labels:
        app: lean-compiler
    spec:
      containers:
      - name: lean-compiler
        image: spec-to-proof/lean-compiler:latest
        env:
        - name: CLAUDE_API_KEY
          valueFrom:
            secretKeyRef:
              name: claude-secret
              key: api-key
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
```

## Development

### Building
```bash
# Build the service
bazel build //proof:lean_compiler

# Build with optimizations
bazel build //proof:lean_compiler --compilation_mode=opt
```

### Testing
```bash
# Run unit tests
bazel test //proof:all

# Run benchmarks
bazel test //proof:lean_compiler_test --test_filter=test_benchmark_resnet_example

# Run fuzz tests
bazel test //proof:lean_compiler_test --test_filter=test_fuzz_prompt_injection
```

### Local Development
```bash
# Start the service locally
CLAUDE_API_KEY="your-key" bazel run //proof:lean_compiler

# Test with grpcurl
grpcurl -plaintext -d '{"invariant_set": {...}}' localhost:50051 spec_to_proof.proof.v1.ProofService/CompileInvariantSet
```

## Contributing

1. Follow the existing code style
2. Add tests for new functionality
3. Update documentation
4. Ensure all tests pass
5. Run benchmarks to verify performance
6. Check security with fuzz tests

## License

This project is licensed under the MIT License - see the LICENSE file for details. 