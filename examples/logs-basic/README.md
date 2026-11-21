# Getting started with OpenTelemetry Rust Logging

This example demonstrates the basics of logging with OpenTelemetry in Rust. If
you're new to OpenTelemetry logging, this is a great place to start!

## Understanding OpenTelemetry Logging

**Important**: OpenTelemetry does not provide its own end-user facing logging
API. Instead, it integrates with existing, popular logging libraries in the Rust
ecosystem. This example uses the [tracing
crate](https://docs.rs/tracing/latest/tracing/), one of the most widely-used
logging frameworks in Rust.

The way this works is through a [log appender (or
bridge)](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/glossary.md#log-appender--bridge) - in this case, the `opentelemetry-appender-tracing` crate. The appender
captures logs emitted through the `tracing` crate and forwards them to
OpenTelemetry's logging pipeline.

## What This Example Does

This example:

1. Sets up an OpenTelemetry `LoggerProvider` with resource attributes (like
   service name)
2. Configures a **stdout exporter** to output logs to the console (for
   simplicity)
3. Bridges the `tracing` crate to OpenTelemetry using
   `opentelemetry-appender-tracing`
4. Emits a sample log event using the `tracing` library's `error!` macro
5. Properly shuts down the logging pipeline

**Note on Exporters**: This example uses the stdout exporter for demonstration
purposes. In production scenarios, you would typically use other exporters such
as:

- **OTLP exporter** (`opentelemetry-otlp`) to send logs to an OpenTelemetry
  Collector or compatible backend
- Other vendor-specific exporters for your observability platform

## Usage

Run the example to see logs emitted through the `tracing` crate being captured
and output via OpenTelemetry:

```shell
cargo run
```

You'll see the log output in your console, demonstrating how OpenTelemetry
captures and processes logs from the `tracing` library.
