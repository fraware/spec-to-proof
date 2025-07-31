.PHONY: all build test fuzz lint clean buf-breaking buf-generate ts-test rust-test

# Default target
all: build test

# Build the proto library
build:
	cargo build --release

# Run all tests
test: rust-test ts-test buf-breaking

# Run Rust tests
rust-test:
	cargo test --release
	cargo test --release --features "fuzz"

# Run TypeScript tests
ts-test:
	cd ../platform/ui && npm test -- --testPathPattern=spec-to-proof.test.ts

# Run fuzz tests with cargo-fuzz
fuzz:
	cargo install cargo-fuzz
	cargo fuzz run round_trip -- -max_len=1000 -runs=100

# Run buf linting
lint:
	buf lint

# Run buf breaking change detection
buf-breaking:
	buf breaking --against '.git#branch=main'

# Generate code from protobuf
buf-generate:
	buf generate

# Clean build artifacts
clean:
	cargo clean
	rm -rf gen/
	rm -rf fuzz/target/

# Install dependencies
install:
	cargo install cargo-fuzz
	cargo install buf
	npm install -g fast-check

# Run property-based tests with coverage
property-test:
	cd ../platform/ui && npm run test:coverage -- --testPathPattern=spec-to-proof.test.ts

# Validate JSON Schema
validate-schema:
	python3 -c "import json; json.load(open('schemas/spec-to-proof.json'))" && echo "JSON Schema is valid"

# Run all validation checks
validate: buf-breaking lint validate-schema

# Performance benchmark
bench:
	cargo bench

# Generate documentation
docs:
	cargo doc --no-deps --open

# Check for security vulnerabilities
security:
	cargo audit
	npm audit

# Format code
format:
	cargo fmt
	cd ../platform/ui && npm run format

# Clippy checks
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# Full CI check
ci: format clippy lint buf-breaking rust-test ts-test fuzz security 