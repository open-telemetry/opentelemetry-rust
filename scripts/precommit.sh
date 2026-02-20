REPO_ROOT=$(dirname $( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd ))

pushd "${REPO_ROOT}" > /dev/null

cargo update && cargo fmt --all && ./scripts/lint.sh && ./scripts/lint_feature_matrix.sh && ./scripts/test.sh

popd > /dev/null