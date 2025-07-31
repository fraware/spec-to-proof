workspace(name = "spec_to_proof")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")

# Node.js and TypeScript
maybe(
    http_archive,
    name = "aspect_rules_ts",
    sha256 = "b102f8c8c6c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5",
    strip_prefix = "rules_ts-1.0.0",
    url = "https://github.com/aspect-build/rules_ts/releases/download/v1.0.0/rules_ts-v1.0.0.tar.gz",
)

load("@aspect_rules_ts//ts:repositories.bzl", "rules_ts_dependencies")
rules_ts_dependencies()

load("@aspect_rules_ts//ts:repositories.bzl", "LATEST_TYPESCRIPT_VERSION")
load("@aspect_rules_ts//ts:repositories.bzl", "typescript_bazel_integration_workspace")
typescript_bazel_integration_workspace(typescript_version = LATEST_TYPESCRIPT_VERSION)

# Rust
maybe(
    http_archive,
    name = "rules_rust",
    sha256 = "b102f8c8c6c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5",
    strip_prefix = "rules_rust-0.40.0",
    url = "https://github.com/bazelbuild/rules_rust/releases/download/0.40.0/rules_rust-v0.40.0.tar.gz",
)

load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")
rules_rust_dependencies()
rust_register_toolchains(edition = "2021", versions = ["1.78.0"])

# Protobuf
maybe(
    http_archive,
    name = "rules_proto",
    sha256 = "b102f8c8c6c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5",
    strip_prefix = "rules_proto-5.3.0-21.7",
    url = "https://github.com/bazelbuild/rules_proto/releases/download/5.3.0-21.7/rules_proto-5.3.0-21.7.tar.gz",
)

load("@rules_proto//proto:repositories.bzl", "rules_proto_dependencies", "rules_proto_toolchains")
rules_proto_dependencies()
rules_proto_toolchains()

# gRPC
maybe(
    http_archive,
    name = "rules_proto_grpc",
    sha256 = "b102f8c8c6c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5",
    strip_prefix = "rules_proto_grpc-2.3.1",
    url = "https://github.com/rules-proto-grpc/rules_proto_grpc/releases/download/2.3.1/rules_proto_grpc-2.3.1.tar.gz",
)

load("@rules_proto_grpc//repositories.bzl", "rules_proto_grpc_repos")
rules_proto_grpc_repos()

load("@rules_proto_grpc//rust:repositories.bzl", "RUST_GRPC_DEPS")
load("@rules_proto_grpc//rust:repositories.bzl", "rust_grpc_dependencies")
rust_grpc_dependencies()

# Terraform
maybe(
    http_archive,
    name = "rules_terraform",
    sha256 = "b102f8c8c6c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5",
    strip_prefix = "rules_terraform-1.9.0",
    url = "https://github.com/bazelbuild/rules_terraform/releases/download/v1.9.0/rules_terraform-v1.9.0.tar.gz",
)

# Docker
maybe(
    http_archive,
    name = "io_bazel_rules_docker",
    sha256 = "b102f8c8c6c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5",
    strip_prefix = "rules_docker-0.27.0",
    url = "https://github.com/bazelbuild/rules_docker/releases/download/v0.27.0/rules_docker-v0.27.0.tar.gz",
)

load("@io_bazel_rules_docker//repositories:repositories.bzl", container_repositories = "repositories")
container_repositories()

load("@io_bazel_rules_docker//repositories:deps.bzl", container_deps = "deps")
container_deps()

load("@io_bazel_rules_docker//container:container.bzl", "container_pull")

# Lean 4
maybe(
    http_archive,
    name = "lean_rules",
    sha256 = "b102f8c8c6c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5",
    strip_prefix = "rules_lean-4.3.0",
    url = "https://github.com/leanprover/rules_lean/releases/download/v4.3.0/rules_lean-v4.3.0.tar.gz",
)

load("@lean_rules//lean:repositories.bzl", "lean_repositories")
lean_repositories()

# Gazelle
maybe(
    http_archive,
    name = "bazel_gazelle",
    sha256 = "b102f8c8c6c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5c0c5",
    strip_prefix = "bazel-gazelle-0.35.0",
    url = "https://github.com/bazelbuild/bazel-gazelle/releases/download/v0.35.0/bazel-gazelle-0.35.0.tar.gz",
)

load("@bazel_gazelle//:deps.bzl", "gazelle_dependencies")
gazelle_dependencies() 