#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="${SCRIPT_DIR}/../"

# freeze the spec version and generator version to make generation reproducible
SPEC_VERSION=1.21.0
SEMCOVGEN_VERSION=0.19.0

cd "$CRATE_DIR"

rm -rf semantic-conventions || true
mkdir semantic-conventions
cd semantic-conventions

git init
git remote add origin https://github.com/open-telemetry/semantic-conventions.git
git fetch origin "v$SPEC_VERSION"
git reset --hard FETCH_HEAD
cd "$CRATE_DIR"

docker run --rm \
	-v "${CRATE_DIR}/semantic-conventions/model:/source" \
	-v "${CRATE_DIR}/scripts/templates:/templates" \
	-v "${CRATE_DIR}/src:/output" \
	otel/semconvgen:$SEMCOVGEN_VERSION \
  --only span,event,attribute_group,scope \
  -f /source code \
	--template /templates/semantic_attributes.rs.j2 \
	--output /output/trace.rs \
	--parameters conventions=trace

docker run --rm \
	-v "${CRATE_DIR}/semantic-conventions/model:/source" \
	-v "${CRATE_DIR}/scripts/templates:/templates" \
	-v "${CRATE_DIR}/src:/output" \
	otel/semconvgen:$SEMCOVGEN_VERSION \
  --only resource \
  -f /source code \
	--template /templates/semantic_attributes.rs.j2 \
	--output /output/resource.rs \
	--parameters conventions=resource

# Keep `SCHEMA_URL` key in sync with spec version
sed -i '' "s/\(opentelemetry.io\/schemas\/\)[^\"]*\"/\1$SPEC_VERSION\"/" src/lib.rs

cargo fmt
