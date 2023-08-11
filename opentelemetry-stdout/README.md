![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

# OpenTelemetry Stdout

Exporters for applications instrumented with [`OpenTelemetry`].

[![Crates.io: opentelemetry-stdout](https://img.shields.io/crates/v/opentelemetry-stdout.svg)](https://crates.io/crates/opentelemetry-stdout)
[![Documentation](https://docs.rs/opentelemetry-stdout/badge.svg)](https://docs.rs/opentelemetry-stdout)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-stdout)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
[![Slack](https://img.shields.io/badge/slack-@cncf/otel/rust-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C03GDP0H023)

## Overview

[`OpenTelemetry`] is a collection of tools, APIs, and SDKs used to instrument,
generate, collect, and export telemetry data (metrics, logs, and traces) for
analysis in order to understand your software's performance and behavior. This
crate provides exporters that export to stdout or any implementation of
[`std::io::Write`].

*Compiler support: [requires `rustc` 1.64+][msrv]*

[`std::io::Write`]: https://doc.rust-lang.org/std/io/trait.Write.html
[`OpenTelemetry`]: https://crates.io/crates/opentelemetry
[msrv]: #supported-rust-versions

### Quickstart

Export telemetry signals to stdout.

```rust
use opentelemetry_api::{
    metrics::MeterProvider as _,
    trace::{Span, Tracer, TracerProvider as _},
    Context, KeyValue,
};
use opentelemetry_sdk::{
    metrics::{MeterProvider, PeriodicReader},
    runtime,
    trace::{BatchSpanProcessor, TracerProvider},
};

fn init_trace() -> TracerProvider {
    let exporter = opentelemetry_stdout::SpanExporter::default();
    let processor = BatchSpanProcessor::builder(exporter, runtime::Tokio).build();
    TracerProvider::builder()
        .with_span_processor(processor)
        .build()
}

fn init_metrics() -> MeterProvider {
    let exporter = opentelemetry_stdout::MetricsExporter::default();
    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
    MeterProvider::builder().with_reader(reader).build()
}

let tracer_provider = init_trace();
let meter_provider = init_metrics();
```

Recorded traces and metrics will now be sent to stdout:

```
{"resourceMetrics":{"resource":{"attributes":[{"key":"service.name","value":{"str..
{"resourceSpans":[{"resource":{"attributes":[{"key":"service.name","value":{"stri..
```

## Supported Rust Versions

OpenTelemetry is built against the latest stable release. The minimum supported
version is 1.64. The current OpenTelemetry version is not guaranteed to build
on Rust versions earlier than the minimum supported version.

The current stable Rust compiler and the three most recent minor versions
before it will always be supported. For example, if the current stable compiler
version is 1.49, the minimum supported version will not be increased past 1.46,
three minor versions prior. Increasing the minimum supported compiler version
is not considered a semver breaking change as long as doing so complies with
this policy.
