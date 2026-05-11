# Release Notes 0.32

OpenTelemetry Rust 0.32 continues to drive the Logs, Metrics, and Distributed
Tracing components forward. The Logs and Metrics API and SDK remain stable, with
no breaking changes in this release. The OTLP Exporters and the Distributed
Tracing API/SDK remain in pre-stable states (Release-Candidate and Beta
respectively), and this release introduces a small number of intentional
breaking changes in those areas to prepare them for stabilization.

For detailed changelogs of individual crates, please refer to their respective
changelog files. This document serves as a summary of the main changes.

## Key Changes

### Metrics SDK

1. **Bound instruments (experimental)**: Added `Counter::bind()` and
   `Histogram::bind()` returning pre-bound measurement handles
   (`BoundCounter<T>`, `BoundHistogram<T>`). Bound instruments resolve the
   attribute-to-aggregator mapping once at bind time and cache the result,
   eliminating per-call HashMap lookups on the hot path. Benchmarks show
   ~28x speedup for counter operations and ~9x for histograms. Gated behind
   the `experimental_metrics_bound_instruments` feature flag.

2. **Delta collection efficiency**: Delta metrics collection now uses in-place
   eviction instead of draining the HashMap on every collect cycle. Stale
   attribute sets that received no measurements since the last collection are
   evicted.

3. **Stable `Aggregation` API**: `Aggregation` and
   `StreamBuilder::with_aggregation()` are now stable and no longer require the
   `spec_unstable_metrics_views` feature flag.

### Logs

1. **Tracing-span attribute enrichment (experimental)**: The
   `opentelemetry-appender-tracing` crate can now copy attributes from active
   `tracing` spans onto each emitted log record. ("Span" here refers to
   `tracing::span!`, not an `opentelemetry::trace::Span`.) Enrichment is
   disabled by default with zero per-span overhead, and is gated behind the
   new `experimental_span_attributes` cargo feature.

2. **`spec_unstable_logs_enabled` removed**: The capability (and the backing
   specification) is now stable and is enabled by default. The feature flag
   has been removed.

### Distributed Tracing (Beta)

The Distributed Tracing API and SDK remain in beta. This release contains
intentional breaking changes to clean up the public surface ahead of
stabilization. In practice, most of these changes affect items that were only
used by the `tracing-opentelemetry` crate rather than by direct end-users of the
OpenTelemetry API. End-user impact is expected to be minimal.

- **Breaking** Removed several public fields and methods from `SpanBuilder`
  (`trace_id`, `span_id`, `end_time`, `status`, `sampling_result`, and their
  `with_*` counterparts).
- **Breaking** Moved SDK sampling types (`SamplingDecision`, `SamplingResult`)
  from `opentelemetry::trace` to `opentelemetry_sdk::trace` — these are SDK
  implementation details and should be imported from
  `opentelemetry_sdk::trace`.
- **Breaking** Removed public hidden methods from `SdkTracer` (`id_generator`,
  `should_sample`).
- **Breaking** `SpanExporter` trait methods (`shutdown`, `shutdown_with_timeout`,
  `force_flush`) now take `&self` instead of `&mut self`, for consistency with
  `LogExporter` and `PushMetricExporter`. Implementers using interior
  mutability require no changes.
- **Breaking** `InMemoryExporterError` has been removed in favor of
  `OTelSdkError`; a new `JaegerRemoteSamplerBuildError` replaces the last uses
  of `TraceError`.
- **Breaking** The SDK `testing` feature is now runtime-agnostic:
  `TokioSpanExporter` / `new_tokio_test_exporter` are renamed to
  `TestSpanExporter` / `new_test_exporter`, and several `tokio` transitive
  dependencies / features are removed.

Other tracing improvements:

- Fix panic when `SpanProcessor::on_end` calls `Context::current()`.

### OTLP Exporters (Release Candidate)

The OTLP Exporters remain in RC. This release introduces breaking changes to
prepare for stabilization, alongside several quality-of-life improvements:

- **Breaking** Removed `ExportConfig`, `HasExportConfig`, `with_export_config()`,
  `HasTonicConfig`, `HasHttpConfig`, `TonicConfig`, and `HttpConfig` from the
  public API. Use the public `WithExportConfig`, `WithTonicConfig`, and
  `WithHttpConfig` trait methods instead — these remain unchanged.
- The gRPC/tonic OTLP exporter now returns an error when an `https://` endpoint
  is configured but no TLS feature (`tls-ring` or `tls-aws-lc`) is enabled,
  instead of silently sending unencrypted traffic.
- New `tls-provider-agnostic` feature for environments that bring their own
  crypto backend (e.g. OpenSSL for FIPS compliance).
- New `build()` directly on `SpanExporterBuilder`, `MetricExporterBuilder`,
  and `LogExporterBuilder` (before selecting a transport), auto-selecting the
  transport based on `OTEL_EXPORTER_OTLP_PROTOCOL` or enabled features.
- Auth tokens / sensitive server responses no longer leak into export error
  messages — full details remain available at DEBUG level.
- Pre-flight transport errors (invalid URL, connect failure, DNS) for
  `grpc-tonic` are now surfaced at ERROR level instead of requiring DEBUG
  logging.

### Deprecations

- `opentelemetry-zipkin` is deprecated. Use the OTLP exporter
  (`opentelemetry-otlp`) — Zipkin supports native OTLP ingestion. The crate
  will be removed in a future release.
- `opentelemetry-jaeger-propagator` is deprecated. The Jaeger propagation
  format is deprecated per the OpenTelemetry specification. Use W3C
  TraceContext propagation instead. The crate will be removed in a future
  release.

## Next Release

In the [next release](https://github.com/open-telemetry/opentelemetry-rust/milestone/24),
the focus will shift to declaring the OTLP Exporters and the Distributed
Tracing API/SDK stable.

## Acknowledgments

Thank you to everyone who contributed to this milestone. We welcome your
feedback through GitHub issues or discussions in the [OTel-Rust Slack
channel](https://cloud-native.slack.com/archives/C03GDP0H023).
