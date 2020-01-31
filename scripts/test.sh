#!/bin/bash

set -eu

cargo test --all "$@"
cargo test --all "$@" --features="default serialize"
