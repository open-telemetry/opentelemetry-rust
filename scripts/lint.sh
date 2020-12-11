#!/bin/bash

set -eu

if rustup component add clippy; then
  cargo clippy --all-targets --all-features --workspace -- \
    `# Exit with a nonzero code if there are clippy warnings` \
    -Dwarnings \
    "$@"
  cargo clippy --manifest-path=opentelemetry-otlp/Cargo.toml --all-targets --features "grpc-sys" --no-default-features -- \
    `# Exit with a nonzero code if there are clippy warnings` \
    -Dwarnings \
    "$@"
  cargo clippy --manifest-path=opentelemetry-jaeger/Cargo.toml --all-targets --features "surf_collector_client" --no-default-features -- \
    `# Exit with a nonzero code if there are clippy warnings` \
    -Dwarnings \
    "$@"
  cargo clippy --manifest-path=opentelemetry-jaeger/Cargo.toml --all-targets --features "isahc_collector_client" --no-default-features -- \
    `# Exit with a nonzero code if there are clippy warnings` \
    -Dwarnings \
    "$@"
  cargo clippy --manifest-path=opentelemetry-jaeger/Cargo.toml --all-targets --features "reqwest_blocking_collector_client" --no-default-features -- \
    `# Exit with a nonzero code if there are clippy warnings` \
    -Dwarnings \
    "$@"
  cargo clippy --manifest-path=opentelemetry-jaeger/Cargo.toml --all-targets --features "reqwest_collector_client" --no-default-features -- \
    `# Exit with a nonzero code if there are clippy warnings` \
    -Dwarnings \
    "$@"
  cargo clippy --manifest-path=opentelemetry-jaeger/Cargo.toml --all-targets --features "collector_client" --no-default-features -- \
    `# Exit with a nonzero code if there are clippy warnings` \
    -Dwarnings \
    "$@"
fi
