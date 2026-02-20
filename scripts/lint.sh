#!/bin/bash

set -eu

cargo_feature() {
    echo "checking $1 with features $2"
    cargo clippy --manifest-path=$1/Cargo.toml --all-targets --features "$2" --no-default-features -- \
    `# Exit with a nonzero code if there are clippy warnings` \
    -Dwarnings
}

if rustup component add clippy; then
  # Exit with a nonzero code if there are clippy warnings
  cargo clippy --workspace --all-targets --all-features -- -Dwarnings

  # `opentelemetry-prometheus` doesn't belong to the workspace
  cargo clippy --manifest-path=opentelemetry-prometheus/Cargo.toml --all-targets --all-features -- \
    -Dwarnings

  # Multi-feature combinations not covered by per-feature checks
  cargo_feature opentelemetry "trace,metrics,logs,testing"

  cargo_feature opentelemetry-otlp "default,tls"
  cargo_feature opentelemetry-otlp "default,tls-roots"
  cargo_feature opentelemetry-otlp "http-proto, reqwest-blocking-client"
  cargo_feature opentelemetry-otlp "http-proto, reqwest-client"
  cargo_feature opentelemetry-otlp "http-proto, reqwest-rustls"

  cargo_feature opentelemetry-proto "gen-tonic,trace"
  cargo_feature opentelemetry-proto "gen-tonic,trace,with-serde"
  cargo_feature opentelemetry-proto "gen-tonic,trace,with-schemars,with-serde"
  cargo_feature opentelemetry-proto "gen-tonic,metrics"
  cargo_feature opentelemetry-proto "gen-tonic,metrics,with-serde"
  cargo_feature opentelemetry-proto "gen-tonic,metrics,with-schemars,with-serde"
  cargo_feature opentelemetry-proto "gen-tonic,logs"
  cargo_feature opentelemetry-proto "gen-tonic,logs,with-serde"
  cargo_feature opentelemetry-proto "gen-tonic,logs,with-schemars,with-serde"

fi
