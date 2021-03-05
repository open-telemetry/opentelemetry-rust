#!/bin/bash

set -eu

cargo test --all "$@"

# Run global tracer provider test in single thread
cargo test --manifest-path=opentelemetry/Cargo.toml --all-features -- --ignored --test-threads=1

# See https://github.com/rust-lang/cargo/issues/5364
cargo test --manifest-path=opentelemetry/Cargo.toml --no-default-features
cargo test --manifest-path=opentelemetry/Cargo.toml --all-features
cargo test --manifest-path=opentelemetry-contrib/Cargo.toml --all-features
cargo test --manifest-path=opentelemetry-jaeger/Cargo.toml --all-features
cargo test --manifest-path=opentelemetry-otlp/Cargo.toml --all-features
cargo test --manifest-path=opentelemetry-otlp/Cargo.toml --features "grpc-sys" --no-default-features
cargo test --manifest-path=opentelemetry-zipkin/Cargo.toml --all-features
