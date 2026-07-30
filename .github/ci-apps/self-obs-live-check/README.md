# self-obs-live-check

This is **not** a user-facing example. It exists solely to support the
`sdk-self-observability` CI workflow (`.github/workflows/sdk-self-observability.yml`).

## What it does

Runs a minimal OpenTelemetry SDK pipeline that drives the SDK's own
`otel.sdk.*` self-observability metrics and validates them against the upstream
semantic conventions:

1. Creates a `MeterProvider` with an OTLP/gRPC exporter pointing at
   [weaver](https://github.com/open-telemetry/weaver) (started by the CI workflow).
   This single exporter collects every `otel.sdk.*` metric the SDK emits,
   regardless of which component produced it.
2. Exercises SDK components (log processors today, more over time) so they emit
   their self-observability metrics, then waits for the periodic export and
   shuts down to flush.

Weaver's `registry live-check` validates the exported metrics against the
upstream [semantic conventions](https://opentelemetry.io/docs/specs/semconv/otel/sdk-metrics/),
catching attribute/unit/naming violations before they reach users.

## Adding coverage for a new metric

Extend `src/main.rs` to exercise the SDK component that emits the metric (for a
new processor variant, add it to the existing provider; for a new signal, add
the corresponding provider). The metric flows to weaver through the existing
metric exporter automatically. Intentionally avoids enumerating individual
metric names here so this doc does not drift as coverage grows.
