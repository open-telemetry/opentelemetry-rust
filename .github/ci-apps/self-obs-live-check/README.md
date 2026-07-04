# self-obs-live-check

This is **not** a user-facing example. It exists solely to support the
`sdk-self-observability` CI workflow (`.github/workflows/sdk-self-observability.yml`).

## What it does

Runs a minimal OpenTelemetry SDK pipeline that exercises the
`otel.sdk.processor.log.processed` self-observability metric:

1. Creates a `MeterProvider` with an OTLP/gRPC exporter pointing at
   [weaver](https://github.com/open-telemetry/weaver) (started by the CI workflow).
2. Creates a `LoggerProvider` with a `BatchLogProcessor` (in-memory log exporter).
3. Emits log records — the batch processor increments `otel.sdk.processor.log.processed`.
4. Waits for the periodic metric export, then shuts down.

Weaver's `registry live-check` validates the exported metrics against the
upstream [semantic conventions](https://opentelemetry.io/docs/specs/semconv/otel/sdk-metrics/),
catching attribute/unit/naming violations before they reach users.
