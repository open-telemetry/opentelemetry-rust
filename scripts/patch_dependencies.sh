#!/bin/bash

function patch_version() {
    local latest_version=$(cargo search --limit 1 $1 | head -1 | cut -d'"' -f2)
    echo "patching $1 from $latest_version to $2"
    cargo update -p $1:$latest_version --precise $2
}

patch_version url 2.5.2 #https://github.com/servo/rust-url/issues/992
patch_version rustls-native-certs 0.8.0 #0.8.1 needs rustc 1.71 or newer
patch_version rustls 0.23.17 #0.23.18 needs rustc 1.71 or newer
