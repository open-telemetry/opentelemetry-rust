#!/bin/bash

function patch_version() {
  local latest_version=$(cargo search --limit 1 $1 | head -1 | cut -d'"' -f2)
  echo "patching $1 from $latest_version to $2"
  cargo update -p $1:$latest_version --precise $2
}

