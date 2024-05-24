#!/bin/bash

set -eu

echo "Running tests for all packages in workspace with --all-features"
cargo test --workspace --all-features

# See https://github.com/rust-lang/cargo/issues/5364
echo "Running tests for opentelemetry package with --no-default-features"
cargo test --manifest-path=opentelemetry/Cargo.toml --no-default-features

# Run global tracer provider test in single thread
echo "Running global tracer provider for opentelemetry-sdk package with single thread."
cargo test --manifest-path=opentelemetry-sdk/Cargo.toml --all-features -- --ignored --test-threads=1
