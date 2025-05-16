#!/bin/bash

set -eu

members=$(cargo metadata -q --no-deps --format-version 1 | jq -r '.packages[].manifest_path')

for member in $members; do
  echo "Verifying MSRV version for $member"
  cargo msrv verify --manifest-path "$member" --output-format json
  echo "" # just for nicer separation between packages
done
