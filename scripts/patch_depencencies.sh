# Dashmap 5.3.4 requires 1.59
latest_version=$(cargo search --limit 1 dashmap | head -1 | cut -d'"' -f2)  &&
cargo update -p dashmap:$latest_version --precise 5.1.0 &&
# We have time 0.1 and 0.3
latest_version=$(cargo search --limit 1 time | head -1 | cut -d'"' -f2) &&
cargo update -p time:$latest_version --precise 0.3.9
