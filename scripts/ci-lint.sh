#!/bin/bash

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔍 Running Spec-to-Proof CI Lint Checks..."

# Function to print status
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Check if Bazel is available
if ! command -v bazel &> /dev/null; then
    print_error "Bazel is not installed. Please install Bazel first."
    exit 1
fi

# 1. Bazel build check
echo "📦 Running Bazel build check..."
if bazel build //...; then
    print_status "Bazel build successful"
else
    print_error "Bazel build failed"
    exit 1
fi

# 2. Bazel test check
echo "🧪 Running Bazel tests..."
if bazel test //...; then
    print_status "Bazel tests passed"
else
    print_error "Bazel tests failed"
    exit 1
fi

# 3. TypeScript/JavaScript linting
echo "🔧 Running TypeScript/JavaScript linting..."
if command -v npx &> /dev/null; then
    # Check for TypeScript files
    if find . -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.jsx" | grep -q .; then
        # Prettier check
        if npx prettier --check "**/*.{ts,tsx,js,jsx,json,md}"; then
            print_status "Prettier formatting check passed"
        else
            print_error "Prettier formatting check failed"
            exit 1
        fi

        # ESLint check
        if npx eslint "**/*.{ts,tsx,js,jsx}"; then
            print_status "ESLint check passed"
        else
            print_error "ESLint check failed"
            exit 1
        fi
    else
        print_warning "No TypeScript/JavaScript files found"
    fi
else
    print_warning "npx not available, skipping TypeScript/JavaScript linting"
fi

# 4. Rust linting with Clippy
echo "🦀 Running Rust Clippy checks..."
if command -v cargo &> /dev/null; then
    # Find all Cargo.toml files
    find . -name "Cargo.toml" -exec dirname {} \; | while read -r dir; do
        echo "Checking Rust crate in: $dir"
        if (cd "$dir" && cargo clippy -- -D warnings); then
            print_status "Clippy passed for $dir"
        else
            print_error "Clippy failed for $dir"
            exit 1
        fi
    done
else
    print_warning "cargo not available, skipping Rust linting"
fi

# 5. Lean linting
echo "📐 Running Lean linting..."
if command -v lean &> /dev/null; then
    # Find all .lean files
    if find . -name "*.lean" | grep -q .; then
        # Check if leanlint is available
        if command -v leanlint &> /dev/null; then
            if leanlint **/*.lean; then
                print_status "Lean linting passed"
            else
                print_error "Lean linting failed"
                exit 1
            fi
        else
            print_warning "leanlint not available, skipping Lean linting"
        fi
    else
        print_warning "No Lean files found"
    fi
else
    print_warning "lean not available, skipping Lean linting"
fi

# 6. Terraform linting
echo "🏗️  Running Terraform linting..."
if command -v terraform &> /dev/null; then
    # Find all .tf files
    if find . -name "*.tf" | grep -q .; then
        find . -name "*.tf" -exec dirname {} \; | sort | uniq | while read -r dir; do
            echo "Checking Terraform in: $dir"
            if (cd "$dir" && terraform fmt -check); then
                print_status "Terraform formatting check passed for $dir"
            else
                print_error "Terraform formatting check failed for $dir"
                exit 1
            fi
            
            if (cd "$dir" && terraform validate); then
                print_status "Terraform validation passed for $dir"
            else
                print_error "Terraform validation failed for $dir"
                exit 1
            fi
        done
    else
        print_warning "No Terraform files found"
    fi
else
    print_warning "terraform not available, skipping Terraform linting"
fi

# 7. Protobuf linting
echo "📋 Running Protobuf linting..."
if command -v buf &> /dev/null; then
    if [ -f "buf.yaml" ] || [ -f "buf.work.yaml" ]; then
        if buf lint; then
            print_status "Protobuf linting passed"
        else
            print_error "Protobuf linting failed"
            exit 1
        fi
        
        if buf breaking --against '.git#branch=main'; then
            print_status "Protobuf breaking change check passed"
        else
            print_error "Protobuf breaking change check failed"
            exit 1
        fi
    else
        print_warning "No buf.yaml or buf.work.yaml found"
    fi
else
    print_warning "buf not available, skipping Protobuf linting"
fi

# 8. Security scanning
echo "🔒 Running security scans..."
if command -v cargo-audit &> /dev/null; then
    if cargo audit; then
        print_status "Cargo audit passed"
    else
        print_error "Cargo audit failed"
        exit 1
    fi
else
    print_warning "cargo-audit not available, skipping security scan"
fi

# 9. License compliance
echo "📄 Checking license compliance..."
if command -v license-checker &> /dev/null; then
    if license-checker --summary; then
        print_status "License compliance check passed"
    else
        print_error "License compliance check failed"
        exit 1
    fi
else
    print_warning "license-checker not available, skipping license compliance check"
fi

echo ""
echo -e "${GREEN}🎉 All lint checks passed!${NC}"
echo "Spec-to-Proof codebase is clean and ready for production." 