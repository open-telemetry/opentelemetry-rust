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
fi
