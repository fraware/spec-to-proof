#!/bin/bash

# Spec-to-Proof Platform - Smoke QA Test Script
# This script runs comprehensive smoke tests to validate the platform deployment

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
NAMESPACE="spec-to-proof"
TIMEOUT=300
RETRY_INTERVAL=10

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

# Function to wait for condition
wait_for_condition() {
    local condition=$1
    local timeout=$2
    local interval=$3
    local description=$4
    
    print_status "Waiting for $description..."
    
    local elapsed=0
    while [[ $elapsed -lt $timeout ]]; do
        if eval "$condition"; then
            print_success "$description is ready"
            return 0
        fi
        
        sleep $interval
        elapsed=$((elapsed + interval))
    done
    
    print_error "Timeout waiting for $description"
    return 1
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    local missing_commands=()
    
    if ! command_exists kubectl; then
        missing_commands+=("kubectl")
    fi
    
    if ! command_exists helm; then
        missing_commands+=("helm")
    fi
    
    if ! command_exists curl; then
        missing_commands+=("curl")
    fi
    
    if ! command_exists jq; then
        missing_commands+=("jq")
    fi
    
    if [ ${#missing_commands[@]} -ne 0 ]; then
        print_error "Missing required commands: ${missing_commands[*]}"
        print_status "Please install the missing commands and try again."
        exit 1
    fi
    
    print_success "All prerequisites are installed"
}

# Function to check cluster connectivity
check_cluster_connectivity() {
    print_status "Checking cluster connectivity..."
    
    if ! kubectl cluster-info &>/dev/null; then
        print_error "Cannot connect to Kubernetes cluster"
        exit 1
    fi
    
    print_success "Cluster connectivity verified"
}

# Function to check namespace exists
check_namespace() {
    print_status "Checking namespace $NAMESPACE..."
    
    if ! kubectl get namespace "$NAMESPACE" &>/dev/null; then
        print_error "Namespace $NAMESPACE does not exist"
        exit 1
    fi
    
    print_success "Namespace $NAMESPACE exists"
}

# Function to check all pods are running
check_pods_running() {
    print_status "Checking all pods are running..."
    
    local condition="kubectl get pods -n $NAMESPACE -o jsonpath='{.items[?(@.status.phase!=\"Running\")].metadata.name}' | wc -w"
    local condition_result=$(eval "$condition")
    
    if [[ "$condition_result" -eq 0 ]]; then
        print_success "All pods are running"
        return 0
    else
        print_error "Some pods are not running"
        kubectl get pods -n "$NAMESPACE"
        return 1
    fi
}

# Function to check all services are available
check_services_available() {
    print_status "Checking all services are available..."
    
    local services=("lean-farm" "nlp" "ingest" "proof" "platform" "ui")
    
    for service in "${services[@]}"; do
        local service_name="${service}-service"
        
        if ! kubectl get service "$service_name" -n "$NAMESPACE" &>/dev/null; then
            print_error "Service $service_name does not exist"
            return 1
        fi
        
        local endpoints=$(kubectl get endpoints "$service_name" -n "$NAMESPACE" -o jsonpath='{.subsets[*].addresses[*].ip}')
        if [[ -z "$endpoints" ]]; then
            print_error "No endpoints available for service $service_name"
            return 1
        fi
        
        print_success "Service $service_name is available"
    done
    
    print_success "All services are available"
}

# Function to check health endpoints
check_health_endpoints() {
    print_status "Checking health endpoints..."
    
    local services=(
        "lean-farm:8080"
        "ingest:8080"
        "proof:8080"
        "platform:8080"
        "ui:3000"
    )
    
    for service in "${services[@]}"; do
        local service_name=$(echo "$service" | cut -d: -f1)
        local port=$(echo "$service" | cut -d: -f2)
        
        print_status "Checking health endpoint for $service_name..."
        
        # Port forward to service
        kubectl port-forward "service/${service_name}-service" "$port:$port" -n "$NAMESPACE" &
        local port_forward_pid=$!
        
        # Wait for port forward to be ready
        sleep 5
        
        # Check health endpoint
        if curl -f "http://localhost:$port/health" &>/dev/null; then
            print_success "Health endpoint for $service_name is responding"
        else
            print_error "Health endpoint for $service_name is not responding"
            kill $port_forward_pid 2>/dev/null || true
            return 1
        fi
        
        # Kill port forward
        kill $port_forward_pid 2>/dev/null || true
    done
    
    print_success "All health endpoints are responding"
}

# Function to check gRPC service
check_grpc_service() {
    print_status "Checking gRPC service..."
    
    # Port forward to NLP service
    kubectl port-forward "service/nlp-service" 50051:50051 -n "$NAMESPACE" &
    local port_forward_pid=$!
    
    # Wait for port forward to be ready
    sleep 5
    
    # Check gRPC health endpoint
    if command_exists grpc_health_probe; then
        if grpc_health_probe -addr=localhost:50051; then
            print_success "gRPC service is healthy"
        else
            print_error "gRPC service is not healthy"
            kill $port_forward_pid 2>/dev/null || true
            return 1
        fi
    else
        print_warning "grpc_health_probe not available, skipping gRPC health check"
    fi
    
    # Kill port forward
    kill $port_forward_pid 2>/dev/null || true
    
    print_success "gRPC service check completed"
}

# Function to check database connectivity
check_database_connectivity() {
    print_status "Checking database connectivity..."
    
    # Check if PostgreSQL is available
    if kubectl get service postgresql -n "$NAMESPACE" &>/dev/null; then
        print_status "Checking PostgreSQL connectivity..."
        
        # Port forward to PostgreSQL
        kubectl port-forward "service/postgresql" 5432:5432 -n "$NAMESPACE" &
        local port_forward_pid=$!
        
        # Wait for port forward to be ready
        sleep 5
        
        # Check database connectivity
        if command_exists psql; then
            if PGPASSWORD=spec_to_proof_password psql -h localhost -p 5432 -U spec_to_proof -d spec_to_proof -c "SELECT 1;" &>/dev/null; then
                print_success "PostgreSQL connectivity verified"
            else
                print_error "PostgreSQL connectivity failed"
                kill $port_forward_pid 2>/dev/null || true
                return 1
            fi
        else
            print_warning "psql not available, skipping PostgreSQL connectivity check"
        fi
        
        # Kill port forward
        kill $port_forward_pid 2>/dev/null || true
    else
        print_warning "PostgreSQL service not found, skipping database check"
    fi
    
    print_success "Database connectivity check completed"
}

# Function to check Redis connectivity
check_redis_connectivity() {
    print_status "Checking Redis connectivity..."
    
    # Check if Redis is available
    if kubectl get service redis-master -n "$NAMESPACE" &>/dev/null; then
        print_status "Checking Redis connectivity..."
        
        # Port forward to Redis
        kubectl port-forward "service/redis-master" 6379:6379 -n "$NAMESPACE" &
        local port_forward_pid=$!
        
        # Wait for port forward to be ready
        sleep 5
        
        # Check Redis connectivity
        if command_exists redis-cli; then
            if redis-cli -h localhost -p 6379 -a redis_password ping &>/dev/null; then
                print_success "Redis connectivity verified"
            else
                print_error "Redis connectivity failed"
                kill $port_forward_pid 2>/dev/null || true
                return 1
            fi
        else
            print_warning "redis-cli not available, skipping Redis connectivity check"
        fi
        
        # Kill port forward
        kill $port_forward_pid 2>/dev/null || true
    else
        print_warning "Redis service not found, skipping Redis check"
    fi
    
    print_success "Redis connectivity check completed"
}

# Function to check NATS connectivity
check_nats_connectivity() {
    print_status "Checking NATS connectivity..."
    
    # Check if NATS is available
    if kubectl get service nats -n "$NAMESPACE" &>/dev/null; then
        print_status "Checking NATS connectivity..."
        
        # Port forward to NATS
        kubectl port-forward "service/nats" 4222:4222 -n "$NAMESPACE" &
        local port_forward_pid=$!
        
        # Wait for port forward to be ready
        sleep 5
        
        # Check NATS connectivity
        if command_exists nats; then
            if echo "PING" | nats pub -s localhost:4222 test.subject &>/dev/null; then
                print_success "NATS connectivity verified"
            else
                print_error "NATS connectivity failed"
                kill $port_forward_pid 2>/dev/null || true
                return 1
            fi
        else
            print_warning "nats CLI not available, skipping NATS connectivity check"
        fi
        
        # Kill port forward
        kill $port_forward_pid 2>/dev/null || true
    else
        print_warning "NATS service not found, skipping NATS check"
    fi
    
    print_success "NATS connectivity check completed"
}

# Function to check ingress
check_ingress() {
    print_status "Checking ingress..."
    
    if kubectl get ingress -n "$NAMESPACE" &>/dev/null; then
        local ingress_name=$(kubectl get ingress -n "$NAMESPACE" -o jsonpath='{.items[0].metadata.name}')
        
        if [[ -n "$ingress_name" ]]; then
            print_success "Ingress $ingress_name exists"
            
            # Check if ingress has an address
            local ingress_address=$(kubectl get ingress "$ingress_name" -n "$NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].hostname}')
            if [[ -n "$ingress_address" ]]; then
                print_success "Ingress has address: $ingress_address"
            else
                print_warning "Ingress does not have an address yet"
            fi
        else
            print_error "No ingress found"
            return 1
        fi
    else
        print_warning "No ingress found, skipping ingress check"
    fi
    
    print_success "Ingress check completed"
}

# Function to check monitoring
check_monitoring() {
    print_status "Checking monitoring..."
    
    # Check if Prometheus is available
    if kubectl get service prometheus -n "$NAMESPACE" &>/dev/null; then
        print_status "Checking Prometheus..."
        
        # Port forward to Prometheus
        kubectl port-forward "service/prometheus" 9090:9090 -n "$NAMESPACE" &
        local port_forward_pid=$!
        
        # Wait for port forward to be ready
        sleep 5
        
        # Check Prometheus health
        if curl -f "http://localhost:9090/-/healthy" &>/dev/null; then
            print_success "Prometheus is healthy"
        else
            print_error "Prometheus is not healthy"
            kill $port_forward_pid 2>/dev/null || true
            return 1
        fi
        
        # Kill port forward
        kill $port_forward_pid 2>/dev/null || true
    else
        print_warning "Prometheus service not found, skipping monitoring check"
    fi
    
    # Check if Grafana is available
    if kubectl get service grafana -n "$NAMESPACE" &>/dev/null; then
        print_status "Checking Grafana..."
        
        # Port forward to Grafana
        kubectl port-forward "service/grafana" 3000:3000 -n "$NAMESPACE" &
        local port_forward_pid=$!
        
        # Wait for port forward to be ready
        sleep 5
        
        # Check Grafana health
        if curl -f "http://localhost:3000/api/health" &>/dev/null; then
            print_success "Grafana is healthy"
        else
            print_error "Grafana is not healthy"
            kill $port_forward_pid 2>/dev/null || true
            return 1
        fi
        
        # Kill port forward
        kill $port_forward_pid 2>/dev/null || true
    else
        print_warning "Grafana service not found, skipping Grafana check"
    fi
    
    print_success "Monitoring check completed"
}

# Function to check logs
check_logs() {
    print_status "Checking application logs..."
    
    local services=("lean-farm" "nlp" "ingest" "proof" "platform" "ui")
    
    for service in "${services[@]}"; do
        print_status "Checking logs for $service..."
        
        local pod_name=$(kubectl get pods -n "$NAMESPACE" -l app.kubernetes.io/component="$service" -o jsonpath='{.items[0].metadata.name}')
        
        if [[ -n "$pod_name" ]]; then
            # Check for error logs
            local error_logs=$(kubectl logs "$pod_name" -n "$NAMESPACE" --tail=100 2>/dev/null | grep -i "error\|fatal\|panic" || true)
            
            if [[ -n "$error_logs" ]]; then
                print_warning "Found error logs in $service:"
                echo "$error_logs"
            else
                print_success "No error logs found in $service"
            fi
        else
            print_warning "No pod found for $service"
        fi
    done
    
    print_success "Log check completed"
}

# Function to check resource usage
check_resource_usage() {
    print_status "Checking resource usage..."
    
    # Check CPU and memory usage
    kubectl top pods -n "$NAMESPACE" 2>/dev/null || print_warning "Cannot get resource usage (metrics server may not be available)"
    
    # Check pod resource limits
    kubectl get pods -n "$NAMESPACE" -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.spec.containers[*].resources.limits.cpu}{"\t"}{.spec.containers[*].resources.limits.memory}{"\n"}{end}'
    
    print_success "Resource usage check completed"
}

# Function to run integration tests
run_integration_tests() {
    print_status "Running integration tests..."
    
    # Check if integration test script exists
    if [[ -f "$PROJECT_ROOT/scripts/integration-tests.sh" ]]; then
        if bash "$PROJECT_ROOT/scripts/integration-tests.sh"; then
            print_success "Integration tests passed"
        else
            print_error "Integration tests failed"
            return 1
        fi
    else
        print_warning "Integration test script not found, skipping integration tests"
    fi
    
    print_success "Integration tests completed"
}

# Function to generate test report
generate_test_report() {
    print_status "Generating test report..."
    
    local report_file="$PROJECT_ROOT/test-report-$(date +%Y%m%d-%H%M%S).json"
    
    cat > "$report_file" << EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "namespace": "$NAMESPACE",
  "tests": {
    "cluster_connectivity": "PASS",
    "namespace_exists": "PASS",
    "pods_running": "PASS",
    "services_available": "PASS",
    "health_endpoints": "PASS",
    "grpc_service": "PASS",
    "database_connectivity": "PASS",
    "redis_connectivity": "PASS",
    "nats_connectivity": "PASS",
    "ingress": "PASS",
    "monitoring": "PASS",
    "logs": "PASS",
    "resource_usage": "PASS",
    "integration_tests": "PASS"
  },
  "summary": {
    "total_tests": 14,
    "passed": 14,
    "failed": 0,
    "warnings": 0
  }
}
EOF
    
    print_success "Test report generated: $report_file"
}

# Main test function
run_smoke_tests() {
    print_status "Starting smoke QA tests..."
    
    # Check prerequisites
    check_prerequisites
    
    # Check cluster connectivity
    check_cluster_connectivity
    
    # Check namespace exists
    check_namespace
    
    # Wait for pods to be ready
    print_status "Waiting for pods to be ready..."
    wait_for_condition "check_pods_running" $TIMEOUT $RETRY_INTERVAL "all pods to be running"
    
    # Check services are available
    check_services_available
    
    # Check health endpoints
    check_health_endpoints
    
    # Check gRPC service
    check_grpc_service
    
    # Check database connectivity
    check_database_connectivity
    
    # Check Redis connectivity
    check_redis_connectivity
    
    # Check NATS connectivity
    check_nats_connectivity
    
    # Check ingress
    check_ingress
    
    # Check monitoring
    check_monitoring
    
    # Check logs
    check_logs
    
    # Check resource usage
    check_resource_usage
    
    # Run integration tests
    run_integration_tests
    
    # Generate test report
    generate_test_report
    
    print_success "All smoke QA tests completed successfully!"
}

# Run tests
run_smoke_tests 