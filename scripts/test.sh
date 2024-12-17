#!/bin/bash

set -eu

#
# Using '--lib' skips integration tests
#

echo "Running tests for all packages in workspace with --all-features"
cargo test --workspace --all-features --lib

# See https://github.com/rust-lang/cargo/issues/5364
echo "Running tests for opentelemetry package with --no-default-features"
cargo test --manifest-path=opentelemetry/Cargo.toml --no-default-features --lib

# Run global tracer provider test in single thread
# //TODO: This tests were not running for a while. Need to find out how to run
# run them. Using --ignored will run other tests as well, so that cannot be used.
# echo "Running global tracer provider for opentelemetry-sdk package with single thread."
# cargo test --manifest-path=opentelemetry-sdk/Cargo.toml --all-features -- --test-threads=1 --lib
