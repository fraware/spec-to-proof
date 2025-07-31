# Lean Farm

A secure, horizontally scalable Kubernetes job runner for Lean theorem proving with production-grade security guarantees.

## Features

- **Horizontal Scaling**: Scale to 500+ pods with automatic load balancing
- **Security First**: gVisor runtime, seccomp restrictions, rootless execution
- **Resource Management**: CPU/memory caps with guaranteed ≥99.9% availability
- **Docker Integration**: Secure Lean container execution with read-only mounts
- **S3/MinIO Storage**: Immutable code bundles and proof artifacts
- **Monitoring**: Prometheus metrics and health checks
- **OSS Scanning**: Zero critical vulnerabilities requirement

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Kubernetes    │    │   Lean Farm     │    │   Storage       │
│   Cluster       │    │   Pods          │    │   Layer         │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ HPA         │ │    │ │ Job Runner  │ │    │ │ S3          │ │
│ │ (1-500 pods)│ │    │ │ (Docker)    │ │    │ │ (Code)      │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ PDB         │ │    │ │ Security    │ │    │ │ MinIO       │ │
│ │ (HA)        │ │    │ │ Manager     │ │    │ │ (Artifacts) │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Security Features

### Runtime Security
- **gVisor Runtime**: Syscall isolation for enhanced security
- **Seccomp Profiles**: Restricted system call access
- **Rootless Execution**: Non-root user (UID 1000)
- **Read-only Root FS**: Immutable container filesystem
- **Dropped Capabilities**: All Linux capabilities removed
- **Network Isolation**: Restricted network access

### Resource Security
- **CPU Limits**: Configurable CPU cores per pod
- **Memory Limits**: Strict memory boundaries
- **Process Limits**: Maximum process count
- **File Descriptor Limits**: Controlled file access

### OSS Security
- **Vulnerability Scanning**: Trivy integration
- **Zero Critical Vulns**: Strict security requirements
- **SBOM Generation**: Software bill of materials
- **License Compliance**: Automated license checking

## Installation

### Prerequisites

- Kubernetes cluster (1.20+)
- Docker runtime
- Helm 3.0+
- gVisor runtime (optional but recommended)

### Quick Start

1. **Add Helm Repository**
```bash
helm repo add lean-farm https://spec-to-proof.github.io/lean-farm
helm repo update
```

2. **Install Lean Farm**
```bash
helm install lean-farm lean-farm/lean-farm \
  --namespace spec-to-proof \
  --create-namespace \
  --set image.tag=1.0.0 \
  --set hpa.enabled=true \
  --set hpa.maxReplicas=500
```

3. **Verify Installation**
```bash
kubectl get pods -n spec-to-proof
kubectl get hpa -n spec-to-proof
```

### Advanced Configuration

```yaml
# values-production.yaml
image:
  repository: lean-farm
  tag: "1.0.0"
  digest: "sha256:abc123..."

security:
  use_gvisor: true
  run_as_non_root: true
  run_as_user: 1000
  read_only_root_filesystem: true
  drop_all_capabilities: true

hpa:
  enabled: true
  minReplicas: 10
  maxReplicas: 500
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

resources:
  requests:
    cpu: "500m"
    memory: "1Gi"
  limits:
    cpu: "2"
    memory: "4Gi"

storage:
  s3:
    bucket: "spec-to-proof-lean"
    region: "us-east-1"
    keyPrefix: "theorems/"
  minio:
    endpoint: "minio.spec-to-proof.svc.cluster.local:9000"
    bucket: "proof-artifacts"
```

## Development

### Building from Source

```bash
# Clone repository
git clone https://github.com/spec-to-proof/lean-farm.git
cd lean-farm

# Build binary
cargo build --release

# Build Docker image
docker build -t lean-farm:1.0.0 .

# Run security benchmark
./scripts/security-benchmark.sh
```

### Local Development

```bash
# Run with local configuration
cargo run -- --config=config/local.yaml

# Run tests
cargo test

# Run integration tests
cargo test --test integration_test
```

## Monitoring

### Health Checks

```bash
# Check health endpoint
curl http://localhost:8080/health

# Check readiness
curl http://localhost:8080/ready

# Get metrics
curl http://localhost:9090/metrics
```

### Prometheus Metrics

- `lean_farm_jobs_total`: Total jobs processed
- `lean_farm_jobs_duration_seconds`: Job processing duration
- `lean_farm_jobs_success_rate`: Success rate percentage
- `lean_farm_queue_size`: Current queue size
- `lean_farm_active_workers`: Number of active workers

### Grafana Dashboard

Import the provided Grafana dashboard JSON to monitor:
- Job processing rates
- Success/failure rates
- Resource utilization
- Queue metrics
- Security events

## Testing

### Unit Tests
```bash
cargo test
```

### Integration Tests
```bash
cargo test --test integration_test
```

### Security Tests
```bash
./scripts/security-benchmark.sh
```

### Load Tests
```bash
# Run load test with k6
k6 run tests/load-test.js
```

## Troubleshooting

### Common Issues

1. **Pod Startup Failures**
```bash
# Check pod logs
kubectl logs -n spec-to-proof deployment/lean-farm

# Check events
kubectl get events -n spec-to-proof --sort-by='.lastTimestamp'
```

2. **Security Validation Failures**
```bash
# Run security benchmark
./scripts/security-benchmark.sh

# Check runtime info
kubectl exec -n spec-to-proof deployment/lean-farm -- env | grep -E "(RUNSC|SECCOMP|USER)"
```

3. **Resource Limit Issues**
```bash
# Check resource usage
kubectl top pods -n spec-to-proof

# Check HPA status
kubectl describe hpa -n spec-to-proof
```

### Debug Mode

```bash
# Enable debug logging
kubectl set env deployment/lean-farm -n spec-to-proof RUST_LOG=debug

# Check debug logs
kubectl logs -n spec-to-proof deployment/lean-farm -f
```

## Performance

### Benchmarks

- **Throughput**: 1000+ jobs/hour per pod
- **Latency**: P99 < 90 seconds
- **Availability**: ≥99.9% uptime
- **Scalability**: 1-500 pods with auto-scaling

### Resource Requirements

- **Minimum**: 500m CPU, 1Gi memory per pod
- **Recommended**: 2 CPU, 4Gi memory per pod
- **Storage**: 10GB temporary storage per pod
