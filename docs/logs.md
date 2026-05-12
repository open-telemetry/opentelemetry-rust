# OpenTelemetry Rust Logs

Status: **Stable**

## Introduction

This document provides guidance on leveraging OpenTelemetry logs in Rust
applications.

In short: use [`tracing`] for logs and events; OpenTelemetry plugs in via the
[`opentelemetry-appender-tracing`] appender. Existing `tracing` (or `log`)
instrumentation continues to work as-is — adopting OpenTelemetry for logs is
a setup change, not a code rewrite. For span guidance, see
[traces.md](traces.md).

[`tracing`]: https://crates.io/crates/tracing
[`opentelemetry-appender-tracing`]: ../opentelemetry-appender-tracing/README.md

## OpenTelemetry Log Bridge API

The OpenTelemetry Log Bridge API (part of the `opentelemetry` crate) is public
and technically usable directly, but OpenTelemetry Rust deliberately does not
advertise it as an end-user logging API. Instead, we point users at `tracing`
and its existing ecosystem, with the Log Bridge API reserved for authoring
appenders. Bridges for the
[`tracing`](https://docs.rs/opentelemetry-appender-tracing/) and
[`log`](https://docs.rs/opentelemetry-appender-log/) crates are already
available.

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
   should apply to every log inside that scope. The appender supports copying
   these span attributes onto each emitted `LogRecord` via the experimental
   `experimental_span_attributes` cargo feature; see the
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

**Note**: These are **not** mapped to Span Events. OpenTelemetry is deprecating
Span Events in favor of Events (Logs with an EventName), so use Events as
described above rather than Span Events.

## See Also

- [Main README](../README.md) — setup guidance for logging libraries and appenders
- [OpenTelemetry Logs
  Specification](https://opentelemetry.io/docs/specs/otel/logs/)
- [`tracing` Documentation](https://docs.rs/tracing/)
- [`opentelemetry-appender-tracing`
  Documentation](https://docs.rs/opentelemetry-appender-tracing/)
