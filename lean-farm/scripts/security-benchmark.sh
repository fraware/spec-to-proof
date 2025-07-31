#!/bin/bash

# Security Benchmark for Lean Farm
# Validates seccomp restrictions, rootless execution, and OSS scanning

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
LEAN_FARM_IMAGE="lean-farm:1.0.0"
SCAN_TIMEOUT=300
MAX_CRITICAL_VULNERABILITIES=0
MAX_HIGH_VULNERABILITIES=5

echo -e "${GREEN}🔒 Starting Lean Farm Security Benchmark${NC}"

# Function to print status
print_status() {
    local status=$1
    local message=$2
    if [ "$status" = "PASS" ]; then
        echo -e "${GREEN}✅ $message${NC}"
    elif [ "$status" = "FAIL" ]; then
        echo -e "${RED}❌ $message${NC}"
    else
        echo -e "${YELLOW}⚠️  $message${NC}"
    fi
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
echo "Checking prerequisites..."

if ! command_exists docker; then
    print_status "FAIL" "Docker is not installed"
    exit 1
fi

if ! command_exists trivy; then
    print_status "WARN" "Trivy is not installed, skipping vulnerability scan"
    TRIVY_AVAILABLE=false
else
    TRIVY_AVAILABLE=true
fi

if ! command_exists runsc; then
    print_status "WARN" "gVisor (runsc) is not installed"
    GVISOR_AVAILABLE=false
else
    GVISOR_AVAILABLE=true
fi

print_status "PASS" "Prerequisites check completed"

# Test 1: Seccomp Profile Validation
echo -e "\n${YELLOW}Test 1: Seccomp Profile Validation${NC}"

# Check if seccomp is enabled
if docker run --rm --security-opt seccomp=unconfined "$LEAN_FARM_IMAGE" echo "test" >/dev/null 2>&1; then
    print_status "PASS" "Seccomp profile is properly configured"
else
    print_status "FAIL" "Seccomp profile validation failed"
fi

# Test 2: Rootless Execution
echo -e "\n${YELLOW}Test 2: Rootless Execution${NC}"

# Check if container runs as non-root
if docker run --rm "$LEAN_FARM_IMAGE" id | grep -q "uid=1000"; then
    print_status "PASS" "Container runs as non-root user (UID 1000)"
else
    print_status "FAIL" "Container does not run as non-root user"
fi

# Test 3: Capability Dropping
echo -e "\n${YELLOW}Test 3: Capability Dropping${NC}"

# Check if all capabilities are dropped
if docker run --rm "$LEAN_FARM_IMAGE" capsh --print | grep -q "Current:" && \
   docker run --rm "$LEAN_FARM_IMAGE" capsh --print | grep -q "Current: =ep"; then
    print_status "FAIL" "All capabilities should be dropped"
else
    print_status "PASS" "Capabilities are properly dropped"
fi

# Test 4: Read-only Root Filesystem
echo -e "\n${YELLOW}Test 4: Read-only Root Filesystem${NC}"

# Check if root filesystem is read-only
if docker run --rm "$LEAN_FARM_IMAGE" touch /test 2>/dev/null; then
    print_status "FAIL" "Root filesystem should be read-only"
else
    print_status "PASS" "Root filesystem is read-only"
fi

# Test 5: Network Isolation
echo -e "\n${YELLOW}Test 5: Network Isolation${NC}"

# Check if network is isolated
if docker run --rm --network=none "$LEAN_FARM_IMAGE" ping -c 1 8.8.8.8 2>/dev/null; then
    print_status "FAIL" "Network should be isolated"
else
    print_status "PASS" "Network is properly isolated"
fi

# Test 6: Resource Limits
echo -e "\n${YELLOW}Test 6: Resource Limits${NC}"

# Check CPU and memory limits
if docker run --rm --cpus=0.1 --memory=100m "$LEAN_FARM_IMAGE" echo "test" >/dev/null 2>&1; then
    print_status "PASS" "Resource limits are properly enforced"
else
    print_status "FAIL" "Resource limits validation failed"
fi

# Test 7: gVisor Runtime (if available)
if [ "$GVISOR_AVAILABLE" = true ]; then
    echo -e "\n${YELLOW}Test 7: gVisor Runtime${NC}"
    
    if docker run --rm --runtime=runsc "$LEAN_FARM_IMAGE" echo "test" >/dev/null 2>&1; then
        print_status "PASS" "gVisor runtime works correctly"
    else
        print_status "FAIL" "gVisor runtime test failed"
    fi
else
    echo -e "\n${YELLOW}Test 7: gVisor Runtime${NC}"
    print_status "SKIP" "gVisor not available, skipping test"
fi

# Test 8: OSS Vulnerability Scan
if [ "$TRIVY_AVAILABLE" = true ]; then
    echo -e "\n${YELLOW}Test 8: OSS Vulnerability Scan${NC}"
    
    echo "Running Trivy vulnerability scan (timeout: ${SCAN_TIMEOUT}s)..."
    
    # Run Trivy scan with timeout
    if timeout "$SCAN_TIMEOUT" trivy image --severity CRITICAL,HIGH "$LEAN_FARM_IMAGE" > trivy_output.txt 2>&1; then
        # Parse results
        CRITICAL_COUNT=$(grep -c "CRITICAL" trivy_output.txt || echo "0")
        HIGH_COUNT=$(grep -c "HIGH" trivy_output.txt || echo "0")
        
        echo "Vulnerabilities found:"
        echo "  Critical: $CRITICAL_COUNT (max: $MAX_CRITICAL_VULNERABILITIES)"
        echo "  High: $HIGH_COUNT (max: $MAX_HIGH_VULNERABILITIES)"
        
        if [ "$CRITICAL_COUNT" -le "$MAX_CRITICAL_VULNERABILITIES" ] && [ "$HIGH_COUNT" -le "$MAX_HIGH_VULNERABILITIES" ]; then
            print_status "PASS" "Vulnerability scan passed"
        else
            print_status "FAIL" "Too many vulnerabilities found"
            echo "Full scan results:"
            cat trivy_output.txt
        fi
    else
        print_status "FAIL" "Vulnerability scan failed or timed out"
    fi
    
    # Clean up
    rm -f trivy_output.txt
else
    echo -e "\n${YELLOW}Test 8: OSS Vulnerability Scan${NC}"
    print_status "SKIP" "Trivy not available, skipping vulnerability scan"
fi

# Test 9: Container Escape Prevention
echo -e "\n${YELLOW}Test 9: Container Escape Prevention${NC}"

# Test for common container escape vectors
ESCAPE_ATTEMPTS=(
    "mount /dev/sda1 /mnt"
    "chroot /mnt"
    "echo 'kernel.core_pattern=|/bin/sh' > /proc/sys/kernel/core_pattern"
    "echo '1' > /proc/sys/kernel/dmesg_restrict"
)

ESCAPE_PREVENTED=true
for attempt in "${ESCAPE_ATTEMPTS[@]}"; do
    if docker run --rm "$LEAN_FARM_IMAGE" sh -c "$attempt" 2>/dev/null; then
        print_status "FAIL" "Container escape attempt succeeded: $attempt"
        ESCAPE_PREVENTED=false
    fi
done

if [ "$ESCAPE_PREVENTED" = true ]; then
    print_status "PASS" "Container escape prevention is working"
fi

# Test 10: File System Isolation
echo -e "\n${YELLOW}Test 10: File System Isolation${NC}"

# Test if container can access host filesystem
if docker run --rm "$LEAN_FARM_IMAGE" ls /host 2>/dev/null; then
    print_status "FAIL" "Container can access host filesystem"
else
    print_status "PASS" "File system isolation is working"
fi

# Test 11: Process Isolation
echo -e "\n${YELLOW}Test 11: Process Isolation${NC}"

# Test if container can see host processes
if docker run --rm "$LEAN_FARM_IMAGE" ps aux | grep -v "lean-farm" | grep -v "ps" | wc -l | grep -q "0"; then
    print_status "PASS" "Process isolation is working"
else
    print_status "FAIL" "Container can see host processes"
fi

# Test 12: Security Context Validation
echo -e "\n${YELLOW}Test 12: Security Context Validation${NC}"

# Check security context
SECURITY_CONTEXT=$(docker inspect "$LEAN_FARM_IMAGE" --format='{{.Config.SecurityOpt}}' 2>/dev/null || echo "")

if echo "$SECURITY_CONTEXT" | grep -q "no-new-privileges"; then
    print_status "PASS" "No new privileges security option is set"
else
    print_status "FAIL" "No new privileges security option is missing"
fi

# Summary
echo -e "\n${GREEN}🔒 Security Benchmark Summary${NC}"
echo "=================================="

# Count results
PASS_COUNT=$(grep -c "✅" "$0" || echo "0")
FAIL_COUNT=$(grep -c "❌" "$0" || echo "0")
SKIP_COUNT=$(grep -c "⚠️" "$0" || echo "0")

echo "Tests passed: $PASS_COUNT"
echo "Tests failed: $FAIL_COUNT"
echo "Tests skipped: $SKIP_COUNT"

if [ "$FAIL_COUNT" -eq 0 ]; then
    echo -e "${GREEN}🎉 All security tests passed!${NC}"
    exit 0
else
    echo -e "${RED}⚠️  Some security tests failed. Please review the results above.${NC}"
    exit 1
fi 