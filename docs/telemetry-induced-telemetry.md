# Preventing telemetry-induced telemetry with OTLP

OTLP exporters use networking libraries such as `reqwest`, `hyper`, `h2`, and
`tonic`. These libraries can emit their own `tracing` events while exporting
telemetry. If an application sends those events through
`OpenTelemetryTracingBridge` to the same OTLP exporter, an export can generate
another export repeatedly. This feedback is called
**telemetry-induced telemetry**.

OpenTelemetry suppresses its own internal telemetry while the standard batch
processors invoke exporters. That suppression is stored in the current
OpenTelemetry `Context`. It does not automatically cross every thread and task
boundary used by external networking libraries.

Whether feedback occurs depends on the application's subscriber configuration.
The relevant transport events are commonly emitted at `DEBUG` or `TRACE`.
Applications that export only `INFO` and higher may not observe a problem, but
should not rely on transport libraries retaining their current levels.

The feedback may cross signals. For example, exporting a trace can cause the
HTTP or gRPC client to emit a `tracing` event, which
`OpenTelemetryTracingBridge` turns into an OpenTelemetry log. Exporting that
log can then produce another event. The advice below therefore applies when
exporting logs, traces, metrics, or any combination of them.

## What should I do?

**For most applications, filter transport-library events out of the
OpenTelemetry layer.** This is the lowest-cost solution: it requires no extra
threads, no experimental processors, and no exporter changes. The OTLP
examples in this repository already use this approach.

Use the dedicated-runtime approach only when the simple filter is insufficient,
usually because the application needs to send `reqwest`, `hyper`, `h2`, or
`tonic` telemetry from its normal traffic to the backend. It provides more
selective isolation and does not depend as heavily on a fixed list of targets,
but it adds runtime resources and experimental APIs. It is not automatically
better for every application.

| Situation | Suggested approach |
|---|---|
| Transport diagnostics do not need to be exported | Filter the OpenTelemetry layer |
| Transport diagnostics are needed only in local output | Filter the OpenTelemetry layer, but allow them in the formatting layer |
| Application transport diagnostics must be exported | Consider a dedicated suppressed runtime |
| The application uses blocking reqwest for OTLP | Use filtering, or switch OTLP to async reqwest before considering a dedicated runtime |
| The application installs `OpenTelemetryLogBridge` as its `log` logger | Keep the global level at `INFO`, or wrap the bridge with target filtering |
| A custom HTTP client starts threads or tasks internally | Start with filtering; runtime isolation works only if the application controls those execution contexts |

The built-in transport options have the following support:

| OTLP transport | Layer filtering | Dedicated suppressed runtime |
|---|---|---|
| HTTP/protobuf or HTTP/JSON with `reqwest-blocking-client` | Yes | No |
| HTTP/protobuf or HTTP/JSON with `reqwest-client` | Yes | Yes |
| HTTP/protobuf or HTTP/JSON with `hyper-client` | Yes | Yes |
| gRPC with `grpc-tonic` | Yes | Yes |
| Custom `HttpClient` | Yes | Only when its execution threads and tasks are controlled |

## Recommended solution: filter the OpenTelemetry layer

Apply a filter to the layer that forwards `tracing` events to OpenTelemetry.
This prevents exporter transport events from reaching the OTLP log pipeline
while allowing a separate formatting layer to display them locally.

```rust
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

fn install_subscriber(logger_provider: &SdkLoggerProvider) {
    let otel_filter = EnvFilter::new("info")
        .add_directive("reqwest=off".parse().expect("valid directive"))
        .add_directive("hyper=off".parse().expect("valid directive"))
        .add_directive("h2=off".parse().expect("valid directive"))
        .add_directive("tonic=off".parse().expect("valid directive"))
        .add_directive("tower=off".parse().expect("valid directive"))
        .add_directive("want=off".parse().expect("valid directive"))
        .add_directive("rustls=off".parse().expect("valid directive"))
        .add_directive("native_tls=off".parse().expect("valid directive"))
        .add_directive("tokio_native_tls=off".parse().expect("valid directive"));

    let otel_layer =
        OpenTelemetryTracingBridge::new(logger_provider).with_filter(otel_filter);

    // This independent layer displays transport diagnostics locally without
    // sending them through OpenTelemetry again.
    let fmt_filter = EnvFilter::new(
        "info,reqwest=debug,hyper=debug,h2=debug,tonic=debug,tower=debug,\
         want=debug,rustls=debug,native_tls=debug,tokio_native_tls=debug",
    );
    let fmt_layer = tracing_subscriber::fmt::layer().with_filter(fmt_filter);

    tracing_subscriber::registry()
        .with(otel_layer)
        .with(fmt_layer)
        .init();
}
```

Choose the base level according to which application events should be
exported. For example, use `debug` instead of `info` while retaining the
transport-specific `off` directives if application debug events should be sent
to OpenTelemetry. Recheck the exported targets when lowering this level or
changing the HTTP, TLS, or gRPC stack; future dependencies may emit telemetry
under additional targets.

`EnvFilter` target directives use prefix matching, so `hyper=off` also covers
targets such as `hyper_util`. If configuration starts with
`EnvFilter::from_default_env()`, append the `off` directives afterward so a
broad setting such as `RUST_LOG=debug` does not re-enable transport targets in
the OpenTelemetry layer.

The tradeoff is that the OpenTelemetry layer drops matching transport events
even when those libraries are used for non-exporter application traffic. The
independent formatting layer can still record them locally.

> **Filtering is not fully future-proof.** Target names are implementation
> details of the current transport stack. Dependency upgrades can rename
> targets, add new instrumented components, or change event levels. Recheck
> exported records after upgrading `reqwest`, `hyper`, `h2`, `tonic`, `tower`,
> TLS implementations, or the OTLP exporter, and monitor for exports that
> continue while the application is otherwise idle.

### Applications using the `log` crate directly

Some transport and TLS libraries emit records through the `log` facade. The
subscriber setup above handles them when `tracing-log` forwards those records
into `tracing`.

Applications that install `OpenTelemetryLogBridge` directly as the global
`log` logger do not pass records through the `tracing` layer filter. Keep the
global `log` level at `INFO`, or place an equivalent target-filtering `log`
implementation in front of the OpenTelemetry bridge. The bridge does not
currently provide its own per-target filter.

See the filter setup in the
[`basic-otlp`](../opentelemetry-otlp/examples/basic-otlp/src/main.rs) and
[`basic-otlp-http`](../opentelemetry-otlp/examples/basic-otlp-http/src/main.rs)
examples. The examples use a shorter target list together with an `info` base
level. The broader list above is intended for applications that enable verbose
dependency diagnostics.

## Advanced solution: a dedicated suppressed runtime

Applications that must export dependency telemetry from normal application
traffic can isolate exporter traffic on a dedicated Tokio runtime. Suppression
is installed on that runtime's worker threads, so events emitted while driving
the exporter are ignored by the OpenTelemetry SDK. The application's regular
runtime remains unsuppressed.

This is a more selective solution than target filtering, but it is also
substantially more complex. Do not add a dedicated runtime merely because the
application uses OTLP. Use it when an observed feedback problem cannot be
handled by filtering, or when filtering would remove dependency telemetry that
the application must export.

This approach requires an async transport:

- `reqwest-client` or `hyper-client` for OTLP/HTTP.
- `grpc-tonic` for OTLP/gRPC.

Async HTTP clients must be paired with the experimental async-runtime
processors and readers. They are not compatible with the standard
thread-based processors and readers.

It does not work with `reqwest-blocking-client`. The blocking client owns a
separate internal runtime thread and does not expose a hook for installing
OpenTelemetry suppression there.

> **Important:** The experimental async-runtime processors do not install
> telemetry suppression themselves. The runtime thread hooks below are
> required. Without them, both transport diagnostics and the exporter's own
> internal logs can feed back into the pipeline.

A compiled example shows the pattern for OTLP logs over async reqwest. See
[`telemetry-suppression`](../opentelemetry-otlp/examples/telemetry-suppression/src/main.rs)
example and its
[`Cargo.toml`](../opentelemetry-otlp/examples/telemetry-suppression/Cargo.toml).
It disables the OTLP crate's default features, selects `reqwest-client`, builds
the exporter and async processor while the dedicated runtime is entered, and
shuts down the provider before stopping that runtime.

Equivalent experimental async-runtime processors exist for traces and metrics.
For traces, use
`trace::span_processor_with_async_runtime::BatchSpanProcessor` with
`experimental_trace_batch_span_processor_with_async_runtime`. For metrics, use
`metrics::periodic_reader_with_async_runtime::PeriodicReader` with
`experimental_metrics_periodicreader_with_async_runtime`. Construct every
exporter and processor while the dedicated runtime is entered, and keep the
runtime alive until every provider has completed shutdown.

Do not schedule application work on the export runtime. Suppression applies to
all of its worker and blocking threads, which also covers work such as DNS
resolution. Application telemetry generated there will be intentionally
dropped.

Provider shutdown waits synchronously for processor work. Invoke it from a
synchronous shutdown path or use `tokio::task::spawn_blocking` from an
asynchronous application. Never invoke it from a task running on the export
runtime because it would wait for work scheduled on the same runtime.

The async-runtime processors are experimental and may change without a
major-version release. A dedicated runtime also adds a thread and runtime
resources, so prefer layer filtering unless selective preservation of
dependency telemetry is required. A Tokio `Runtime` must not be dropped from
inside an asynchronous context; use `shutdown_background`, or move ownership
to a dedicated OS thread and shut it down there.

Context suppression is checked by OpenTelemetry log and trace creation. Metrics
recording does not currently check this context flag. If an exporter dependency
emits OpenTelemetry metrics directly, use filtering or another mechanism that
prevents those metrics from entering the export pipeline.

## `tracing-opentelemetry` spans

The runtime approach suppresses telemetry that honors the current
OpenTelemetry `Context`. A `tracing` span can explicitly request
`parent: None`, causing some versions of `tracing-opentelemetry` to translate
it using a fresh OpenTelemetry context. For example, `h2` creates its
connection-lifetime span this way.

If the application converts `tracing` spans to OpenTelemetry spans, use a
per-layer target filter as well. Filtering occurs before translation and does
not depend on the span's parent context.

The runtime experiments described above exercised `tracing` events sent to
OpenTelemetry Logs through `OpenTelemetryTracingBridge`. They did not exercise
span conversion through `tracing-opentelemetry`.

## Diagnosing a feedback loop

Common symptoms include:

- Repeated OTLP exports when the application is otherwise idle.
- Exported log records whose targets are `reqwest`, `hyper`, `hyper_util`,
  `h2`, `tonic`, `tower`, or `want`.
- Request volume increasing after enabling `DEBUG` or `TRACE` for the
  OpenTelemetry bridge.
- Export batches containing only networking diagnostics generated by the
  preceding export.

Apply the filter to the OpenTelemetry layer rather than globally whenever
possible. A global filter can hide transport diagnostics from local logs as
well.

See [issue #2877] for ongoing discussion.

[issue #2877]: https://github.com/open-telemetry/opentelemetry-rust/issues/2877
