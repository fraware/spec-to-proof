#!/bin/bash

# Spec-to-Proof Platform - Upgrade Playbook
# This script handles safe upgrades with semantic versioning and SBOM generation

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
REGISTRY="your-registry"
CURRENT_VERSION="1.0.0"
NEW_VERSION=""
DRY_RUN=false
SKIP_SBOM=false
SKIP_TESTS=false
FORCE=false

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

# Function to show usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS] NEW_VERSION

Upgrade the spec-to-proof platform to a new version.

OPTIONS:
    -h, --help          Show this help message
    -d, --dry-run       Perform a dry run without making changes
    --skip-sbom         Skip SBOM generation
    --skip-tests        Skip running tests
    -f, --force         Force upgrade even if checks fail
    -v, --version       Show version information

NEW_VERSION:
    Semantic version (e.g., 1.1.0, 2.0.0)

EXAMPLES:
    $0 1.1.0                    # Upgrade to version 1.1.0
    $0 -d 2.0.0                # Dry run upgrade to version 2.0.0
    $0 --skip-tests 1.0.1      # Upgrade without running tests

EOF
}

# Function to validate semantic version
validate_version() {
    local version=$1
    if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        print_error "Invalid version format: $version"
        print_status "Version must be in semantic versioning format (e.g., 1.0.0)"
        exit 1
    fi
}

# Function to compare versions
compare_versions() {
    local version1=$1
    local version2=$2
    
    IFS='.' read -ra v1 <<< "$version1"
    IFS='.' read -ra v2 <<< "$version2"
    
    for i in {0..2}; do
        if [[ ${v1[$i]} -lt ${v2[$i]} ]]; then
            return 0  # version1 < version2
        elif [[ ${v1[$i]} -gt ${v2[$i]} ]]; then
            return 1  # version1 > version2
        fi
    done
    return 2  # version1 == version2
}

# Function to check if upgrade is valid
validate_upgrade() {
    local current=$1
    local target=$2
    
    if compare_versions "$current" "$target"; then
        if [[ $? -eq 1 ]]; then
            print_error "Cannot downgrade from $current to $target"
            exit 1
        elif [[ $? -eq 2 ]]; then
            print_error "Target version $target is the same as current version $current"
            exit 1
        fi
    fi
    
    print_success "Upgrade from $current to $target is valid"
}

# Function to generate SBOM
generate_sbom() {
    if [[ "$SKIP_SBOM" == "true" ]]; then
        print_warning "Skipping SBOM generation"
        return
    fi
    
    print_status "Generating SBOM..."
    
    # Check if syft is installed
    if ! command -v syft &> /dev/null; then
        print_error "syft is not installed. Please install it first."
        print_status "Installation: https://github.com/anchore/syft#installation"
        exit 1
    fi
    
    # Create SBOM directory
    local sbom_dir="$PROJECT_ROOT/sbom"
    mkdir -p "$sbom_dir"
    
    # Generate SBOM for each component
    local components=("lean-farm" "nlp" "ingest" "proof" "platform" "ui")
    
    for component in "${components[@]}"; do
        print_status "Generating SBOM for $component..."
        
        local dockerfile_path="$PROJECT_ROOT/$component/Dockerfile"
        if [[ -f "$dockerfile_path" ]]; then
            # Build image for SBOM generation
            local image_name="$REGISTRY/$component:$NEW_VERSION"
            docker build -t "$image_name" "$PROJECT_ROOT/$component/"
            
            # Generate SBOM
            syft "$image_name" -o json > "$sbom_dir/${component}-${NEW_VERSION}.json"
            syft "$image_name" -o spdx-json > "$sbom_dir/${component}-${NEW_VERSION}.spdx.json"
            
            print_success "SBOM generated for $component"
        else
            print_warning "Dockerfile not found for $component, skipping SBOM generation"
        fi
    done
    
    # Generate combined SBOM
    print_status "Generating combined SBOM..."
    syft dir:"$PROJECT_ROOT" -o json > "$sbom_dir/spec-to-proof-${NEW_VERSION}.json"
    syft dir:"$PROJECT_ROOT" -o spdx-json > "$sbom_dir/spec-to-proof-${NEW_VERSION}.spdx.json"
    
    print_success "SBOM generation completed"
}

# Function to run tests
run_tests() {
    if [[ "$SKIP_TESTS" == "true" ]]; then
        print_warning "Skipping tests"
        return
    fi
    
    print_status "Running tests..."
    
    # Run unit tests
    print_status "Running unit tests..."
    if ! make test; then
        print_error "Unit tests failed"
        exit 1
    fi
    
    # Run integration tests
    print_status "Running integration tests..."
    if ! make integration-test; then
        print_error "Integration tests failed"
        exit 1
    fi
    
    # Run Helm chart tests
    print_status "Running Helm chart tests..."
    if ! helm test spec-to-proof --namespace spec-to-proof; then
        print_error "Helm chart tests failed"
        exit 1
    fi
    
    print_success "All tests passed"
}

# Function to update version in files
update_version() {
    local new_version=$1
    
    print_status "Updating version to $new_version..."
    
    # Update Chart.yaml
    if [[ -f "$PROJECT_ROOT/charts/spec-to-proof/Chart.yaml" ]]; then
        sed -i "s/version: .*/version: $new_version/" "$PROJECT_ROOT/charts/spec-to-proof/Chart.yaml"
        sed -i "s/appVersion: .*/appVersion: \"$new_version\"/" "$PROJECT_ROOT/charts/spec-to-proof/Chart.yaml"
    fi
    
    # Update values.yaml
    if [[ -f "$PROJECT_ROOT/charts/spec-to-proof/values.yaml" ]]; then
        sed -i "s/tag: .*/tag: \"$new_version\"/" "$PROJECT_ROOT/charts/spec-to-proof/values.yaml"
    fi
    
    # Update docker-compose.yml
    if [[ -f "$PROJECT_ROOT/docker-compose.yml" ]]; then
        sed -i "s/:${CURRENT_VERSION}/:$new_version/g" "$PROJECT_ROOT/docker-compose.yml"
    fi
    
    # Update kind-cluster.sh
    if [[ -f "$PROJECT_ROOT/scripts/kind-cluster.sh" ]]; then
        sed -i "s/tag: \"${CURRENT_VERSION}\"/tag: \"$new_version\"/g" "$PROJECT_ROOT/scripts/kind-cluster.sh"
    fi
    
    print_success "Version updated to $new_version"
}

# Function to build and push images
build_images() {
    if [[ "$DRY_RUN" == "true" ]]; then
        print_warning "Dry run: Skipping image build and push"
        return
    fi
    
    print_status "Building and pushing images..."
    
    local components=("lean-farm" "nlp" "ingest" "proof" "platform" "ui")
    
    for component in "${components[@]}"; do
        print_status "Building $component image..."
        
        local dockerfile_path="$PROJECT_ROOT/$component/Dockerfile"
        if [[ -f "$dockerfile_path" ]]; then
            local image_name="$REGISTRY/$component:$NEW_VERSION"
            
            # Build image
            docker build -t "$image_name" "$PROJECT_ROOT/$component/"
            
            # Push image
            docker push "$image_name"
            
            print_success "Built and pushed $component:$NEW_VERSION"
        else
            print_warning "Dockerfile not found for $component, skipping"
        fi
    done
}

# Function to deploy upgrade
deploy_upgrade() {
    if [[ "$DRY_RUN" == "true" ]]; then
        print_warning "Dry run: Skipping deployment"
        return
    fi
    
    print_status "Deploying upgrade..."
    
    # Check if kubectl is available
    if ! command -v kubectl &> /dev/null; then
        print_error "kubectl is not installed or not in PATH"
        exit 1
    fi
    
    # Check if helm is available
    if ! command -v helm &> /dev/null; then
        print_error "helm is not installed or not in PATH"
        exit 1
    fi
    
    # Update Helm release
    print_status "Updating Helm release..."
    helm upgrade spec-to-proof "$PROJECT_ROOT/charts/spec-to-proof" \
        --namespace spec-to-proof \
        --set global.environment=production \
        --set images.leanFarm.tag="$NEW_VERSION" \
        --set images.nlp.tag="$NEW_VERSION" \
        --set images.ingest.tag="$NEW_VERSION" \
        --set images.proof.tag="$NEW_VERSION" \
        --set images.platform.tag="$NEW_VERSION" \
        --set images.ui.tag="$NEW_VERSION" \
        --wait --timeout=10m
    
    print_success "Upgrade deployed successfully"
}

# Function to rollback
rollback() {
    print_status "Rolling back to version $CURRENT_VERSION..."
    
    helm rollback spec-to-proof --namespace spec-to-proof
    
    print_success "Rollback completed"
}

# Function to create backup
create_backup() {
    print_status "Creating backup..."
    
    # Create backup directory
    local backup_dir="$PROJECT_ROOT/backups/$(date +%Y%m%d-%H%M%S)"
    mkdir -p "$backup_dir"
    
    # Backup current configuration
    kubectl get all -n spec-to-proof -o yaml > "$backup_dir/k8s-resources.yaml"
    kubectl get configmaps -n spec-to-proof -o yaml > "$backup_dir/configmaps.yaml"
    kubectl get secrets -n spec-to-proof -o yaml > "$backup_dir/secrets.yaml"
    
    # Backup Helm values
    helm get values spec-to-proof -n spec-to-proof > "$backup_dir/helm-values.yaml"
    
    print_success "Backup created in $backup_dir"
}

# Function to verify deployment
verify_deployment() {
    print_status "Verifying deployment..."
    
    # Wait for pods to be ready
    kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=spec-to-proof \
        --namespace spec-to-proof --timeout=5m
    
    # Check pod status
    local failed_pods=$(kubectl get pods -n spec-to-proof -o jsonpath='{.items[?(@.status.phase=="Failed")].metadata.name}')
    if [[ -n "$failed_pods" ]]; then
        print_error "Some pods failed to start: $failed_pods"
        return 1
    fi
    
    # Check service endpoints
    local services=("lean-farm" "nlp" "ingest" "proof" "platform" "ui")
    for service in "${services[@]}"; do
        local endpoints=$(kubectl get endpoints "$service-service" -n spec-to-proof -o jsonpath='{.subsets[*].addresses[*].ip}')
        if [[ -z "$endpoints" ]]; then
            print_error "No endpoints available for $service"
            return 1
        fi
    done
    
    print_success "Deployment verification completed"
}

# Function to update documentation
update_documentation() {
    print_status "Updating documentation..."
    
    # Update README.md
    if [[ -f "$PROJECT_ROOT/README.md" ]]; then
        sed -i "s/version: .*/version: $NEW_VERSION/" "$PROJECT_ROOT/README.md"
    fi
    
    # Update CHANGELOG.md
    if [[ -f "$PROJECT_ROOT/CHANGELOG.md" ]]; then
        # Add new version entry to changelog
        local changelog_entry="## [$NEW_VERSION] - $(date +%Y-%m-%d)"
        sed -i "1i $changelog_entry" "$PROJECT_ROOT/CHANGELOG.md"
    fi
    
    print_success "Documentation updated"
}

# Function to create git tag
create_git_tag() {
    if [[ "$DRY_RUN" == "true" ]]; then
        print_warning "Dry run: Skipping git tag creation"
        return
    fi
    
    print_status "Creating git tag..."
    
    # Check if we're in a git repository
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        print_warning "Not in a git repository, skipping tag creation"
        return
    fi
    
    # Create and push tag
    git tag -a "v$NEW_VERSION" -m "Release version $NEW_VERSION"
    git push origin "v$NEW_VERSION"
    
    print_success "Git tag v$NEW_VERSION created and pushed"
}

# Main upgrade function
perform_upgrade() {
    local new_version=$1
    
    print_status "Starting upgrade to version $new_version..."
    
    # Validate upgrade
    validate_upgrade "$CURRENT_VERSION" "$new_version"
    
    # Create backup
    create_backup
    
    # Generate SBOM
    generate_sbom
    
    # Run tests
    run_tests
    
    # Update version in files
    update_version "$new_version"
    
    # Build and push images
    build_images
    
    # Deploy upgrade
    deploy_upgrade
    
    # Verify deployment
    if ! verify_deployment; then
        print_error "Deployment verification failed"
        if [[ "$FORCE" != "true" ]]; then
            print_status "Rolling back..."
            rollback
            exit 1
        fi
    fi
    
    # Update documentation
    update_documentation
    
    # Create git tag
    create_git_tag
    
    print_success "Upgrade to version $new_version completed successfully!"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        --skip-sbom)
            SKIP_SBOM=true
            shift
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        -v|--version)
            echo "Spec-to-Proof Upgrade Playbook v1.0.0"
            exit 0
            ;;
        -*)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
        *)
            if [[ -z "$NEW_VERSION" ]]; then
                NEW_VERSION=$1
            else
                print_error "Multiple versions specified"
                exit 1
            fi
            shift
            ;;
    esac
done

# Check if version is provided
if [[ -z "$NEW_VERSION" ]]; then
    print_error "No version specified"
    show_usage
    exit 1
fi

# Validate version format
validate_version "$NEW_VERSION"

# Perform upgrade
perform_upgrade "$NEW_VERSION" 