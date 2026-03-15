#!/bin/bash

set -eu

# Run clippy for each feature individually across all workspace crates.
# This catches dead code, unused imports, and other warnings that only
# surface when features are tested in isolation (e.g., a function gated
# behind feature A but missing #[cfg(feature = "A")]).

if rustup component add clippy && \
  ((cargo --list | grep -q hack) || cargo install cargo-hack); then
  cargo hack --each-feature --no-dev-deps clippy -- -Dwarnings
fi
