#!/bin/bash

set -euo pipefail

script_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$script_dir/.." && pwd)
config_file="$script_dir/msrv_config.json"
requested_class=${1:-all}
metadata_file=$(mktemp)
trap 'rm -f "$metadata_file"' EXIT

cd "$repo_root"
cargo metadata -q --no-deps --format-version 1 > "$metadata_file"

if ! jq -e '[.classes[].packages[]] | length == (unique | length)' "$config_file" >/dev/null; then
  echo "A package appears in more than one MSRV class." >&2
  exit 1
fi

unclassified=$(jq -r --slurpfile config "$config_file" '
  ([.packages[] | select(.publish == null) | .name]
    - [$config[0].classes[].packages[]])[]
' "$metadata_file")
if [ -n "$unclassified" ]; then
  echo "Published packages missing from $config_file:" >&2
  echo "$unclassified" >&2
  exit 1
fi

unknown=$(jq -r --slurpfile metadata "$metadata_file" '
  ([$metadata[0].packages[] | select(.publish == null) | .name]) as $published
  | [.classes[].packages[]] - $published
  | .[]
' "$config_file")
if [ -n "$unknown" ]; then
  echo "Unknown or unpublished packages in $config_file:" >&2
  echo "$unknown" >&2
  exit 1
fi

if [ "$requested_class" = "all" ]; then
  classes=$(jq -r '.classes | keys[]' "$config_file")
elif jq -e --arg class "$requested_class" '.classes[$class] != null' "$config_file" >/dev/null; then
  classes=$requested_class
else
  echo "Unknown MSRV class '$requested_class'. Expected one of: all, $(jq -r '.classes | keys | join(", ")' "$config_file")" >&2
  exit 1
fi

for class in $classes; do
  expected_version=$(jq -r --arg class "$class" '.classes[$class].rust_version' "$config_file")
  packages=$(jq -r --arg class "$class" '.classes[$class].packages[]' "$config_file")

  echo "Verifying $class packages against Rust $expected_version"
  for package in $packages; do
    manifest=$(jq -r --arg package "$package" '.packages[] | select(.name == $package) | .manifest_path' "$metadata_file" | tr -d '\r')
    declared_version=$(jq -r --arg package "$package" '.packages[] | select(.name == $package) | .rust_version' "$metadata_file")

    if [ "$declared_version" != "$expected_version" ]; then
      echo "$package declares rust-version $declared_version; expected $expected_version for the $class class." >&2
      exit 1
    fi

    echo "Verifying $package ($manifest)"
    cargo msrv verify \
      --manifest-path "$manifest" \
      --rust-version "$expected_version" \
      --all-features \
      --output-format json
    echo ""
  done
done
