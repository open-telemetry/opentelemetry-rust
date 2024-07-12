#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="${SCRIPT_DIR}/../"

# freeze the spec version and generator version to make generation reproducible
SPEC_VERSION=1.26.0
SEMCOVGEN_VERSION=0.24.0

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
  -f /source code \
	--template /templates/semantic_attributes.rs.j2 \
	--output /output/attribute.rs \
	--parameters conventions=attribute

docker run --rm \
	-v "${CRATE_DIR}/semantic-conventions/model:/source" \
	-v "${CRATE_DIR}/scripts/templates:/templates" \
	-v "${CRATE_DIR}/src:/output" \
	otel/semconvgen:$SEMCOVGEN_VERSION \
  --only span,event,scope \
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

docker run --rm \
	-v "${CRATE_DIR}/semantic-conventions/model:/source" \
	-v "${CRATE_DIR}/scripts/templates:/templates" \
	-v "${CRATE_DIR}/src:/output" \
	otel/semconvgen:$SEMCOVGEN_VERSION \
  -f /source code \
	--template /templates/semantic_metrics.rs.j2 \
	--output /output/metric.rs

SED=(sed -i)
if [[ "$(uname)" = "Darwin" ]]; then
  SED=(sed -i "")
fi

# Keep `SCHEMA_URL` key in sync with spec version
"${SED[@]}" "s/\(opentelemetry.io\/schemas\/\)[^\"]*\"/\1$SPEC_VERSION\"/" src/lib.rs

# handle doc generation failures
"${SED[@]}" 's/\[2\]\.$//' src/attribute.rs # remove trailing [2] from few of the doc comments

# handle escaping ranges like [0,n] / [0.0, ...] in descriptions/notes which will cause broken intra-doc links
# unescape any mistakenly escaped ranges which actually contained a link (not that we currently have any)
expression='
  s/\[([a-zA-Z0-9\.\s]+,[a-zA-Z0-9\.\s]+)\]/\\[\1\\]/g
  s/\\\[([^\]]+)\]\(([^)]+)\)/[\1](\2)/g
'
"${SED[@]}" -E "${expression}" src/metric.rs
"${SED[@]}" -E "${expression}" src/attribute.rs

cargo fmt
