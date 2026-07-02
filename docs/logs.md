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
