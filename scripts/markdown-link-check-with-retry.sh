#!/bin/bash -e

# this script helps to reduce sporadic link check failures by retrying at a file-by-file level

retry_count=3
ignore_dirs=("opentelemetry-proto/src/proto/opentelemetry-proto")

for file in "$@"; do
  skip=false
  # Check if file path contains any ignore dir
  for ignore_dir in "${ignore_dirs[@]}"; do
    if [[ "$file" == *"$ignore_dir"* ]]; then
      skip=true
      break
    fi
  done
  if [ "$skip" = false ]; then
    for i in $(seq 1 $retry_count); do
      if markdown-link-check --config "$(dirname "$0")/markdown-link-check-config.json" \
                           "$file"; then
      break
    elif [[ $i -eq $retry_count ]]; then
      exit 1
    fi
    sleep 5
  done
  fi
done
