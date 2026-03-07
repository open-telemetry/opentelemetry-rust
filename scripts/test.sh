#!/bin/bash

set -eu

#
# Using '--lib' skips integration tests
#

echo "Running tests for all packages in workspace with --all-features"
cargo test --workspace --all-features --lib

echo "Running doctests for all packages in workspace with --all-features"
cargo test --workspace --all-features --doc --exclude opentelemetry-proto

# See https://github.com/rust-lang/cargo/issues/5364
echo "Running tests for opentelemetry package with --no-default-features"
cargo test --manifest-path=opentelemetry/Cargo.toml --no-default-features --lib

# Run tests for non-workspace member crate
echo "Running tests for opentelemetry-prometheus with --all-features"
(cd opentelemetry-prometheus && cargo test --all-features --lib)

# Run ignored tests one by one separately.
# These tests have global side effects (e.g., setting GlobalTracerProvider or global logger) and cannot run concurrently.
# Using `--ignored --exact` ensures each test runs in isolation without affecting others.
echo "Running ignored tests for opentelemetry-sdk package (global tracer provider tests)"
cargo test --manifest-path=opentelemetry-sdk/Cargo.toml --all-features trace::runtime_tests::test_set_provider_multiple_thread_tokio -- --ignored --exact
cargo test --manifest-path=opentelemetry-sdk/Cargo.toml --all-features trace::runtime_tests::test_set_provider_multiple_thread_tokio_shutdown -- --ignored --exact
cargo test --manifest-path=opentelemetry-sdk/Cargo.toml --all-features trace::runtime_tests::test_set_provider_single_thread_tokio_with_simple_processor -- --ignored --exact
cargo test --manifest-path=opentelemetry-sdk/Cargo.toml --all-features trace::runtime_tests::test_set_provider_single_thread_tokio -- --ignored --exact
cargo test --manifest-path=opentelemetry-sdk/Cargo.toml --all-features trace::runtime_tests::test_set_provider_single_thread_tokio_shutdown -- --ignored --exact

echo "Running ignored tests for opentelemetry-appender-tracing package (global logger tests)"
cargo test --manifest-path=opentelemetry-appender-tracing/Cargo.toml --all-features layer::tests::tracing_appender_standalone_with_tracing_log -- --ignored --exact
cargo test --manifest-path=opentelemetry-appender-tracing/Cargo.toml --all-features layer::tests::tracing_appender_inside_tracing_context_with_tracing_log -- --ignored --exact

# Run test which set environment variable `OTEL_SDK_DISABLED`
echo "Running ignored test for opentelemetry-sdk package with OTEL_SDK_DISABLED environment variable set"
cargo test --package opentelemetry_sdk --lib --all-features -- otel_sdk_disabled_env --ignored
