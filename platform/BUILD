load("@aspect_rules_ts//ts:defs.bzl", "ts_project")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_test")
load("@rules_proto//proto:defs.bzl", "proto_library")
load("@rules_proto_grpc//rust:defs.bzl", "rust_grpc_library")

package(default_visibility = ["//visibility:public"])

proto_library(
    name = "platform_proto",
    srcs = ["proto/platform.proto"],
)

rust_grpc_library(
    name = "platform_grpc",
    deps = [":platform_proto"],
)

rust_library(
    name = "platform_lib",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        ":platform_grpc",
        "@crate_index//:tokio",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:axum",
        "@crate_index//:tower",
        "@crate_index//:github-api",
    ],
)

rust_binary(
    name = "api_server",
    srcs = ["src/bin/api_server.rs"],
    deps = [":platform_lib"],
)

rust_binary(
    name = "gh_app",
    srcs = ["src/bin/gh_app.rs"],
    deps = [":platform_lib"],
)

rust_test(
    name = "platform_test",
    srcs = glob(["tests/**/*.rs"]),
    deps = [":platform_lib"],
)

ts_project(
    name = "ui",
    srcs = glob(["ui/src/**/*.ts", "ui/src/**/*.tsx"]),
    declaration = True,
    out_dir = "ui/dist",
    root_dir = "ui",
    tsconfig = "//platform/ui:tsconfig.json",
    deps = [
        "@npm//@types/react",
        "@npm//@types/react-dom",
        "@npm//react",
        "@npm//react-dom",
        "@npm//next",
        "@npm//@trpc/client",
        "@npm//@trpc/server",
        "@npm//zod",
    ],
) 