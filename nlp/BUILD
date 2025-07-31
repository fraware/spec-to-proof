load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_test")
load("@rules_proto//proto:defs.bzl", "proto_library")
load("@rules_proto_grpc//rust:defs.bzl", "rust_grpc_library")

package(default_visibility = ["//visibility:public"])

proto_library(
    name = "nlp_proto",
    srcs = ["proto/nlp.proto"],
)

rust_grpc_library(
    name = "nlp_grpc",
    deps = [":nlp_proto"],
)

rust_library(
    name = "nlp_lib",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        ":nlp_grpc",
        "@crate_index//:tokio",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:reqwest",
        "@crate_index//:aws-sdk-dynamodb",
        "@crate_index//:aws-config",
        "@crate_index//:regex",
        "@crate_index//:tracing",
        "@crate_index//:tracing-subscriber",
        "@crate_index//:tonic",
        "@crate_index//:prost-types",
        "@crate_index//:sha2",
        "@crate_index//:hex",
    ],
)

rust_binary(
    name = "invariant_extractor",
    srcs = ["src/bin/invariant_extractor.rs"],
    deps = [":nlp_lib"],
)

rust_test(
    name = "nlp_test",
    srcs = glob(["tests/**/*.rs"]),
    deps = [":nlp_lib"],
) 