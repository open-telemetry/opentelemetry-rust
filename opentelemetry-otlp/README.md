![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

# OpenTelemetry Collector Rust Exporter

[`OTLP`] integration for applications instrumented with [`OpenTelemetry`].

[![Crates.io: opentelemetry-otlp](https://img.shields.io/crates/v/opentelemetry-otlp.svg)](https://crates.io/crates/opentelemetry-otlp)
[![Documentation](https://docs.rs/opentelemetry-otlp/badge.svg)](https://docs.rs/opentelemetry-otlp)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-otlp)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
[![Slack](https://img.shields.io/badge/slack-@cncf/otel/rust-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C03GDP0H023)

## Overview

[`OpenTelemetry`] is a collection of tools, APIs, and SDKs used to instrument,
generate, collect, and export telemetry data (metrics, logs, and traces) for
analysis in order to understand your software's performance and behavior.

This crate provides an exporter for sending trace and metric data in the OTLP
format to the OpenTelemetry collector. The OpenTelemetry Collector offers a
vendor-agnostic implementation on how to receive, process, and export telemetry
data. In addition, it removes the need to run, operate, and maintain multiple
agents/collectors in order to support open-source telemetry data formats (e.g.
Jaeger, Prometheus, etc.) sending to multiple open-source or commercial
back-ends.

[`OTLP`]: https://github.com/open-telemetry/opentelemetry-collector
[`OpenTelemetry`]: https://crates.io/crates/opentelemetry

## Quickstart

First make sure you have a running version of the opentelemetry collector you
want to send data to:

```shell
$ docker run -p 4317:4317 otel/opentelemetry-collector-dev:latest
```

Then install a new pipeline with the recommended defaults to start exporting
telemetry:

```rust
use opentelemetry::trace::Tracer;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // use tonic as grpc layer here.
    let tracer = opentelemetry_otlp::new_pipeline()
      .tracing()
      .with_exporter(opentelemetry_otlp::new_exporter().tonic())
      .install_simple()?;

    tracer.in_span("doing_work", |cx| {
        // Traced app logic here...
    });

    Ok(())
}
```

## Performance

For optimal performance, a batch exporter is recommended as the simple exporter
will export each span synchronously on drop. You can enable the [`rt-tokio`],
[`rt-tokio-current-thread`] or [`rt-async-std`] features and specify a runtime
on the pipeline builder to have a batch exporter configured for you
automatically.

```toml
[dependencies]
opentelemetry_sdk = { version = "*", features = ["async-std"] }
opentelemetry-otlp = { version = "*", features = ["grpc-tonic"] }
```

```rust
let tracer = opentelemetry_otlp::new_pipeline()
    .install_batch(opentelemetry_sdk::runtime::AsyncStd)?;
```

[`tokio`]: https://tokio.rs
[`async-std`]: https://async.rs

## Kitchen Sink Full Configuration

[Example](https://docs.rs/opentelemetry-otlp/latest/opentelemetry_otlp/#kitchen-sink-full-configuration)
showing how to override all configuration options.

Generally there are two parts of configuration. One is metrics config
or tracing config. Users can config it via [`OtlpTracePipeline`]
or [`OtlpMetricPipeline`]. The other is exporting configuration.
Users can set those configurations using [`OtlpExporterPipeline`] based
on the choice of exporters.
