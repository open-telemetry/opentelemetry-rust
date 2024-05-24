#!/bin/bash

set -eu

cargo test --workspace --all-features

# See https://github.com/rust-lang/cargo/issues/5364
cargo test --manifest-path=opentelemetry/Cargo.toml --no-default-features
