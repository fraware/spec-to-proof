load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_test")
load("@rules_proto//proto:defs.bzl", "proto_library")
load("@rules_proto_grpc//rust:defs.bzl", "rust_grpc_library")
load("@lean_rules//lean:defs.bzl", "lean_library", "lean_binary")

package(default_visibility = ["//visibility:public"])

proto_library(
    name = "proof_proto",
    srcs = ["proto/proof.proto"],
    deps = ["//proto:spec_to_proof_proto"],
)

rust_grpc_library(
    name = "proof_grpc",
    deps = [":proof_proto"],
)

rust_library(
    name = "proof_lib",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        ":proof_grpc",
        "//proto:spec_to_proof_grpc",
        "@crate_index//:tokio",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:aws-sdk-s3",
        "@crate_index//:aws-sdk-kms",
        "@crate_index//:aws-config",
        "@crate_index//:reqwest",
        "@crate_index//:tonic",
        "@crate_index//:tracing",
        "@crate_index//:tracing-subscriber",
        "@crate_index//:prost-types",
        "@crate_index//:sha2",
        "@crate_index//:futures",
        "@crate_index//:rand",
        "@crate_index//:tokio-stream",
    ],
)

rust_binary(
    name = "lean_compiler",
    srcs = ["src/bin/lean_compiler.rs"],
    deps = [":proof_lib"],
)

rust_binary(
    name = "proof_farm",
    srcs = ["src/bin/proof_farm.rs"],
    deps = [":proof_lib"],
)

rust_test(
    name = "proof_test",
    srcs = glob(["tests/**/*.rs"]),
    deps = [":proof_lib"],
)

rust_test(
    name = "lean_compiler_test",
    srcs = ["tests/lean_compiler_test.rs"],
    deps = [":proof_lib"],
)

lean_library(
    name = "spec_theorems",
    srcs = glob(["lean/**/*.lean"]),
    deps = [],
) 