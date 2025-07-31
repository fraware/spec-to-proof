#!/bin/bash

# Spec-to-Proof Platform - Kind Cluster Setup Script
# This script creates a local Kubernetes cluster and deploys the full spec-to-proof platform

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CLUSTER_NAME="spec-to-proof"
NAMESPACE="spec-to-proof"
REGISTRY_PORT="5000"
REGISTRY_NAME="spec-to-proof-registry"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    local missing_commands=()
    
    if ! command_exists kind; then
        missing_commands+=("kind")
    fi
    
    if ! command_exists kubectl; then
        missing_commands+=("kubectl")
    fi
    
    if ! command_exists helm; then
        missing_commands+=("helm")
    fi
    
    if ! command_exists docker; then
        missing_commands+=("docker")
    fi
    
    if [ ${#missing_commands[@]} -ne 0 ]; then
        print_error "Missing required commands: ${missing_commands[*]}"
        print_status "Please install the missing commands and try again."
        exit 1
    fi
    
    print_success "All prerequisites are installed"
}

# Create Kind cluster configuration
create_cluster_config() {
    print_status "Creating Kind cluster configuration..."
    
    cat > kind-config.yaml << EOF
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
name: ${CLUSTER_NAME}
nodes:
- role: control-plane
  extraPortMappings:
  - containerPort: 80
    hostPort: 80
    protocol: TCP
  - containerPort: 443
    hostPort: 443
    protocol: TCP
  - containerPort: 3000
    hostPort: 3000
    protocol: TCP
  - containerPort: 8080
    hostPort: 8080
    protocol: TCP
  - containerPort: 50051
    hostPort: 50051
    protocol: TCP
  - containerPort: 9090
    hostPort: 9090
    protocol: TCP
  - containerPort: 3001
    hostPort: 3001
    protocol: TCP
  - containerPort: 9091
    hostPort: 9091
    protocol: TCP
- role: worker
- role: worker
containerdConfigPatches:
- |-
  [plugins."io.containerd.grpc.v1.cri".registry.mirrors."localhost:${REGISTRY_PORT}"]
    endpoint = ["http://${REGISTRY_NAME}:${REGISTRY_PORT}"]
EOF
    
    print_success "Kind cluster configuration created"
}

# Create local registry
create_registry() {
    print_status "Creating local Docker registry..."
    
    # Check if registry already exists
    if docker ps -q -f name="${REGISTRY_NAME}" | grep -q .; then
        print_warning "Registry already exists, skipping creation"
        return
    fi
    
    docker run -d --name "${REGISTRY_NAME}" \
        --restart=always \
        -p "${REGISTRY_PORT}:5000" \
        registry:2
    
    print_success "Local Docker registry created"
}

# Create Kind cluster
create_cluster() {
    print_status "Creating Kind cluster..."
    
    # Check if cluster already exists
    if kind get clusters | grep -q "${CLUSTER_NAME}"; then
        print_warning "Cluster ${CLUSTER_NAME} already exists"
        read -p "Do you want to delete and recreate it? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            print_status "Deleting existing cluster..."
            kind delete cluster --name "${CLUSTER_NAME}"
        else
            print_status "Using existing cluster"
            return
        fi
    fi
    
    kind create cluster --name "${CLUSTER_NAME}" --config kind-config.yaml
    
    # Connect registry to cluster
    docker network connect kind "${REGISTRY_NAME}" 2>/dev/null || true
    
    print_success "Kind cluster created successfully"
}

# Install Helm repositories
setup_helm() {
    print_status "Setting up Helm repositories..."
    
    helm repo add bitnami https://charts.bitnami.com/bitnami
    helm repo add nats https://nats-io.github.io/k8s/helm/charts/
    helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
    helm repo add grafana https://grafana.github.io/helm-charts
    helm repo update
    
    print_success "Helm repositories configured"
}

# Create namespace
create_namespace() {
    print_status "Creating namespace..."
    
    kubectl create namespace "${NAMESPACE}" --dry-run=client -o yaml | kubectl apply -f -
    
    print_success "Namespace ${NAMESPACE} created"
}

# Install dependencies
install_dependencies() {
    print_status "Installing dependencies..."
    
    # Install PostgreSQL
    helm upgrade --install postgresql bitnami/postgresql \
        --namespace "${NAMESPACE}" \
        --set auth.postgresPassword=spec_to_proof_password \
        --set auth.database=spec_to_proof \
        --set auth.username=spec_to_proof \
        --set auth.password=spec_to_proof_password \
        --set primary.persistence.enabled=true \
        --set primary.persistence.size=10Gi \
        --wait --timeout=5m
    
    # Install Redis
    helm upgrade --install redis bitnami/redis \
        --namespace "${NAMESPACE}" \
        --set auth.enabled=true \
        --set auth.password=redis_password \
        --set master.persistence.enabled=true \
        --set master.persistence.size=5Gi \
        --wait --timeout=5m
    
    # Install NATS
    helm upgrade --install nats nats/nats \
        --namespace "${NAMESPACE}" \
        --set nats.jetstream.enabled=true \
        --set nats.jetstream.memStorage.enabled=true \
        --set nats.jetstream.memStorage.size=2Gi \
        --set nats.jetstream.fileStorage.enabled=true \
        --set nats.jetstream.fileStorage.size=10Gi \
        --wait --timeout=5m
    
    print_success "Dependencies installed successfully"
}

# Build and push images
build_images() {
    print_status "Building and pushing images..."
    
    # Set up environment variables for image building
    export DOCKER_BUILDKIT=1
    
    # Build Lean Farm image
    print_status "Building Lean Farm image..."
    docker build -t localhost:${REGISTRY_PORT}/lean-farm:1.0.0 ./lean-farm/
    docker push localhost:${REGISTRY_PORT}/lean-farm:1.0.0
    
    # Build NLP image
    print_status "Building NLP image..."
    docker build -t localhost:${REGISTRY_PORT}/nlp:1.0.0 ./nlp/
    docker push localhost:${REGISTRY_PORT}/nlp:1.0.0
    
    # Build Ingest image
    print_status "Building Ingest image..."
    docker build -t localhost:${REGISTRY_PORT}/ingest:1.0.0 ./ingest/
    docker push localhost:${REGISTRY_PORT}/ingest:1.0.0
    
    # Build Proof image
    print_status "Building Proof image..."
    docker build -t localhost:${REGISTRY_PORT}/proof:1.0.0 ./proof/
    docker push localhost:${REGISTRY_PORT}/proof:1.0.0
    
    # Build Platform image
    print_status "Building Platform image..."
    docker build -t localhost:${REGISTRY_PORT}/platform:1.0.0 ./platform/gh-app/
    docker push localhost:${REGISTRY_PORT}/platform:1.0.0
    
    # Build UI image
    print_status "Building UI image..."
    docker build -t localhost:${REGISTRY_PORT}/ui:1.0.0 ./platform/ui/
    docker push localhost:${REGISTRY_PORT}/ui:1.0.0
    
    print_success "All images built and pushed successfully"
}

# Deploy spec-to-proof platform
deploy_platform() {
    print_status "Deploying spec-to-proof platform..."
    
    # Create values file for local deployment
    cat > values-local.yaml << EOF
global:
  environment: development
  domain: localhost

images:
  leanFarm:
    repository: localhost:${REGISTRY_PORT}/lean-farm
    tag: "1.0.0"
  nlp:
    repository: localhost:${REGISTRY_PORT}/nlp
    tag: "1.0.0"
  ingest:
    repository: localhost:${REGISTRY_PORT}/ingest
    tag: "1.0.0"
  proof:
    repository: localhost:${REGISTRY_PORT}/proof
    tag: "1.0.0"
  platform:
    repository: localhost:${REGISTRY_PORT}/platform
    tag: "1.0.0"
  ui:
    repository: localhost:${REGISTRY_PORT}/ui
    tag: "1.0.0"

# Disable external dependencies for local deployment
postgresql:
  enabled: false

redis:
  enabled: false

nats:
  enabled: false

# Use external services
externalServices:
  postgresql:
    host: postgresql.${NAMESPACE}.svc.cluster.local
    port: 5432
    database: spec_to_proof
    username: spec_to_proof
    password: spec_to_proof_password
  redis:
    host: redis-master.${NAMESPACE}.svc.cluster.local
    port: 6379
    password: redis_password
  nats:
    url: nats://nats.${NAMESPACE}.svc.cluster.local:4222

# Reduce resource requirements for local deployment
leanFarm:
  replicaCount: 1
  resources:
    requests:
      cpu: 500m
      memory: 1Gi
    limits:
      cpu: 2000m
      memory: 4Gi

nlp:
  replicaCount: 1
  resources:
    requests:
      cpu: 250m
      memory: 512Mi
    limits:
      cpu: 1000m
      memory: 2Gi

ingest:
  replicaCount: 1
  resources:
    requests:
      cpu: 250m
      memory: 512Mi
    limits:
      cpu: 1000m
      memory: 2Gi

proof:
  replicaCount: 1
  resources:
    requests:
      cpu: 500m
      memory: 1Gi
    limits:
      cpu: 2000m
      memory: 4Gi

platform:
  replicaCount: 1
  resources:
    requests:
      cpu: 250m
      memory: 512Mi
    limits:
      cpu: 1000m
      memory: 2Gi

ui:
  replicaCount: 1
  resources:
    requests:
      cpu: 100m
      memory: 256Mi
    limits:
      cpu: 500m
      memory: 1Gi

# Disable monitoring for local deployment
monitoring:
  enabled: false

# Disable security features for local deployment
security:
  podSecurityStandards:
    enabled: false
  networkPolicies:
    enabled: false
  rbac:
    enabled: true
    create: true

# Disable backup for local deployment
backup:
  enabled: false

# Disable resource quotas for local deployment
resourceQuotas:
  enabled: false
EOF
    
    # Deploy the platform
    helm upgrade --install spec-to-proof ./charts/spec-to-proof \
        --namespace "${NAMESPACE}" \
        --values values-local.yaml \
        --wait --timeout=10m
    
    print_success "Platform deployed successfully"
}

# Wait for all pods to be ready
wait_for_pods() {
    print_status "Waiting for all pods to be ready..."
    
    kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=spec-to-proof \
        --namespace "${NAMESPACE}" --timeout=10m
    
    print_success "All pods are ready"
}

# Display access information
show_access_info() {
    print_success "Spec-to-Proof platform is now running!"
    echo
    echo "Access URLs:"
    echo "  UI:                    http://localhost:3000"
    echo "  Lean Farm API:         http://localhost:8080"
    echo "  NLP API:               http://localhost:50051"
    echo "  Ingest API:            http://localhost:8081"
    echo "  Proof API:             http://localhost:8082"
    echo "  Platform API:          http://localhost:8083"
    echo "  Grafana:               http://localhost:3001 (admin/admin)"
    echo "  Prometheus:            http://localhost:9091"
    echo
    echo "Kubernetes commands:"
    echo "  View pods:             kubectl get pods -n ${NAMESPACE}"
    echo "  View services:         kubectl get svc -n ${NAMESPACE}"
    echo "  View logs:             kubectl logs -f deployment/lean-farm -n ${NAMESPACE}"
    echo "  Port forward:          kubectl port-forward svc/ui 3000:3000 -n ${NAMESPACE}"
    echo
    echo "To stop the cluster:"
    echo "  kind delete cluster --name ${CLUSTER_NAME}"
    echo
    echo "To clean up:"
    echo "  docker stop ${REGISTRY_NAME}"
    echo "  docker rm ${REGISTRY_NAME}"
}

# Main execution
main() {
    print_status "Starting Spec-to-Proof platform deployment..."
    
    check_prerequisites
    create_cluster_config
    create_registry
    create_cluster
    setup_helm
    create_namespace
    install_dependencies
    build_images
    deploy_platform
    wait_for_pods
    show_access_info
    
    print_success "Deployment completed successfully!"
}

# Run main function
main "$@" 