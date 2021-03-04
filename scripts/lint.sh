#!/bin/bash

set -eu

cargo_feature() {
    echo "checking $1 with features $2"
    cargo clippy --manifest-path=$1/Cargo.toml --all-targets --features "$2" --no-default-features -- \
    `# Exit with a nonzero code if there are clippy warnings` \
    -Dwarnings
}

if rustup component add clippy; then
  cargo clippy --all-targets --all-features -- \
    `# Exit with a nonzero code if there are clippy warnings` \
    -Dwarnings

  cargo_feature opentelemetry "trace,rt-tokio"
  cargo_feature opentelemetry "trace,rt-tokio-current-thread"
  cargo_feature opentelemetry "trace,async-std"

  cargo_feature opentelemetry-otlp "default"
  cargo_feature opentelemetry-otlp "default,tls"
  cargo_feature opentelemetry-otlp "default,tls-roots"
  cargo_feature opentelemetry-otlp "trace,grpc-sys"
  cargo_feature opentelemetry-otlp "trace,grpc-sys,openssl"
  cargo_feature opentelemetry-otlp "trace,grpc-sys,openssl-vendored"

  cargo_feature opentelemetry-jaeger "surf_collector_client"
  cargo_feature opentelemetry-jaeger "isahc_collector_client"
  cargo_feature opentelemetry-jaeger "reqwest_blocking_collector_client"
  cargo_feature opentelemetry-jaeger "reqwest_collector_client"
  cargo_feature opentelemetry-jaeger "collector_client"

fi