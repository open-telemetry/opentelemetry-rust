# SDK Self-Diagnostics via Metrics

Status:
[Development](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/document-status.md)

The OpenTelemetry Rust SDK can emit metrics about its own internal state,
following the [semantic conventions for SDK metrics](https://github.com/open-telemetry/semantic-conventions/blob/main/docs/otel/sdk-metrics.md).

## Implemented Metrics

### `otel.sdk.processor.log.processed`

- **Instrument**: `Counter<u64>`
- **Unit**: `{log_record}`
- **Description**: The number of log records for which processing has finished,
  either successful or failed.
- **Component**: `BatchLogProcessor`

**Attributes:**

| Attribute | Value |
|-----------|-------|
| `otel.component.type` | `batching_log_processor` |
| `otel.component.name` | `batching_log_processor/{id}` (auto-assigned) |
| `error.type` | `queue_full` when dropped due to full queue; `already_shutdown` when emitted after shutdown. Absent on success. |

The counter is incremented on every `emit()` call: once for successful
enqueue, once with `error.type=queue_full` when dropped due to a full queue,
and once with `error.type=already_shutdown` when emitted after the processor
has been shut down.

## Feature Gate

Self-diagnostics metrics require the `experimental_metrics_bound_instruments`
feature on `opentelemetry_sdk`. This feature is not enabled by default.

Without bound instruments, every `Counter::add()` call would need to resolve
attributes to the internal aggregation state — roughly 50 ns per call on the
`emit()` hot path. With bound instruments, attributes are resolved once at
construction and subsequent `add()` calls are a single atomic increment at
~1.8 ns. Since `emit()` is called for every log record in the application,
this overhead matters. Bound instruments are what make self-diagnostics
practical without measurable performance impact.

## Provider Initialization Order

The `BatchLogProcessor` obtains a `Meter` from the global `MeterProvider`
during construction. Rust's `global::meter()` returns a snapshot — it does
**not** retroactively upgrade if the global provider changes later.

For self-diagnostics to produce real data, the global `MeterProvider` must be
set **before** creating the `LoggerProvider` (and its `BatchLogProcessor`).
The recommended setup order is:

```rust
// 1. MeterProvider first. Optionally set up a throwaway thread-local fmt
//    subscriber so that any internal logs emitted during MeterProvider
//    construction still appear on stdout. This is not required — without it
//    those few startup-time debug messages are simply lost.
let meter_provider = {
    let _guard = tracing::subscriber::set_default(
        tracing_subscriber::fmt().with_env_filter("info").finish(),
    );
    let mp = init_metrics();
    global::set_meter_provider(mp.clone());
    mp
}; // _guard drops here, removing the throwaway subscriber

// 2. LoggerProvider second (BatchLogProcessor picks up the real meter)
let logger_provider = init_logs();

// 3. Full tracing subscriber (fmt + OTel bridge)
tracing_subscriber::registry()
    .with(OpenTelemetryTracingBridge::new(&logger_provider))
    .with(fmt::layer())
    .init();

// 4. TracerProvider last (init logs captured by OTel pipeline)
let tracer_provider = init_traces();
global::set_tracer_provider(tracer_provider.clone());
```

See the OTLP examples (`basic-otlp`, `basic-otlp-http`) for the full pattern
including a temporary thread-local `fmt` subscriber during MeterProvider setup.

If the `MeterProvider` is set after the `LoggerProvider`, the counter will be
backed by a no-op meter and silently produce nothing. This is harmless but
means no self-diagnostics data.

## TODO

- Emit `otel.sdk.processor.log.queue.size` (current queue depth).
- Emit `otel.sdk.processor.log.queue.capacity` (configured max queue size).
- Emit `otel.sdk.exporter.log.exported` from log exporters.
- Add self-diagnostics to `BatchSpanProcessor`.
- Add self-diagnostics to `SimpleLogProcessor`.
- Record logs lost when `shutdown_with_timeout` times out (the background
  thread may still hold unfinished exports and queued records).
- Long-term: `global::meter()` currently returns a snapshot that does not
  reflect later calls to `set_meter_provider()`. Investigate whether the
  global meter can be made to pick up provider changes after the fact.
