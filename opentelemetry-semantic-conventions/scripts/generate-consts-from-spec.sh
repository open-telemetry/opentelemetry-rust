#!/bin/bash
# Semantic conventions are currently defined in Markdown files in the
# specification repository. This script downloads these files and attempts to
# find the attributes and their documentation. It then prints Rust code with
# constants for these attributes to stdout.
#
# This helps you get started but doesn't do all the things. You will need to the
# following manually:
# - This prints some attributes multiple times, because it mistakes examples for
#   attribute definitions. Remove those.
# - In some cases the description isn't parsed properly. In those cases you will
#   need to copy it manually.
# - Hard wrap description text, so it fits the code style of this repo.
# - Add URLs for links in the documentation.

set -e

md_to_rs() {
	while read line; do
		# Regex explanation:
		# Find markdown tables (lines starting with "|") and take the first 2 columns
		# - The 1st column is the attribute name consisting of a-z0-9._
		# - The 2nd column is the description
		if [[ "$line" =~ ^\|\ \[?\`?([a-z0-9._]+)\`?\]?(:?\([^|]+\))?\ +\|\ ([^|]+)\ +\|\ ([^|]+) ]]; then
			name="${BASH_REMATCH[1]}"
			const_name=$(echo "$name" | tr a-z A-Z | sed "s/\\./_/g")
			doc="${BASH_REMATCH[4]}"
			echo ""
			echo "/// $doc"
			echo "pub const $const_name: Key = Key::from_static_str(\"$name\");"
		fi
	done <<<"$1"
}

md_url_to_rs() {
	md_to_rs "$(curl -sSf "https://raw.githubusercontent.com/open-telemetry/opentelemetry-specification/master/specification/$1")"
}

case "$1" in
	"resource")
		md_url_to_rs "resource/semantic_conventions/README.md"
		md_url_to_rs "resource/semantic_conventions/cloud.md"
		md_url_to_rs "resource/semantic_conventions/container.md"
		md_url_to_rs "resource/semantic_conventions/deployment_environment.md"
		md_url_to_rs "resource/semantic_conventions/faas.md"
		md_url_to_rs "resource/semantic_conventions/host.md"
		md_url_to_rs "resource/semantic_conventions/k8s.md"
		md_url_to_rs "resource/semantic_conventions/os.md"
		md_url_to_rs "resource/semantic_conventions/process.md"
		;;
	"trace")
		md_url_to_rs "trace/semantic_conventions/database.md"
		md_url_to_rs "trace/semantic_conventions/exceptions.md"
		md_url_to_rs "trace/semantic_conventions/faas.md"
		md_url_to_rs "trace/semantic_conventions/http.md"
		md_url_to_rs "trace/semantic_conventions/messaging.md"
		md_url_to_rs "trace/semantic_conventions/rpc.md"
		md_url_to_rs "trace/semantic_conventions/span-general.md"
		;;
	*)
		echo "Usage: $0 <resource|trace>"
		;;
esac
