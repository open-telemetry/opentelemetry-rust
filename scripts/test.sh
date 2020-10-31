#!/bin/bash

set -eu

cargo test --all "$@"

# See https://github.com/rust-lang/cargo/issues/5364
cargo test --manifest-path=opentelemetry/Cargo.toml --all-features
cargo test --manifest-path=opentelemetry-contrib/Cargo.toml --all-features
cargo test --manifest-path=opentelemetry-jaeger/Cargo.toml --all-features
cargo test --manifest-path=opentelemetry-zipkin/Cargo.toml --all-features
