#!/bin/bash

set -eu

cargo test --all "$@"
cargo test --all "$@" --features="default serialize base64_format binary_propagator"

# See https://github.com/rust-lang/cargo/issues/5364
cargo test --manifest-path=opentelemetry-contrib/Cargo.toml --all-features
