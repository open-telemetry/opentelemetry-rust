#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="${SCRIPT_DIR}/../"

# freeze the spec version and generator version to make generation reproducible
SPEC_VERSION=v1.3.0
SEMCOVGEN_VERSION=0.3.1

cd "$CRATE_DIR"

rm -rf opentelemetry-specification || true
mkdir opentelemetry-specification
cd opentelemetry-specification

git init
git remote add origin https://github.com/open-telemetry/opentelemetry-specification.git
git fetch origin "$SPEC_VERSION"
git reset --hard FETCH_HEAD
cd "$CRATE_DIR"

docker run --rm \
	-v "${CRATE_DIR}/opentelemetry-specification/semantic_conventions/trace:/source" \
	-v "${CRATE_DIR}/scripts/templates:/templates" \
	-v "${CRATE_DIR}/src:/output" \
	otel/semconvgen:$SEMCOVGEN_VERSION \
	--yaml-root /source \
	code \
	--template /templates/semantic_attributes.rs.j2 \
	--output /output/trace.rs \
	--parameters conventions=trace

docker run --rm \
	-v "${CRATE_DIR}/opentelemetry-specification/semantic_conventions/resource:/source" \
	-v "${CRATE_DIR}/scripts/templates:/templates" \
	-v "${CRATE_DIR}/src:/output" \
	otel/semconvgen:$SEMCOVGEN_VERSION \
	--yaml-root /source \
	code \
	--template /templates/semantic_attributes.rs.j2 \
	--output /output/resource.rs \
	--parameters conventions=resource

cargo fmt
