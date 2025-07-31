load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_test")
load("@rules_proto//proto:defs.bzl", "proto_library")
load("@rules_proto_grpc//rust:defs.bzl", "rust_grpc_library")

package(default_visibility = ["//visibility:public"])

proto_library(
    name = "ingest_proto",
    srcs = ["proto/ingest.proto"],
)

rust_grpc_library(
    name = "ingest_grpc",
    deps = [":ingest_proto"],
)

rust_library(
    name = "ingest_lib",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        ":ingest_grpc",
        "@crate_index//:tokio",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:reqwest",
        "@crate_index//:aws-sdk-secretsmanager",
        "@crate_index//:aws-sdk-kms",
        "@crate_index//:aws-config",
        "@crate_index//:nats",
        "@crate_index//:sha2",
        "@crate_index//:chrono",
        "@crate_index//:rand",
        "@crate_index//:tracing",
        "@crate_index//:tracing-subscriber",
        "@crate_index//:aes-gcm",
        "@crate_index//:tonic",
    ],
)

rust_binary(
    name = "jira_connector",
    srcs = ["src/bin/jira_connector.rs"],
    deps = [":ingest_lib"],
)

rust_binary(
    name = "confluence_connector",
    srcs = ["src/bin/confluence_connector.rs"],
    deps = [":ingest_lib"],
)

rust_binary(
    name = "gdocs_connector",
    srcs = ["src/bin/gdocs_connector.rs"],
    deps = [":ingest_lib"],
)

rust_test(
    name = "ingest_test",
    srcs = glob(["tests/**/*.rs"]),
    deps = [
        ":ingest_lib",
        "@crate_index//:wiremock",
        "@crate_index//:aws-config",
    ],
) 