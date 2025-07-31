.PHONY: help dev build test lint clean install deps

# Default target
help:
	@echo "Spec-to-Proof Development Commands"
	@echo "=================================="
	@echo "make dev          - Start development environment"
	@echo "make build        - Build all targets"
	@echo "make test         - Run all tests"
	@echo "make lint         - Run linting checks"
	@echo "make clean        - Clean build artifacts"
	@echo "make install      - Install dependencies"
	@echo "make deps         - Update dependencies"
	@echo "make docker       - Build Docker images"
	@echo "make deploy       - Deploy to production"

# Development environment
dev:
	@echo "🚀 Starting Spec-to-Proof development environment..."
	bazel run //platform/api_server &
	bazel run //ingest/jira_connector &
	bazel run //nlp/invariant_extractor &
	bazel run //proof/lean_compiler &
	@echo "✅ Development environment started"

# Build all targets
build:
	@echo "🔨 Building all targets..."
	bazel build //...
	@echo "✅ Build completed"

# Run all tests
test:
	@echo "🧪 Running all tests..."
	bazel test //...
	@echo "✅ Tests completed"

# Run linting
lint:
	@echo "🔍 Running lint checks..."
	./scripts/ci-lint.sh
	@echo "✅ Lint checks completed"

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	bazel clean --expunge
	rm -rf node_modules
	rm -rf .next
	@echo "✅ Clean completed"

# Install dependencies
install:
	@echo "📦 Installing dependencies..."
	npm install
	bazel sync
	@echo "✅ Dependencies installed"

# Update dependencies
deps:
	@echo "🔄 Updating dependencies..."
	npm update
	bazel sync
	@echo "✅ Dependencies updated"

# Build Docker images
docker:
	@echo "🐳 Building Docker images..."
	bazel build //ingest:jira_connector_image
	bazel build //nlp:invariant_extractor_image
	bazel build //proof:lean_compiler_image
	bazel build //platform:api_server_image
	@echo "✅ Docker images built"

# Deploy to production
deploy:
	@echo "🚀 Deploying to production..."
	helm upgrade --install spec-to-proof charts/spec-to-proof
	@echo "✅ Deployment completed"

# Run specific domain tests
test-ingest:
	bazel test //ingest/...

test-nlp:
	bazel test //nlp/...

test-proof:
	bazel test //proof/...

test-platform:
	bazel test //platform/...

# Run specific binaries
run-api:
	bazel run //platform/api_server

run-jira:
	bazel run //ingest/jira_connector

run-nlp:
	bazel run //nlp/invariant_extractor

run-proof:
	bazel run //proof/lean_compiler

# Development utilities
format:
	@echo "🎨 Formatting code..."
	prettier --write .
	@echo "✅ Formatting completed"

type-check:
	@echo "🔍 Type checking..."
	npm run type-check
	@echo "✅ Type checking completed"

# Security checks
security:
	@echo "🔒 Running security checks..."
	cargo audit
	npm audit
	@echo "✅ Security checks completed"

# Performance benchmarks
bench:
	@echo "⚡ Running benchmarks..."
	bazel test //proof:benchmark
	@echo "✅ Benchmarks completed"

# Documentation
docs:
	@echo "📚 Generating documentation..."
	bazel build //docs/...
	@echo "✅ Documentation generated"

# Show project status
status:
	@echo "📊 Project Status"
	@echo "=================="
	@echo "Bazel version: $(shell bazel --version 2>/dev/null || echo 'Not installed')"
	@echo "Rust version: $(shell rustc --version 2>/dev/null || echo 'Not installed')"
	@echo "Node version: $(shell node --version 2>/dev/null || echo 'Not installed')"
	@echo "Lean version: $(shell lean --version 2>/dev/null || echo 'Not installed')"
	@echo "Git branch: $(shell git branch --show-current 2>/dev/null || echo 'Unknown')"
	@echo "Last commit: $(shell git log -1 --oneline 2>/dev/null || echo 'Unknown')"

# Triple-Check Policy: Manual QA
smoke-qa:
	@echo "🧪 Running Spec-to-Proof Smoke QA Tests..."
	@echo "This is part of the Triple-Check Policy for PR validation."
	@echo "Running in clean Docker environment..."
	./scripts/smoke-qa.sh
	@echo "✅ Smoke QA completed successfully" 