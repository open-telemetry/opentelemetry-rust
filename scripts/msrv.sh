#!/bin/bash

set -eu

members=$(cargo metadata -q --no-deps --format-version 1 | jq -r '.packages[].manifest_path')

for member in $members; do
  # needed for windows CI run
  clean_member=$(printf '%s' "$member" | tr -d '\r')
  echo "Verifying MSRV version for $clean_member"
  cargo msrv verify --manifest-path "$clean_member" --output-format json
  echo "" # just for nicer separation between packages
done
