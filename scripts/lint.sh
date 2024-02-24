#!/bin/bash

set -eu

cargo_feature() {
    echo "checking $1 with features $2"
    cargo clippy --manifest-path=$1/Cargo.toml --all-targets --features "$2" --no-default-features -- \
    `# Exit with a nonzero code if there are clippy warnings` \
    -Dwarnings
}

if rustup component add clippy; then
 crates=( "opentelemetry"
                "opentelemetry-http"
                "opentelemetry-jaeger"
                "opentelemetry-jaeger-propagator"
                "opentelemetry-appender-log"
                "opentelemetry-appender-tracing"
                "opentelemetry-otlp"
                "opentelemetry-prometheus"
                "opentelemetry-proto"
                "opentelemetry-sdk"
                "opentelemetry-semantic-conventions"
                "opentelemetry-stdout"
                "opentelemetry-zipkin")
  for create in "${creates[@]}"; do
      cargo clippy --manifest-path=$create/Cargo.toml --all-targets --all-features -- \
          `# Exit with a nonzero code if there are clippy warnings` \
          -Dwarnings
  done

  cargo_feature opentelemetry "trace,metrics,logs,logs_level_enabled,testing"

  cargo_feature opentelemetry-otlp "default"
  cargo_feature opentelemetry-otlp "default,tls"
  cargo_feature opentelemetry-otlp "default,tls-roots"
  cargo_feature opentelemetry-otlp "http-proto"
  cargo_feature opentelemetry-otlp "http-proto, reqwest-blocking-client"
  cargo_feature opentelemetry-otlp "http-proto, reqwest-client"
  cargo_feature opentelemetry-otlp "http-proto, reqwest-rustls"
  cargo_feature opentelemetry-otlp "metrics"

  cargo_feature opentelemetry-jaeger "isahc_collector_client"
  cargo_feature opentelemetry-jaeger "reqwest_blocking_collector_client"
  cargo_feature opentelemetry-jaeger "reqwest_collector_client"
  cargo_feature opentelemetry-jaeger "hyper_collector_client"
  cargo_feature opentelemetry-jaeger "hyper_tls_collector_client"
  cargo_feature opentelemetry-jaeger "collector_client"
  cargo_feature opentelemetry-jaeger "wasm_collector_client"
  cargo_feature opentelemetry-jaeger "collector_client, wasm_collector_client"
  cargo_feature opentelemetry-jaeger "default"

  cargo_feature opentelemetry-jaeger-propagator "default"

  cargo_feature opentelemetry-proto "default"
  cargo_feature opentelemetry-proto "full"
  cargo_feature opentelemetry-proto "gen-tonic,trace"
  cargo_feature opentelemetry-proto "gen-tonic,trace,with-serde"
  cargo_feature opentelemetry-proto "gen-tonic,trace,with-schemars,with-serde"
  cargo_feature opentelemetry-proto "gen-tonic,metrics"
  cargo_feature opentelemetry-proto "gen-tonic,logs"

fi
