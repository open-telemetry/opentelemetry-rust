# OpenTelemetry Rust Logs

Status: **Stable**

## Introduction

This document provides guidance on leveraging OpenTelemetry logs in Rust
applications.

In short: for application logging, use [`tracing`] with the
[`opentelemetry-appender-tracing`] appender (not [`tracing-opentelemetry`],
which bridges *spans*, not logs — see [traces.md](traces.md)). If you have
an existing codebase using the [`log`] crate, keep using it and bridge to
OpenTelemetry via [`opentelemetry-appender-log`]; but for **new code, use
`tracing`**, which supports structured logging and is what OpenTelemetry
itself uses internally. Adopting OpenTelemetry for logs is primarily a setup
change, not a code rewrite. For span guidance, see [traces.md](traces.md).

[`tracing`]: https://crates.io/crates/tracing
[`tracing-opentelemetry`]: https://crates.io/crates/tracing-opentelemetry
[`log`]: https://crates.io/crates/log
[`opentelemetry-appender-tracing`]: ../opentelemetry-appender-tracing/README.md
[`opentelemetry-appender-log`]: ../opentelemetry-appender-log/README.md

## Avoiding OTLP transport feedback

OTLP exporters use libraries such as `reqwest`, `hyper`, `h2`, and `tonic`.
When dependency logs at `DEBUG` or `TRACE` are sent through
`OpenTelemetryTracingBridge`, an export can generate more logs and repeatedly
trigger another export.

Filter these transport targets only on the OpenTelemetry layer:

```rust
let otel_filter =
    EnvFilter::new("info,reqwest=off,hyper=off,h2=off,tonic=off");
let otel_layer =
    OpenTelemetryTracingBridge::new(&logger_provider).with_filter(otel_filter);

tracing_subscriber::registry()
    .with(otel_layer)
    .with(tracing_subscriber::fmt::layer())
    .init();
```

This is not a global filter. It only prevents matching events from being
exported through the OpenTelemetry layer. The same events remain available to
`fmt` and every other subscriber layer according to each layer's own filter.
This is sufficient for almost all applications, including troubleshooting
transport problems through local output.

A target filter is insufficient only when transport logs from normal
application traffic must be exported to the OpenTelemetry backend while
exporter-generated transport logs are excluded. That advanced case needs more
selective isolation, such as running async exporter work on a dedicated
suppressed runtime; see [issue #2877]. Applications using
`OpenTelemetryLogBridge` directly must instead apply equivalent filtering before
records reach that bridge.

[issue #2877]: https://github.com/open-telemetry/opentelemetry-rust/issues/2877

## Filtering log records

As a general rule, filter at the earliest layer that has both the information
and policy control needed for the decision:

- **Logging library:** Use this layer when the condition is available at the
  callsite, such as severity, target or module, event name, or other logging
  metadata. Filtering before the OpenTelemetry bridge avoids creating an SDK
  log record and performing bridge conversion, SDK processing, batching,
  serialization, and transport for rejected events. `tracing` offers flexible
  [filtering capabilities] through `tracing-subscriber`, including level and
  target filters, `EnvFilter`, custom predicates, and per-layer filters.
- **SDK processor:** Use a custom [`LogProcessor`] when the condition requires
  the normalized `SdkLogRecord`, instrumentation scope, or application-local
  state; when one OpenTelemetry-level policy must cover multiple logging
  libraries, bridges, or third-party instrumentation; or when export
  destinations require different rules. Record creation and bridge conversion
  have already occurred, but rejected records can still be kept out of batching
  and export. The processor implementation is maintained in the application.
- **Collector or telemetry pipeline:** Use this layer when policy must be
  managed centrally across services, changed without redeploying applications,
  or evaluated using information added by pipeline processors, such as
  Kubernetes metadata. This still incurs the application-side work and
  transport cost of sending records to that point in the pipeline.

Filter sensitive data before the process, host, or other trust boundary that it
must not cross. Filtering downstream cannot undo exposure across that boundary.

If you have a reason to filter in an SDK processor, the following approach
shows how to compose it with other processors.

`SdkLoggerProvider` delivers records independently to every processor registered
with it. Therefore, a filtering processor must wrap the processor that exports
or batches records:

```text
SdkLoggerProvider
└── FilteringLogProcessor
    └── BatchLogProcessor
        └── Exporter
```

In its `emit` method, the filter evaluates the condition and calls the wrapped
processor only when the record should be kept. Register only the wrapper for
that export branch. Registering the processors as siblings does not form a
pipeline, regardless of registration order:

```text
SdkLoggerProvider
├── FilteringLogProcessor
└── BatchLogProcessor
    └── Exporter
```

In the sibling configuration, both processors receive the record. A filter
controls only the processor it wraps, so wrap each export branch that requires
filtering.

The runnable [logs-advanced example] uses an attribute value as its filtering
condition. A processor could instead filter by severity, event name, scope,
body, external configuration, or any other information available to it. The
example also demonstrates forwarding processor lifecycle methods and composing
the filter with `SimpleLogProcessor`; a production application can wrap
`BatchLogProcessor` in the same way.

`LogProcessor::event_enabled` can reject records earlier, but it receives only
severity, target, and event name. Conditions that require any other record data
must be evaluated in `emit`.

[`LogProcessor`]: https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/logs/trait.LogProcessor.html
[filtering capabilities]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/index.html
[logs-advanced example]: ../examples/logs-advanced/

## OpenTelemetry Log Bridge API

Do **not** use the OpenTelemetry Log Bridge API (part of the `opentelemetry`
crate) directly in application code. It is public only to allow authoring
appenders that bridge existing logging frameworks into OpenTelemetry, and is
not intended as an end-user logging API. Bridges for the
[`tracing`](https://docs.rs/opentelemetry-appender-tracing/) and
[`log`](https://docs.rs/opentelemetry-appender-log/) crates are already
available; application code should emit logs via those crates.

## Instrumentation Guidance

1. **Use the `tracing` crate**: We strongly recommend using the
   [`tracing`](https://crates.io/crates/tracing) crate for structured logging in
   Rust applications.

2. **Lean on the `tracing` ecosystem**: OpenTelemetry doesn't replace what
   `tracing` already offers. The appender is a standard `tracing-subscriber`
   `Layer`, so it composes with `fmt::Layer`, `EnvFilter`, and any other
   existing layer — for example, sending logs to stdout via `tracing`'s
   `fmt::Layer` while exporting the same logs to an OTLP endpoint via
   OpenTelemetry, or filtering what reaches the OpenTelemetry pipeline. Use
   `tracing`'s ecosystem directly; OpenTelemetry just plugs into it.

3. **Explicitly provide `name` and `target` fields**: These map to OpenTelemetry's
   EventName and Instrumentation Scope respectively. Without them, `tracing`
   synthesizes a `name` from the source location (e.g. `event src/foo.rs:42`)
   and uses the module path as `target`, neither of which is meaningful as an
   EventName or Instrumentation Scope.

4. **Trace correlation is automatic**: When a log is emitted inside an active
   OpenTelemetry span, the appender attaches the current `TraceId` and `SpanId`
   to the resulting `LogRecord`. No extra wiring is required.

5. **In-proc contextual enrichment via `tracing::span!`**: Use `tracing::span!`
   to attach contextual attributes (e.g. `session.id`, `request.id`) that
   should apply to every log inside that scope. This is the recommended
   pattern. The appender supports copying these span attributes onto each
   emitted `LogRecord` via the `experimental_span_attributes` cargo feature;
   the feature is experimental because the implementation may evolve, not
   the pattern itself. See the
   [appender README](../opentelemetry-appender-tracing/README.md) for usage.

### Example

```rust
use tracing::error;
error!(
    name: "db.client.connection.failed",
    target: "myapp.db",
    db.system.name = "postgresql",
    db.namespace = "orders",
    error.type = "connection_timeout",
    retry_count = 3,
    message = "Failed to connect to database after retries"
);
```

## Terminology

OpenTelemetry defines Events as Logs with an EventName. When you follow the guidance
above and explicitly set the `name` field on every `tracing` log, each log maps to
an OpenTelemetry Event. (Without an explicit `name`, the synthesized source-location
string is technically present but is not a meaningful EventName.)

**Note**: These are **not** mapped to Span Events. Prefer Events
(Logs with an EventName) as described above. [OTEP-4430] proposes deprecating
the *Span Events API* in favor of Events.

[OTEP-4430]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/oteps/4430-span-event-api-deprecation-plan.md

## See Also

- [Main README](../README.md) — setup guidance for logging libraries and appenders
- [OpenTelemetry Logs
  Specification](https://opentelemetry.io/docs/specs/otel/logs/)
- [`tracing` Documentation](https://docs.rs/tracing/)
- [`opentelemetry-appender-tracing`
  Documentation](https://docs.rs/opentelemetry-appender-tracing/)

## TODO

This document is intentionally high-level. Areas to expand over time, similar
to the depth in [metrics.md](metrics.md):

- Best practices, with links to runnable examples
- `LoggerProvider` lifecycle and shutdown
- Performance considerations (allocation, attribute cost)
- Attribute modelling and semantic conventions
- Common pitfalls (lost logs, missing correlation, mis-set `target`)
- Batching, exporter configuration, and back-pressure
