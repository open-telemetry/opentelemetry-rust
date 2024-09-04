#!/bin/bash

function patch_version() {
  local latest_version=$(cargo search --limit 1 $1 | head -1 | cut -d'"' -f2)
  echo "patching $1 from $latest_version to $2"
  cargo update -p $1:$latest_version --precise $2
}

patch_version cc 1.0.105
patch_version url 2.5.0
patch_version hyper-rustls 0.27.2 # 0.27.3 needs rustc v1.70.0
