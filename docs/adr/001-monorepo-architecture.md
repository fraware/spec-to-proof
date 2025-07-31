# ADR 001: Monorepo Architecture & Technology Stack

## Status
Accepted

## Context
Spec-to-Proof needs a scalable, maintainable architecture that can handle:
- Multiple programming languages (TypeScript, Rust, Lean 4)
- Complex build dependencies
- Fast incremental builds
- Cross-platform development
- Production deployment

## Decision
We will use a Bazel-powered monorepo with the following technology stack:

### Build System: Bazel
- **Rationale**: Bazel provides hermetic, reproducible builds across multiple languages
- **Benefits**: 
  - Incremental builds with dependency tracking
  - Remote execution and caching
  - Cross-language dependency management
  - Google's proven scale

### Language Choices

#### TypeScript + Node 20 (Web Services)
- **Domain**: Platform UI, API servers
- **Rationale**: 
  - Rich ecosystem for web development
  - Strong typing with TypeScript
  - Excellent tooling (ESLint, Prettier, Jest)
  - Next.js 14 for modern React development

#### Rust 1.78 (Heavy NLP/Queue Workers)
- **Domain**: Ingest connectors, NLP pipeline, proof generation
- **Rationale**:
  - Zero-cost abstractions for performance-critical code
  - Memory safety without garbage collection
  - Excellent async runtime with Tokio
  - Strong ecosystem for systems programming

#### Lean 4.3.x (Proofs)
- **Domain**: Formal theorem proving
- **Rationale**:
  - State-of-the-art theorem prover
  - Functional programming with dependent types
  - Excellent for formal verification
  - Active development by Microsoft Research

#### Terraform 1.9 (Infrastructure as Code)
- **Domain**: Cloud infrastructure provisioning
- **Rationale**:
  - Industry standard for IaC
  - Multi-cloud support
  - Strong state management
  - Rich provider ecosystem

### Interface Contracts: gRPC + Protocol Buffers
- **Rationale**:
  - Language-agnostic serialization
  - Strong typing and schema evolution
  - Excellent performance
  - Bidirectional streaming support

## Module Boundaries

### `/ingest/` - Document Ingestion
- **Purpose**: Connect to external systems (Jira, Confluence, Google Docs)
- **Components**: 
  - OAuth2 token management
  - Incremental document fetching
  - NATS JetStream integration
- **Interfaces**: gRPC services for document streaming

### `/nlp/` - Natural Language Processing
- **Purpose**: Extract invariants from plain English specifications
- **Components**:
  - Claude 3 Opus integration
  - Prompt chaining and orchestration
  - Deterministic post-processing
- **Interfaces**: gRPC services for invariant extraction

### `/proof/` - Formal Verification
- **Purpose**: Generate and verify Lean 4 theorems
- **Components**:
  - LLM-to-Lean compiler
  - Sandboxed proof farm
  - Proof artifact management
- **Interfaces**: gRPC services for proof generation

### `/platform/` - Web Platform
- **Purpose**: User interface and API management
- **Components**:
  - Next.js 14 frontend
  - tRPC API server
  - GitHub App integration
- **Interfaces**: REST APIs and WebSocket connections

## Consequences

### Positive
- **Scalability**: Bazel enables efficient builds at scale
- **Performance**: Rust provides excellent performance for critical paths
- **Type Safety**: Strong typing across all languages
- **Maintainability**: Clear module boundaries and interfaces
- **Deployment**: Consistent build artifacts across environments

### Negative
- **Complexity**: Multiple languages increase cognitive load
- **Learning Curve**: Team needs expertise in multiple technologies
- **Build Time**: Initial Bazel setup can be complex
- **Tooling**: Need to maintain tooling for multiple languages

### Risks
- **Team Expertise**: Requires diverse skill set
- **Integration Complexity**: Cross-language communication via gRPC
- **Build Performance**: Bazel configuration complexity

## Mitigation Strategies
1. **Documentation**: Comprehensive ADRs and runbooks
2. **Training**: Cross-training sessions for team members
3. **Tooling**: Automated linting and testing across all languages
4. **Monitoring**: Comprehensive observability stack
5. **Gradual Migration**: Phased rollout of new components

## Implementation Plan
1. **Phase 1**: Set up Bazel workspace and basic structure
2. **Phase 2**: Implement core gRPC interfaces
3. **Phase 3**: Build ingest connectors
4. **Phase 4**: Develop NLP pipeline
5. **Phase 5**: Create proof generation system
6. **Phase 6**: Build platform UI and API
7. **Phase 7**: Deploy and monitor

## References
- [Bazel Documentation](https://bazel.build/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Lean 4 Documentation](https://leanprover.github.io/lean4/doc/)
- [gRPC Documentation](https://grpc.io/docs/)
- [Protocol Buffers](https://developers.google.com/protocol-buffers) 