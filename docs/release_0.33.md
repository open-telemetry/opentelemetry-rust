# Release Notes 0.33

The release-candidate (RC) period for OpenTelemetry Rust 0.33 applies to the
OTLP exporters for Logs and Metrics. We intend to keep these exporters in RC
for two weeks before publishing the final release. It does not change the
stability status of the other components in this release.

OpenTelemetry Rust 0.33 declares the **OTLP exporters for Logs and Metrics
stable**, brings OTLP retries on by default, and completes the stabilisation
cleanups that began in 0.32. The Logs and Metrics API and SDK remain stable with
no breaking changes. The Distributed Tracing API, SDK, and OTLP exporter remain
pre-stable, and this release includes intentional breaking changes in that area
to continue preparing it for stabilisation.

For detailed changelogs of individual crates, refer to their respective
changelog files. This document summarises the key changes.

## Key Changes

### OTLP Exporters for Logs and Metrics (Stable)

With this release, the OTLP exporters for Logs and Metrics are declared
**stable**. The stabilisation work that began in 0.32 is now complete, and their
public API surface is considered ready for production use with semver
guarantees.

#### Retries enabled by default

OTLP/HTTP and OTLP/gRPC retries are now enabled by default with exponential
backoff, jitter, and up to 3 retries (4 attempts total). Use
`.with_retry_policy(RetryPolicy::disabled())` to opt out, or provide a custom
`RetryPolicy`.

Users of the experimental retry feature flags (`grpc-tonic-with-retry`,
`http-proto-with-retry`, `http-json-with-retry`) should remove them from
`Cargo.toml`. The `retry` and `retry_classification` modules are now
crate-private; replace `opentelemetry_otlp::retry` imports with
`opentelemetry_otlp::RetryPolicy` and use its `with_*` methods instead of
struct literals.

#### Stabilisation cleanups

- **Breaking** `Protocol` and `Compression` are now `#[non_exhaustive]`.
  Exhaustive matches must add a wildcard arm.
- **Breaking** `Protocol::from_env()` is now crate-private. Builders resolve
  `OTEL_EXPORTER_OTLP_PROTOCOL` automatically; read the env var directly if
  needed.
- Invalid OTLP endpoint environment variables (HTTP and gRPC) now produce a
  build error instead of silently falling back to localhost. Empty values are
  treated as unset.
- **Breaking** `WithHttpConfig::with_max_request_body_size` is now a required
  trait method. OTLP/HTTP request bodies are limited to 64 MiB by default
  (before and after compression).
- **Breaking** Removed the `reqwest-rustls-webpki-roots` feature (broken since
  reqwest 0.13.0). Use `reqwest-rustls` instead.

#### gRPC INSECURE environment variables

Added spec-compliant support for `OTEL_EXPORTER_OTLP_INSECURE` and its
signal-specific variants (`_TRACES_INSECURE`, `_METRICS_INSECURE`,
`_LOGS_INSECURE`). **Breaking:** schemeless endpoints (e.g.
`collector.example.com:4317`) now default to `https://` instead of being passed
as-is. Set `OTEL_EXPORTER_OTLP_INSECURE=true` for plaintext connections.
Endpoints with an explicit scheme are unaffected.

### Metrics SDK

1. **Bound instruments completed**: `BoundGauge<T>` and
   `BoundUpDownCounter<T>` complete the experimental bound-instrument API
   across all sync instruments. Gated behind the
   `experimental_metrics_bound_instruments` feature flag.

2. **SDK self-observability metrics** (experimental): `otel.sdk.log.created`,
   `otel.sdk.processor.log.processed`, `otel.sdk.processor.span.processed`,
   and `otel.sdk.processor.log.queue.capacity` are now available behind the
   `experimental_metrics_bound_instruments` feature flag.

3. **Minimal SDK build**: `futures-channel`, `futures-executor`, `futures-util`,
   and `thiserror` are now optional. With `default-features = false` the SDK's
   only dependency is the `opentelemetry` API crate.

### Distributed Tracing API

- `TraceState` now enforces the W3C 32 list-member limit. `from_str`,
  `from_key_value`, and `insert` drop excess members from the end.

### opentelemetry-http

- **Breaking** Removed the deprecated `HttpClient::send` method (accepting
  `Request<Vec<u8>>`). Use `HttpClient::send_bytes` instead.
- Built-in HTTP clients now limit response body reads to 4 MiB.
- Built-in reqwest and hyper clients return HTTP error responses instead of
  converting 4xx/5xx to transport errors, preserving status and headers for
  retry classification.
- **Breaking** Removed `reqwest-rustls-webpki-roots` feature (same as OTLP).

### opentelemetry-prometheus

- Replaced `without_scope_info` with `scope_info_enabled`, inverting the
  option. Scope labels (`otel_scope_name`, `otel_scope_version`,
  `otel_scope_schema_url`, and scope-attribute prefixed labels) are now
  enabled by default on metric points.

### opentelemetry-jaeger-propagator

- **Removed.** The deprecated `opentelemetry-jaeger-propagator` crate has been
  removed. The Jaeger propagation format is deprecated per the OpenTelemetry
  specification; use W3C TraceContext propagation instead.

## Next Release

With the OTLP exporters for Logs and Metrics now stable, the next release will
focus on declaring the Distributed Tracing API, SDK, and OTLP exporter stable.

## Acknowledgments

Thank you to everyone who contributed to this milestone. We welcome your
feedback through GitHub issues or discussions in the [OTel-Rust Slack
channel](https://cloud-native.slack.com/archives/C03GDP0H023).
