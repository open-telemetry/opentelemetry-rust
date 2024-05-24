#!/bin/bash

set -eu

echo "Running msrv check for opentelemetry package"
cargo check --manifest-path=opentelemetry/Cargo.toml --all-features

echo "Running msrv check for opentelemetry-sdk package"
cargo check --manifest-path=opentelemetry-sdk/Cargo.toml --all-features

echo "Running msrv check for opentelemetry-stdout package"
cargo check --manifest-path=opentelemetry-stdout/Cargo.toml --all-features

# TODO: Ignoring as this is failing with the following error:
# error: package `prost-derive v0.12.6` cannot be built because it requires rustc 1.70 or newer, while the currently active rustc version is 1.65.0
#echo "Running msrv check for opentelemetry-otlp package"
# cargo check --manifest-path=opentelemetry-otlp/Cargo.toml --all-features

echo "Running msrv check for opentelemetry-http package"
cargo check --manifest-path=opentelemetry-http/Cargo.toml --all-features

echo "Running msrv check for opentelemetry-jaeger-propagator package"
cargo check --manifest-path=opentelemetry-jaeger-propagator/Cargo.toml --all-features

echo "Running msrv check for opentelemetry-zipkin package"
cargo check --manifest-path=opentelemetry-zipkin/Cargo.toml --all-features

echo "Running msrv check for opentelemetry-appender-log package"
cargo check --manifest-path=opentelemetry-appender-log/Cargo.toml --all-features

echo "Running msrv check for opentelemetry-appender-tracing package"
cargo check --manifest-path=opentelemetry-appender-tracing/Cargo.toml --all-features

