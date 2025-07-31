#!/bin/bash

# Spec-to-Proof Platform - SBOM Generation Script
# This script generates Software Bill of Materials (SBOM) for all components
# using Syft and attaches them to releases with semantic versioning

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
VERSION=""
DRY_RUN=false
SKIP_IMAGES=false
SKIP_PROJECT=false
OUTPUT_FORMATS=("json" "spdx-json" "cyclonedx-json")

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
Usage: $0 [OPTIONS] VERSION

Generate SBOM for the spec-to-proof platform.

OPTIONS:
    -h, --help          Show this help message
    -d, --dry-run       Perform a dry run without making changes
    --skip-images        Skip generating SBOM for Docker images
    --skip-project       Skip generating SBOM for the project
    --formats FORMATS    Comma-separated list of output formats (default: json,spdx-json,cyclonedx-json)
    -v, --version       Show version information

VERSION:
    Semantic version (e.g., 1.0.0, 2.0.0)

EXAMPLES:
    $0 1.0.0                    # Generate SBOM for version 1.0.0
    $0 -d 2.0.0                # Dry run SBOM generation for version 2.0.0
    $0 --skip-images 1.0.1     # Generate only project SBOM

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

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    local missing_commands=()
    
    if ! command -v syft &> /dev/null; then
        missing_commands+=("syft")
    fi
    
    if ! command -v docker &> /dev/null; then
        missing_commands+=("docker")
    fi
    
    if ! command -v jq &> /dev/null; then
        missing_commands+=("jq")
    fi
    
    if [ ${#missing_commands[@]} -ne 0 ]; then
        print_error "Missing required commands: ${missing_commands[*]}"
        print_status "Please install the missing commands and try again."
        print_status "Syft installation: https://github.com/anchore/syft#installation"
        exit 1
    fi
    
    print_success "All prerequisites are installed"
}

# Function to create SBOM directory
create_sbom_directory() {
    local sbom_dir="${PROJECT_ROOT}/sbom/${VERSION}"
    
    print_status "Creating SBOM directory: ${sbom_dir}"
    
    if [ "$DRY_RUN" = false ]; then
        mkdir -p "${sbom_dir}"
    fi
    
    echo "${sbom_dir}"
}

# Function to generate SBOM for Docker image
generate_image_sbom() {
    local component=$1
    local image_name="${REGISTRY}/${component}:${VERSION}"
    local sbom_dir=$2
    
    print_status "Generating SBOM for ${component} image: ${image_name}"
    
    if [ "$DRY_RUN" = true ]; then
        print_status "DRY RUN: Would generate SBOM for ${image_name}"
        return
    fi
    
    # Check if image exists locally
    if ! docker image inspect "${image_name}" &> /dev/null; then
        print_warning "Image ${image_name} not found locally, attempting to pull..."
        if ! docker pull "${image_name}" &> /dev/null; then
            print_error "Failed to pull image ${image_name}"
            return 1
        fi
    fi
    
    # Generate SBOM in multiple formats
    for format in "${OUTPUT_FORMATS[@]}"; do
        local output_file="${sbom_dir}/${component}-${VERSION}.${format}"
        print_status "Generating ${format} SBOM for ${component}..."
        
        if syft "${image_name}" -o "${format}" > "${output_file}" 2>/dev/null; then
            print_success "Generated ${format} SBOM for ${component}"
        else
            print_error "Failed to generate ${format} SBOM for ${component}"
            return 1
        fi
    done
}

# Function to generate SBOM for project
generate_project_sbom() {
    local sbom_dir=$1
    
    print_status "Generating SBOM for project directory"
    
    if [ "$DRY_RUN" = true ]; then
        print_status "DRY RUN: Would generate project SBOM"
        return
    fi
    
    # Generate SBOM in multiple formats
    for format in "${OUTPUT_FORMATS[@]}"; do
        local output_file="${sbom_dir}/spec-to-proof-${VERSION}.${format}"
        print_status "Generating ${format} SBOM for project..."
        
        if syft dir:"${PROJECT_ROOT}" -o "${format}" > "${output_file}" 2>/dev/null; then
            print_success "Generated ${format} SBOM for project"
        else
            print_error "Failed to generate ${format} SBOM for project"
            return 1
        fi
    done
}

# Function to create SBOM manifest
create_sbom_manifest() {
    local sbom_dir=$1
    
    print_status "Creating SBOM manifest..."
    
    if [ "$DRY_RUN" = true ]; then
        print_status "DRY RUN: Would create SBOM manifest"
        return
    fi
    
    local manifest_file="${sbom_dir}/manifest.json"
    
    cat > "${manifest_file}" << EOF
{
  "version": "${VERSION}",
  "generated_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "generator": "syft",
  "generator_version": "$(syft version | head -n1 | cut -d' ' -f2)",
  "components": [
    "lean-farm",
    "nlp", 
    "ingest",
    "proof",
    "platform",
    "ui"
  ],
  "formats": $(jq -n --arg formats "$(IFS=','; echo "${OUTPUT_FORMATS[*]}")" '$formats | split(",")'),
  "files": []
}
EOF
    
    # Add file information to manifest
    local files_json="[]"
    while IFS= read -r -d '' file; do
        local filename=$(basename "$file")
        local filesize=$(stat -c%s "$file")
        local checksum=$(sha256sum "$file" | cut -d' ' -f1)
        
        files_json=$(echo "$files_json" | jq --arg name "$filename" \
            --arg size "$filesize" \
            --arg sha256 "$checksum" \
            --arg path "$file" \
            '. += [{"name": $name, "size": ($size | tonumber), "sha256": $sha256, "path": $path}]')
    done < <(find "${sbom_dir}" -type f -name "*.json" -print0)
    
    # Update manifest with file information
    jq --argjson files "$files_json" '.files = $files' "${manifest_file}" > "${manifest_file}.tmp" && mv "${manifest_file}.tmp" "${manifest_file}"
    
    print_success "SBOM manifest created: ${manifest_file}"
}

# Function to validate SBOM files
validate_sbom_files() {
    local sbom_dir=$1
    
    print_status "Validating SBOM files..."
    
    local validation_errors=0
    
    # Check if all expected files exist
    local expected_files=(
        "lean-farm-${VERSION}.json"
        "nlp-${VERSION}.json"
        "ingest-${VERSION}.json"
        "proof-${VERSION}.json"
        "platform-${VERSION}.json"
        "ui-${VERSION}.json"
        "spec-to-proof-${VERSION}.json"
        "manifest.json"
    )
    
    for file in "${expected_files[@]}"; do
        if [ ! -f "${sbom_dir}/${file}" ]; then
            print_error "Missing SBOM file: ${file}"
            validation_errors=$((validation_errors + 1))
        fi
    done
    
    # Validate JSON format
    for file in "${sbom_dir}"/*.json; do
        if [ -f "$file" ]; then
            if ! jq empty "$file" &> /dev/null; then
                print_error "Invalid JSON in file: ${file}"
                validation_errors=$((validation_errors + 1))
            fi
        fi
    done
    
    if [ $validation_errors -eq 0 ]; then
        print_success "All SBOM files validated successfully"
    else
        print_error "SBOM validation failed with ${validation_errors} errors"
        return 1
    fi
}

# Function to create SBOM archive
create_sbom_archive() {
    local sbom_dir=$1
    
    print_status "Creating SBOM archive..."
    
    if [ "$DRY_RUN" = true ]; then
        print_status "DRY RUN: Would create SBOM archive"
        return
    fi
    
    local archive_name="spec-to-proof-sbom-${VERSION}.tar.gz"
    local archive_path="${PROJECT_ROOT}/${archive_name}"
    
    cd "${sbom_dir}" && tar -czf "${archive_path}" . && cd "${PROJECT_ROOT}"
    
    print_success "SBOM archive created: ${archive_path}"
    echo "Archive size: $(du -h "${archive_path}" | cut -f1)"
    echo "Archive checksum: $(sha256sum "${archive_path}" | cut -d' ' -f1)"
}

# Function to clean up temporary files
cleanup() {
    print_status "Cleaning up temporary files..."
    
    if [ "$DRY_RUN" = false ]; then
        # Remove temporary files if any
        find "${PROJECT_ROOT}" -name "*.tmp" -delete 2>/dev/null || true
    fi
}

# Main execution
main() {
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
            --skip-images)
                SKIP_IMAGES=true
                shift
                ;;
            --skip-project)
                SKIP_PROJECT=true
                shift
                ;;
            --formats)
                IFS=',' read -ra OUTPUT_FORMATS <<< "$2"
                shift 2
                ;;
            -v|--version)
                echo "SBOM Generator v1.0.0"
                exit 0
                ;;
            -*)
                print_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
            *)
                if [ -z "$VERSION" ]; then
                    VERSION="$1"
                else
                    print_error "Version already specified: $VERSION"
                    exit 1
                fi
                shift
                ;;
        esac
    done
    
    # Validate version
    if [ -z "$VERSION" ]; then
        print_error "Version is required"
        show_usage
        exit 1
    fi
    
    validate_version "$VERSION"
    
    print_status "Starting SBOM generation for version: ${VERSION}"
    
    # Check prerequisites
    check_prerequisites
    
    # Create SBOM directory
    local sbom_dir=$(create_sbom_directory)
    
    # Generate SBOM for Docker images
    if [ "$SKIP_IMAGES" = false ]; then
        local components=("lean-farm" "nlp" "ingest" "proof" "platform" "ui")
        
        for component in "${components[@]}"; do
            if ! generate_image_sbom "$component" "$sbom_dir"; then
                print_error "Failed to generate SBOM for ${component}"
                exit 1
            fi
        done
    fi
    
    # Generate SBOM for project
    if [ "$SKIP_PROJECT" = false ]; then
        if ! generate_project_sbom "$sbom_dir"; then
            print_error "Failed to generate project SBOM"
            exit 1
        fi
    fi
    
    # Create SBOM manifest
    create_sbom_manifest "$sbom_dir"
    
    # Validate SBOM files
    validate_sbom_files "$sbom_dir"
    
    # Create SBOM archive
    create_sbom_archive "$sbom_dir"
    
    # Cleanup
    cleanup
    
    print_success "SBOM generation completed successfully!"
    print_status "SBOM files location: ${sbom_dir}"
    print_status "SBOM archive: ${PROJECT_ROOT}/spec-to-proof-sbom-${VERSION}.tar.gz"
}

# Trap to ensure cleanup on exit
trap cleanup EXIT

# Run main function
main "$@" 