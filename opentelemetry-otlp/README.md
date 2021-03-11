![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

# OpenTelemetry Collector Rust Exporter

[`OTLP`] integration for applications instrumented with [`OpenTelemetry`].

[![Crates.io: opentelemetry-otlp](https://img.shields.io/crates/v/opentelemetry-otlp.svg)](https://crates.io/crates/opentelemetry-otlp)
[![Documentation](https://docs.rs/opentelemetry-otlp/badge.svg)](https://docs.rs/opentelemetry-otlp)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-otlp)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
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
$ docker run -p 4317:4317 otel/opentelemetry-collector-dev:latest
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

## Options

Multiple gRPC transport layers are available. [`tonic`](https://crates.io/crates/tonic) is the default gRPC transport 
layer and is enabled by default. [`grpcio`](https://crates.io/crates/grpcio) is optional.

| gRPC transport layer | [hyperium/tonic](https://github.com/hyperium/tonic) | [tikv/grpc-rs](https://github.com/tikv/grpc-rs) |
|---|---|---|
| Feature | --features=default | --features=grpc-sys |
| gRPC library | [`tonic`](https://crates.io/crates/tonic) | [`grpcio`](https://crates.io/crates/grpcio) |
| Transport | [hyperium/hyper](https://github.com/hyperium/hyper) (Rust) | [grpc/grpc](https://github.com/grpc/grpc) (C++ binding) |
| TLS support | yes | yes |
| TLS library | rustls | OpenSSL |
| TLS optional | yes | yes |
| Supported .proto generator | [`prost`](https://crates.io/crates/prost) | [`prost`](https://crates.io/crates/prost), [`protobuf`](https://crates.io/crates/protobuf) |

## Performance

For optimal performance, a batch exporter is recommended as the simple
exporter will export each span synchronously on drop. Enable a runtime
to have a batch exporter configured automatically for either executor
when using the pipeline.

```toml
[dependencies]
opentelemetry = { version = "*", features = ["async-std"] }
opentelemetry-otlp = { version = "*", features = ["grpc-sys"] }
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
use opentelemetry_otlp::{Protocol};
use std::time::Duration;
use tonic::{
    metadata::*,
    transport::{Certificate, ClientTlsConfig},
};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let cert = std::fs::read_to_string("ca.pem")?;

    let mut map = MetadataMap::with_capacity(3);

    map.insert("x-host", "example.com".parse().unwrap());
    map.insert("x-number", "123".parse().unwrap());
    map.insert_bin("trace-proto-bin", MetadataValue::from_bytes(b"[binary data]"));

    let (tracer, _uninstall) = opentelemetry_otlp::new_pipeline()
        .with_endpoint("localhost:4317")
        .with_protocol(Protocol::Grpc)
        .with_metadata(map)
        .with_timeout(Duration::from_secs(3))
        .with_tls_config(ClientTlsConfig::new()
            .ca_certificate(Certificate::from_pem(&cert))
            .domain_name("example.com".to_string())
        )
        .with_trace_config(
            trace::config()
                .with_default_sampler(Sampler::AlwaysOn)
                .with_id_generator(IdGenerator::default())
                .with_max_events_per_span(64)
                .with_max_attributes_per_span(16)
                .with_max_events_per_span(16)
                .with_resource(Resource::new(vec![KeyValue::new("service.name", "example")])),
        )
        .install()?;

    tracer.in_span("doing_work", |cx| {
        // Traced app logic here...
    });

    Ok(())
}
```
