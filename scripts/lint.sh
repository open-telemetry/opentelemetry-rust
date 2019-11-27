#!/bin/bash

set -eu

if rustup component add clippy; then
  cargo clippy --all-targets --all -- \
    `# Exit with a nonzero code if there are clippy warnings` \
    -Dwarnings \
    "$@"
fi
