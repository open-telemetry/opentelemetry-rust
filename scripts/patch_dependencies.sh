#!/bin/bash

# Function to compare two version numbers
function version_lt() {
    [ "$(printf '%s\n' "$1" "$2" | sort -V | head -n1)" != "$2" ]
}

# Get the current Rust compiler version
rust_version=$(rustc --version | cut -d' ' -f2)

# Target version (Rust 1.71.1)
target_version="1.71.1"

# Check if the current Rust version is less than the target version
  function patch_version() {
    local latest_version=$(cargo search --limit 1 $1 | head -1 | cut -d'"' -f2)
    echo "patching $1 from $latest_version to $2"
    cargo update -p $1:$latest_version --precise $2
  }

  patch_version cc 1.0.105
  patch_version url 2.5.0
if version_lt "$rust_version" "$target_version"; then
  patch_version tonic 0.12.2
  patch_version hyper-rustls 0.27.2 # 0.27.3 needs rustc v1.70.0
  patch_version tokio-util 0.7.11 # 0.7.12 needs rustc v1.70.0
  patch_version tokio-stream 0.1.15 # 0.1.16 needs rustc v1.70.0
  patch_version tokio 1.38.0 # 1.39 needs msrv bump to rustc 1.70
fi
