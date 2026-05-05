# Examples

This directory contains runnable examples for the `opentelemetry-rust` crates.

## Getting Started

If you are new to OpenTelemetry Rust, start with one of the **Getting Started**
examples below. Each one focuses on a single signal, uses a stdout exporter,
and can be run with `cargo run` so you can immediately see telemetry on your
console:

| Signal  | Example                                       | What it shows                                              |
|---------|-----------------------------------------------|------------------------------------------------------------|
| Logs    | [logs-basic](./logs-basic/README.md)          | Bridging the `tracing` crate to OpenTelemetry              |
| Metrics | [metrics-basic](./metrics-basic/README.md)    | Counter, UpDownCounter, Histogram, Gauge and observables   |
| Traces  | [tracing-grpc](./tracing-grpc/README.md)      | A trace propagated across a gRPC client and server         |

For OTLP - the recommended exporter for production - see the
[opentelemetry-otlp examples](../opentelemetry-otlp/examples), in particular
[basic-otlp](../opentelemetry-otlp/examples/basic-otlp/README.md) (gRPC) and
[basic-otlp-http](../opentelemetry-otlp/examples/basic-otlp-http/README.md).

## All examples

### logs-basic

Uses: `opentelemetry`, `opentelemetry-appender-tracing`, `opentelemetry-stdout`.

The recommended starting point for OpenTelemetry **logging** in Rust. Shows how
to bridge logs emitted via the [`tracing`](https://docs.rs/tracing) crate to
OpenTelemetry using the `opentelemetry-appender-tracing` appender.

### logs-advanced

Uses: `opentelemetry`, `opentelemetry-appender-tracing`, `opentelemetry-stdout`.

Builds on top of `logs-basic` and shows how to implement and compose custom
`LogProcessor`s.

### metrics-basic

Uses: `opentelemetry`, `opentelemetry-stdout`.

The recommended starting point for OpenTelemetry **metrics** in Rust.
Demonstrates every instrument type (`Counter`, `UpDownCounter`, `Histogram`,
`Gauge`, and their observable counterparts).

### metrics-advanced

Uses: `opentelemetry`, `opentelemetry-stdout`.

Builds on top of `metrics-basic` and shows advanced features of the Metrics
SDK such as Views.

### tracing-grpc

Uses: `opentelemetry`, `opentelemetry-stdout`, `tonic`, `tokio`.

The recommended starting point for OpenTelemetry **tracing** in Rust. Shows
how to create spans and propagate/restore trace context across a gRPC
client/server boundary so the two sides share a single trace.

### tracing-http-propagator

Uses: `opentelemetry`, `opentelemetry-http`, `opentelemetry-stdout`.

Same idea as `tracing-grpc`, but propagating trace context over HTTP headers
instead of gRPC metadata.
