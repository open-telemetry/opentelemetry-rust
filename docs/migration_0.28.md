# Migration guide from 0.27 to 0.28

OpenTelemetry Rust 0.28 introduces a large number of breaking changes that
impact all signals (logs/metrics/traces). This guide is intended to help with a
smooth migration for the common use cases of using `opentelemetry`,
`opentelemetry_sdk` `opentelemetry-otlp`, `opentelemetry-appender-tracing`
crates. The detailed changelog for each crate that you use can be consulted for
the full set of changes. This doc covers only the common scenario.

## Tracing Shutdown changes

`opentelemetry::global::shutdown_tracer_provider()` is removed. Now, you should
explicitly call shutdown() on the created tracer provider.

Before (0.27):

```rust
opentelemetry::global::shutdown_tracer_provider();
```

 • After (0.28):

```rust
let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
    .build();

// Clone and set the tracer provider globally. Retain the original to invoke shutdown later.
opentelemetry::global::set_tracer_provider(tracer_provider.clone());

// Shutdown the provider when application is exiting.
tracer_provider.shutdown();
```

This now makes shutdown consistent across signals.

## Rename SDK Structs

`LoggerProvider`, `TracerProvider` are renamed to `SdkLoggerProvider` and
`SdkTracerProvider` respectively. `MeterProvider` was already named
`SdkMeterProvider` and this now ensures consistency across signals.

### Async Runtime Requirements removed

When using OTLP Exporter for Logs, Traces a "batching" exporter is recommended.
Also, metrics always required a component named `PeriodicReader`. These
components previously needed user to pass in an async runtime and enable
appropriate feature flag depending on the runtime.

These components have been re-written to no longer require an async runtime.
Instead they operate by spawning dedicated background thread, and making
blocking calls from the same.

PeriodicReader, BatchSpanProcessor, BatchLogProcessor are the components
affected.

For Logs,Traces replace `.with_batch_exporter(exporter, runtime::Tokio)` with
`.with_batch_exporter(exporter)`. For Metrics, replace `let reader =
PeriodicReader::builder(exporter, runtime::Tokio).build();` with `let reader =
PeriodicReader::builder(exporter).build();` or more conveniently,
`.with_periodic_exporter(exporter)`.

Please note the following:

1. With the new approach, only the following grpc/http clients are supported in
   `opentelemetry-otlp`. grpc-tonic, reqwest-blocking-client In other words,
   `reqwest` and `hyper` are not supported. If using `grpc-tonic`, the OTLP
   Exporter must be created from within a Tokio runtime.
   If using exporters other than `opentelemetry-otlp`, consult the docs
   for the same to know if there are any restrictions/requirements on async
   runtime.
2. Timeout enforcement is now moved to Exporters. i.e
BatchProcessor,PeriodicReader does not enforce timeouts. For logs and traces,
`max_export_timeout` (on Processors) or `OTEL_BLRP_EXPORT_TIMEOUT` or
`OTEL_BSP_EXPORT_TIMEOUT` is no longer supported. For metrics, `with_timeout` on
PeriodicReader is no longer supported.

`OTEL_EXPORTER_OTLP_TIMEOUT` can be used to setup timeout for OTLP Exporters via
environment variables, or `.with_tonic().with_timeout()` or
`.with_http().with_timeout()` programmatically.

If you need the old behavior (your application cannot spawn a new thread, or
need to use another networking client etc.) use appropriate feature flags from
  below “experimental_metrics_periodicreader_with_async_runtime”
  "experimental_logs_batch_log_processor_with_async_runtime"
  "experimental_trace_batch_span_processor_with_async_runtime"

 **and** adjust the namespace:

 Example, when using Tokio runtime.

 ```rust
let reader = opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader::builder(exporter, runtime::Tokio).build();
let tracer_provider = SdkTracerProvider::builder()
        .with_span_processor(span_processor_with_async_runtime::BatchSpanProcessor::builder(exporter, runtime::Tokio).build())
        .build();
let logger_provider = SdkLoggerProvider::builder()
.with_log_processor(log_processor_with_async_runtime::BatchLogProcessor::builder(exporter, runtime::Tokio).build())
.build();
```

## OTLP Default change

"grpc-tonic" feature flag is no longer enabled by default. "http-proto" and
  "reqwest-blocking-client" features are added as default, to align with the
  OTel specification.

## Resource Changes

`Resource` creation is moved to a builder pattern, and `Resource::{new, empty,
from_detectors, new_with_defaults, from_schema_url, merge, default}` are
replaced with `Resource::builder()`.

Before:

```rust
Resource::default().with_attributes([
    KeyValue::new("service.name", "test_service"),
    KeyValue::new("key", "value"),
]);
```

After:

```rust
Resource::builder()
    .with_service_name("test_service")
    .with_attribute(KeyValue::new("key", "value"))
    .build();
```

## Improved internal logging

OpenTelemetry internally used `tracing` to emit its internal logs. This is under
feature-flag "internal-logs" that is enabled by default in all crates. When
using OTel Logging, care must be taken to avoid OTel's own internal log being
fed back to OTel, creating an infinite loop. This can be achieved via proper
filtering. The OTLP Examples in the repo shows how to achieve this. It also
shows how to send OTel's internal logs to stdtout using `tracing::Fmt`.

## Full example

A fully runnable example application using OTLP Exporter is provided in this
repo. Comparing the 0.27 vs 0.28 of the example would give a good overview of
the changes required to be made.

[Basic OTLP Example
(0.27)](https://github.com/open-telemetry/opentelemetry-rust/tree/opentelemetry-otlp-0.27.0/opentelemetry-otlp/examples)
[Basic OTLP Example
(0.28)](https://github.com/open-telemetry/opentelemetry-rust/tree/opentelemetry-otlp-0.27.0/opentelemetry-otlp/examples)
// TODO: Update this link after github tag is created.

This guide covers only the most common breaking changes. If you’re using custom
exporters or processors (or authoring one), please consult the changelog for
additional migration details.
