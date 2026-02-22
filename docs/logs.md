# OpenTelemetry Rust Logs

Status: **Work-In-Progress**

## Introduction

This document provides guidance on leveraging OpenTelemetry logs
in Rust applications.

## OTel Log Bridge API

The OpenTelemetry Log Bridge API (part of the `opentelemetry` crate) is **not intended
for direct use by application developers**. It is provided for authoring log
appenders that bridge existing logging systems with OpenTelemetry. Bridges for
`tracing` and `log` crates are already available.

## Instrumentation Guidance

1. **Use the `tracing` crate**: We strongly recommend using the
   [`tracing`](https://crates.io/crates/tracing) crate for structured logging in
   Rust applications.

2. **Explicitly provide `name` and `target` fields**: These map to OpenTelemetry's
   EventName and Instrumentation Scope respectively, instead of relying on defaults.

3. **For setup details**: See
   [`opentelemetry-appender-tracing`](https://docs.rs/opentelemetry-appender-tracing/)
   for mapping details and code examples.

```rust
use tracing::error;
error!(
    name: "database_connection_failed",
    target: "database",
    error_code = "CONNECTION_TIMEOUT",
    retry_count = 3,
    message = "Failed to connect to database after retries"
);
```

## Terminology

OpenTelemetry defines Events as Logs with an EventName. Since every log from the `tracing`
crate has a `name` field that maps to EventName, every log becomes an OpenTelemetry Event.

**Note**: These are **not** mapped to Span Events. If you want to emit Span Events,
use [`tracing-opentelemetry`](https://docs.rs/tracing-opentelemetry/).

## See Also

- [OpenTelemetry Logs
  Specification](https://opentelemetry.io/docs/specs/otel/logs/)
- [`tracing` Documentation](https://docs.rs/tracing/)
- [`opentelemetry-appender-tracing`
  Documentation](https://docs.rs/opentelemetry-appender-tracing/)
