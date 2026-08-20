#!/usr/bin/env bash
#
# ci-doctor-report.sh — open, close, or leave alone a "CI Doctor" tracking issue
# based on the outcome of a scheduled hygiene check.
#
# The script is idempotent:
#   * If the check FAILED and no open issue with the signature exists → create one.
#   * If the check FAILED and an open issue exists → do nothing (avoid comment spam).
#   * If the check PASSED and an open issue with the signature exists → close it.
#   * If the check PASSED and no issue exists → do nothing.
#
# Dedupe key is a hidden HTML comment embedded in the issue body:
#   <!-- ci-doctor:signature=<signature> -->
#
# Required env: GH_TOKEN (with issues:write on the repo).
#
# Required flags:
#   --signature <slug>            stable dedupe key, e.g. "clippy-stable"
#   --title "<text>"              issue title used when creating
#   --status pass|fail            outcome of the check
#   --command "<text>"            failing command, shown in body
#   --log <path>                  path to the captured log
#   --run-url <url>               link back to the GitHub Actions run
#   --label <name>                label to apply / filter by
#   --toolchain <name>            optional, informational only

set -euo pipefail

SIGNATURE=""
TITLE=""
STATUS=""
COMMAND=""
LOG_PATH=""
RUN_URL=""
LABEL=""
TOOLCHAIN=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --signature) SIGNATURE="$2"; shift 2 ;;
        --title) TITLE="$2"; shift 2 ;;
        --status) STATUS="$2"; shift 2 ;;
        --command) COMMAND="$2"; shift 2 ;;
        --log) LOG_PATH="$2"; shift 2 ;;
        --run-url) RUN_URL="$2"; shift 2 ;;
        --label) LABEL="$2"; shift 2 ;;
        --toolchain) TOOLCHAIN="$2"; shift 2 ;;
        *) echo "unknown flag: $1" >&2; exit 2 ;;
    esac
done

: "${SIGNATURE:?--signature is required}"
: "${STATUS:?--status is required}"
: "${LABEL:?--label is required}"

if [[ -z "${GH_TOKEN:-}" ]]; then
    echo "GH_TOKEN must be set" >&2
    exit 2
fi

MARKER="<!-- ci-doctor:signature=${SIGNATURE} -->"

# Ensure the label exists (idempotent).
gh label create "${LABEL}" \
    --description "Tracking issue opened by CI Doctor" \
    --color "d93f0b" \
    --force >/dev/null

# Find any existing open issue for this signature.
# The MARKER is embedded in the issue body when we create it; gh's default
# search covers title+body, so plain-text search on the marker is enough.
EXISTING_NUMBER="$(gh issue list \
    --state open \
    --label "${LABEL}" \
    --search "${MARKER}" \
    --json number \
    --jq '.[0].number // empty')"

case "${STATUS}" in
    pass|success)
        if [[ -n "${EXISTING_NUMBER}" ]]; then
            echo "Check ${SIGNATURE} passed — closing issue #${EXISTING_NUMBER}."
            gh issue comment "${EXISTING_NUMBER}" \
                --body "Resolved: the \`${SIGNATURE}\` check passed in run ${RUN_URL} on $(date -u +%Y-%m-%dT%H:%M:%SZ). Closing."
            gh issue close "${EXISTING_NUMBER}" --reason completed
        else
            echo "Check ${SIGNATURE} passed and no tracking issue exists — nothing to do."
        fi
        ;;
    fail|failure)
        if [[ -n "${EXISTING_NUMBER}" ]]; then
            echo "Check ${SIGNATURE} still failing — tracking issue #${EXISTING_NUMBER} already open. No-op."
            exit 0
        fi

        # Build the issue body. Trim the log to keep the issue readable.
        BODY_FILE="$(mktemp)"
        {
            echo "${MARKER}"
            echo
            echo "The scheduled **CI Doctor** run detected a failure on \`main\`."
            echo
            if [[ -n "${TOOLCHAIN}" ]]; then
                echo "- **Check:** \`${SIGNATURE}\` (toolchain: \`${TOOLCHAIN}\`)"
            else
                echo "- **Check:** \`${SIGNATURE}\`"
            fi
            echo "- **Run:** ${RUN_URL}"
            echo "- **Command:**"
            echo
            echo '  ```'
            echo "  ${COMMAND}"
            echo '  ```'
            echo
            echo "### Trimmed log"
            echo
            echo '```'
            if [[ -n "${LOG_PATH}" && -f "${LOG_PATH}" ]]; then
                # First and last 80 lines, since clippy/cargo-deny output can be long.
                LINE_COUNT="$(wc -l <"${LOG_PATH}" | tr -d ' ')"
                if [[ "${LINE_COUNT}" -le 200 ]]; then
                    cat "${LOG_PATH}"
                else
                    head -n 80 "${LOG_PATH}"
                    echo
                    echo "… [$(( LINE_COUNT - 160 )) lines omitted] …"
                    echo
                    tail -n 80 "${LOG_PATH}"
                fi
            else
                echo "(no log captured)"
            fi
            echo '```'
            echo
            echo "---"
            echo
            echo "This issue was opened automatically. It will be closed on the next successful"
            echo "run of the same check. To suppress CI Doctor entirely, set the repo variable"
            echo "\`CI_DOCTOR_ENABLED=false\`."
        } >"${BODY_FILE}"

        echo "Opening tracking issue for ${SIGNATURE}."
        gh issue create \
            --title "${TITLE}" \
            --label "${LABEL}" \
            --body-file "${BODY_FILE}"
        rm -f "${BODY_FILE}"
        ;;
    *)
        echo "Unknown --status value: ${STATUS}" >&2
        exit 2
        ;;
esac
