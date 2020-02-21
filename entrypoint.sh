#!/usr/bin/env bash
set -e
export PATH=$HOME/.cargo/bin:$PATH
cargo build 
cargo test
