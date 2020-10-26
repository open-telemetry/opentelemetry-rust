![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/master/assets/logo-text.png

# OpenTelemetry Collector Rust Exporter

[`OTLP`] integration for applications instrumented with [`OpenTelemetry`].

[![Crates.io: opentelemetry-otlp](https://img.shields.io/crates/v/opentelemetry-otlp.svg)](https://crates.io/crates/opentelemetry-otlp)
[![Documentation](https://docs.rs/opentelemetry-otlp/badge.svg)](https://docs.rs/opentelemetry-otlp)
[![Crates.io](https://img.shields.io/crates/l/opentelemetry-otlp)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amaster)
[![Gitter chat](https://img.shields.io/badge/gitter-join%20chat%20%E2%86%92-brightgreen.svg)](https://gitter.im/open-telemetry/opentelemetry-rust)

[Documentation](https://docs.rs/opentelemetry-otlp) |
[Chat](https://gitter.im/open-telemetry/opentelemetry-rust)

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
$ docker run -p 55680:55680 otel/opentelemetry-collector-dev:latest
```

Then install a new pipeline with the recommended defaults to start exporting
telemetry:

```rust
use opentelemetry::tracer;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let (tracer, _uninstall) = opentelemetry_otlp::new_pipeline().install()?;

    tracer.in_span("doing_work", |cx| {
        // Traced app logic here...
    });

    Ok(())
}
```
## Performance

For optimal performance, a batch exporter is recommended as the simple
exporter will export each span synchronously on drop. You can enable the
[`tokio`] or [`async-std`] features to have a batch exporter configured for
you automatically for either executor when you install the pipeline.

```toml
[dependencies]
opentelemetry = { version = "*", features = ["tokio"] }
opentelemetry-otlp = "*"
```

[`tokio`]: https://tokio.rs
[`async-std`]: https://async.rs

## Kitchen Sink Full Configuration

Example showing how to override all configuration options. See the
[`OtlpPipelineBuilder`] docs for details of each option.

[`OtlpPipelineBuilder`]: struct.OtlpPipelineBuilder.html

```rust
use opentelemetry::{KeyValue, Tracer};
use opentelemetry::sdk::{trace, IdGenerator, Resource, Sampler};
use opentelemetry_otlp::{Compression, Credentials, Protocol};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let headers = vec![("X-Custom".to_string(), "Custom-Value".to_string())]
        .into_iter()
        .collect();

    let (tracer, _uninstall) = opentelemetry_otlp::new_pipeline()
        .with_endpoint("localhost:55680")
        .with_protocol(Protocol::Grpc)
        .with_headers(headers)
        .with_compression(Compression::Gzip)
        .with_timeout(Duration::from_secs(3))
        .with_completion_queue_count(2)
        .with_credentials(Credentials {
            cert: "tls.cert".to_string(),
            key: "tls.key".to_string(),
        })
        .with_trace_config(
            trace::config()
                .with_default_sampler(Sampler::AlwaysOn)
                .with_id_generator(IdGenerator::default())
                .with_max_events_per_span(64)
                .with_max_attributes_per_span(16)
                .with_max_events_per_span(16)
                .with_resource(Resource::new(vec![KeyValue::new("key", "value")])),
        )
        .install()?;

    tracer.in_span("doing_work", |cx| {
        // Traced app logic here...
    });

    Ok(())
}
```
