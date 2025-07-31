load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library")
load("@rules_proto//proto:defs.bzl", "proto_library")
load("@io_bazel_rules_go//go:defs.bzl", "go_binary", "go_library")

package(default_visibility = ["//visibility:public"])

proto_library(
    name = "gh_app_proto",
    srcs = ["proto/gh_app.proto"],
    deps = [
        "//proto:spec_to_proof_proto",
    ],
)

rust_library(
    name = "gh_app_lib",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        "//proto:spec_to_proof_rust",
        "@crates_index//:axum",
        "@crates_index//:tokio",
        "@crates_index//:serde",
        "@crates_index//:serde_json",
        "@crates_index//:jsonwebtoken",
        "@crates_index//:reqwest",
        "@crates_index//:tracing",
        "@crates_index//:anyhow",
        "@crates_index//:thiserror",
        "@crates_index//:uuid",
        "@crates_index//:base64",
        "@crates_index//:sha2",
        "@crates_index//:hmac",
        "@crates_index//:hex",
        "@crates_index//:chrono",
        "@crates_index//:async_trait",
        "@crates_index//:futures",
        "@crates_index//:tower",
        "@crates_index//:tower_http",
        "@crates_index//:tracing_subscriber",
        "@crates_index//:clap",
        "@crates_index//:config",
        "@crates_index//:aws_sdk_secretsmanager",
        "@crates_index//:aws_sdk_s3",
        "@crates_index//:aws_sdk_sts",
        "@crates_index//:sigstore_rs",
        "@crates_index//:openssl",
    ],
)

rust_binary(
    name = "gh_app",
    srcs = ["src/main.rs"],
    deps = [
        ":gh_app_lib",
    ],
)

go_binary(
    name = "gh_app_go",
    srcs = ["cmd/main.go"],
    deps = [
        ":gh_app_go_lib",
    ],
)

go_library(
    name = "gh_app_go_lib",
    srcs = glob(["pkg/**/*.go"]),
    importpath = "github.com/spec-to-proof/gh-app",
    deps = [
        "@com_github_gin_gonic_gin//:gin",
        "@com_github_google_go_github_v62_github//:github",
        "@com_github_sigstore_sigstore_pkg_signing//:signing",
        "@com_github_sigstore_sigstore_pkg_verification//:verification",
        "@com_github_aws_aws_sdk_go_v2_service_secretsmanager//:secretsmanager",
        "@com_github_aws_aws_sdk_go_v2_service_s3//:s3",
    ],
)

filegroup(
    name = "gh_app_sources",
    srcs = glob([
        "src/**/*.rs",
        "proto/**/*.proto",
        "cmd/**/*.go",
        "pkg/**/*.go",
        "config/**/*.yaml",
        "config/**/*.yml",
        "config/**/*.json",
    ]),
) 