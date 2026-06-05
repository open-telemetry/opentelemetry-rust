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

# opentelemetry-otlp has a large feature matrix. The --all-features run above
# does NOT exercise test code that is compiled out when a TLS feature is
# enabled (e.g. the no-TLS gRPC scheme-handling paths), and it never catches
# per-feature compile breakage. Run a curated set of real-world feature
# combinations so that feature-gated tests actually compile and execute.
echo "Running opentelemetry-otlp lib tests across curated feature combinations"
for otlp_features in \
  "grpc-tonic,trace,metrics,logs" \
  "http-proto,trace,metrics,logs,reqwest-blocking-client" \
  "http-json,trace,metrics,logs,reqwest-blocking-client"; do
  echo "  features: ${otlp_features}"
  cargo test -p opentelemetry-otlp --no-default-features --features "${otlp_features}" --lib
done

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

echo "Running ignored tests for opentelemetry-sdk package (self-diagnostics, requires global MeterProvider)"
cargo test --manifest-path=opentelemetry-sdk/Cargo.toml --all-features logs::batch_log_processor::tests::self_diagnostics_counter_records_success -- --ignored --exact

echo "Running ignored tests for opentelemetry-appender-tracing package (global logger tests)"
cargo test --manifest-path=opentelemetry-appender-tracing/Cargo.toml --all-features layer::tests::tracing_appender_standalone_with_tracing_log -- --ignored --exact
cargo test --manifest-path=opentelemetry-appender-tracing/Cargo.toml --all-features layer::tests::tracing_appender_inside_tracing_context_with_tracing_log -- --ignored --exact
