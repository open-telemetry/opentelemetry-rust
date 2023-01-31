#!/bin/bash

function patch_version() {
  local latest_version=$(cargo search --limit 1 $1 | head -1 | cut -d'"' -f2)
  echo "patching $1 from $latest_version to $2"
  cargo update -p $1:$latest_version --precise $2
}
# Dashmap >= 5.3.4 requires rust 1.59
patch_version "dashmap" "5.1.0"
# async-global-executor >= 2.3.0 requires rust 1.59
patch_version "async-global-executor" "2.2.0"
