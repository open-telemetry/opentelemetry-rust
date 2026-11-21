# Getting started with OpenTelemetry Rust Metrics

This example demonstrates the basics of recording metrics with OpenTelemetry in
Rust. If you're new to OpenTelemetry metrics, this is a great place to start!

## Understanding OpenTelemetry Metrics

OpenTelemetry provides an end-user facing Metrics API. Application and library
authors use this API directly (via the
[`opentelemetry`](https://docs.rs/opentelemetry/latest/opentelemetry/) crate)
to create instruments and record measurements. The
[`opentelemetry-sdk`](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/)
crate provides the implementation that aggregates those measurements and
forwards them to one or more exporters.

The Metrics API offers several instrument types, each suited to a particular
kind of measurement:

- **Counter** - monotonically increasing values (e.g. number of requests
  served).
- **UpDownCounter** - values that can increase or decrease (e.g. number of
  active connections).
- **Histogram** - distribution of values (e.g. request latency).
- **Gauge** - the current value of something at the time of measurement (e.g.
  current CPU temperature).
- **Observable Counter / UpDownCounter / Gauge** - asynchronous variants where a
  user-supplied callback is invoked at collection time to report the current
  value.

## What This Example Does

This example:

1. Sets up an OpenTelemetry `SdkMeterProvider` with resource attributes (like
   service name)
2. Configures a **stdout exporter** to output metrics to the console (for
   simplicity)
3. Creates one of every instrument type (`Counter`, `UpDownCounter`,
   `Histogram`, `Gauge`, and their observable counterparts) and records sample
   measurements
4. Properly shuts down the metrics pipeline so all buffered measurements are
   flushed

**Note on Exporters**: This example uses the stdout exporter for demonstration
purposes. In production scenarios, you would typically use other exporters such
as:

- **OTLP exporter** (`opentelemetry-otlp`) to send metrics to an OpenTelemetry
  Collector or compatible backend. See the [OTLP
  example](../../opentelemetry-otlp/examples/basic-otlp/README.md) for details.
- **Prometheus exporter** (`opentelemetry-prometheus`) to expose metrics in a
  Prometheus-scrapeable format.
- Other vendor-specific exporters for your observability platform.

## Usage

Run the example to see metrics being recorded through the OpenTelemetry Metrics
API and output via the stdout exporter:

```shell
cargo run
```

You'll see the metric output in your console, demonstrating how OpenTelemetry
collects, aggregates, and exports measurements from each instrument type.
