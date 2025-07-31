# Spec-to-Proof

> Turns everyday product specs into machine-checked Lean 4 guarantees, closing the gap between "should" and "will."

[![CI](https://github.com/fraware/spec-to-proof/workflows/CI/badge.svg)](https://github.com/fraware/spec-to-proof/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.78.0+-blue.svg)](https://www.rust-lang.org/)
[![TypeScript](https://img.shields.io/badge/typescript-5.0+-blue.svg)](https://www.typescriptlang.org/)
[![Lean](https://img.shields.io/badge/lean-4.3.0+-green.svg)](https://leanprover.github.io/)

## Overview

Spec-to-Proof automatically extracts hidden invariants from plain English specifications and generates formal proofs using Lean 4. Connect your Jira, Confluence, or Google Docs, and our NLP pipeline transforms natural language into mathematically verified guarantees.

### Key Features

- **Multi-Platform Integration**: Connect to Jira, Confluence, and Google Docs
- **AI-Powered Extraction**: Claude 3 Opus extracts invariants from plain English
- **Formal Verification**: Lean 4 generates machine-checked proofs
- **GitHub Integration**: Automatic PR badges with cryptographic signatures
- **Real-time Coverage**: Dashboard mapping user stories to formal guarantees
- **Drift Detection**: Automatic re-verification when specs change

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Ingest Layer  │    │   NLP Pipeline  │    │  Proof Engine   │
│                 │    │                 │    │                 │
│ • Jira Connector│───▶│ • Invariant     │───▶│ • Lean Compiler │
│ • Confluence    │    │   Extraction    │    │ • Proof Farm    │
│ • Google Docs   │    │ • Disambiguation│    │ • Artifact Mgmt │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
┌─────────────────┐    ┌─────────────────┐
│  Platform UI    │◀───│   GitHub App    │
│                 │    │                 │
│ • Next.js 14    │    │ • PR Badges     │
│ • tRPC API      │    │ • Sigstore      │
│ • Coverage Dash │    │ • Webhooks      │
└─────────────────┘    └─────────────────┘
```

## Technology Stack

- **Build System**: Bazel 6.4.0
- **Backend**: Rust 1.78.0 (performance-critical services)
- **Frontend**: TypeScript + Node 20 (Next.js 14)
- **Formal Verification**: Lean 4.3.x
- **Infrastructure**: Terraform 1.9
- **Communication**: gRPC + Protocol Buffers
- **Message Queue**: NATS JetStream
- **Storage**: AWS S3, DynamoDB, Redis

## Quick Start

### Prerequisites

- [Bazel](https://bazel.build/install) 6.4.0+
- [Rust](https://rustup.rs/) 1.78.0+
- [Node.js](https://nodejs.org/) 20+
- [Lean 4](https://leanprover.github.io/lean4/doc/setup.html) 4.3.0+

### Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/fraware/spec-to-proof.git
   cd spec-to-proof
   ```

2. **Install dependencies**
   ```bash
   # Install Node.js dependencies
   npm install
   
   # Install Rust dependencies (handled by Bazel)
   bazel sync
   ```

3. **Run the development environment**
   ```bash
   # Start all services locally
   make dev
   
   # Or run individual components
   bazel run //platform/api_server
   bazel run //ingest/jira_connector
   bazel run //nlp/invariant_extractor
   bazel run //proof/lean_compiler
   ```

4. **Run tests**
   ```bash
   # Run all tests
   bazel test //...
   
   # Run specific domain tests
   bazel test //ingest/...
   bazel test //nlp/...
   bazel test //proof/...
   bazel test //platform/...
   ```

5. **Run linting**
   ```bash
   ./scripts/ci-lint.sh
   ```

## Project Structure

```
spec-to-proof/
├── ingest/           # Document ingestion connectors
│   ├── src/         # Rust source code
│   ├── proto/       # gRPC service definitions
│   └── tests/       # Integration tests
├── nlp/             # Natural language processing
│   ├── src/         # Rust NLP pipeline
│   ├── prompts/     # Claude 3 prompt templates
│   └── tests/       # Unit tests
├── proof/           # Formal verification engine
│   ├── src/         # Rust proof generation
│   ├── lean/        # Lean 4 theorem definitions
│   └── tests/       # Proof verification tests
├── platform/        # Web platform and APIs
│   ├── src/         # Rust API server
│   ├── ui/          # Next.js 14 frontend
│   └── tests/       # API tests
├── terraform/       # Infrastructure as Code
├── charts/          # Kubernetes Helm charts
├── docs/            # Documentation
├── scripts/         # Build and deployment scripts
└── e2e/            # End-to-end tests
```

## Development Workflow

### Adding New Features

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Implement your changes**
   - Follow the established patterns for your domain
   - Add tests for new functionality
   - Update documentation as needed

3. **Run quality checks**
   ```bash
   ./scripts/ci-lint.sh
   bazel test //...
   ```

4. **Submit a pull request**
   - Ensure CI passes
   - Get review from domain owners (see CODEOWNERS)
   - Address feedback and merge

### Code Quality Standards

- **Rust**: Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- **TypeScript**: Use ESLint + Prettier configuration
- **Lean**: Follow [Lean 4 Style Guide](https://leanprover.github.io/lean4/doc/style.html)
- **Terraform**: Use `terraform fmt` and `terraform validate`

## Deployment

### Local Development

```bash
# Start all services with Docker Compose
docker-compose up -d

# Or run with Bazel
bazel run //platform/api_server
```

### Production Deployment

```bash
# Deploy to Kubernetes
helm install spec-to-proof charts/spec-to-proof

# Or deploy with Terraform
terraform apply
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Environment

- **IDE**: VS Code with Rust, TypeScript, and Lean extensions
- **Testing**: Unit tests for each domain, integration tests for workflows
- **Documentation**: ADRs for architectural decisions, runbooks for operations

### Code Review Process

1. **Automated Checks**: CI must pass (lint, test, build)
2. **Peer Review**: Minimum two senior reviewers
3. **Manual QA**: Smoke tests in clean environment

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.