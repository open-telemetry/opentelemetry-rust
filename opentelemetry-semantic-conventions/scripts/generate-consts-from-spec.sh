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
  --only resource,attribute_group \
  -f /source code \
	--template /templates/semantic_attributes.rs.j2 \
	--output /output/resource.rs \
	--parameters conventions=resource

SED=(sed -i)
if [[ "$(uname)" = "Darwin" ]]; then
  SED=(sed -i "")
fi

# Keep `SCHEMA_URL` key in sync with spec version
"${SED[@]}" "s/\(opentelemetry.io\/schemas\/\)[^\"]*\"/\1$SPEC_VERSION\"/" src/lib.rs

# handle doc generation failures
"${SED[@]}" 's/\[2\]\.$//' src/resource.rs src/trace.rs # remove trailing [2] from few of the doc comments

# Remove the messaging.client_id definition along with its comments from the generated files
#   - semconv "messaging.client_id" is deprecated
#   - semconv "messaging.client.id" is to be used instead
#   - Now because we use:
#      	pub const {{attribute.fqn | to_const_name}}: &str = "{{attribute.fqn}}";
#     to generate the consts, where to_const_name replaces '.' with '_', we need to remove the old definition
#	  to avoid conflicts with the new one. Refer - https://github.com/open-telemetry/semantic-conventions/issues/1031
"${SED[@]}" '/\/\/\/ Deprecated, use `messaging.client.id` instead\./{N;N;N;N;d;}' src/trace.rs src/resource.rs
"${SED[@]}" '/pub const MESSAGING_CLIENT_ID: &str = "messaging.client_id";/{N;d;}' src/trace.rs src/resource.rs

cargo fmt
