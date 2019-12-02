# OpenTelemetry-Rust

[![Crates.io: opentelemetry](https://img.shields.io/crates/v/opentelemetry.svg)](https://crates.io/crates/opentelemetry)
[![Documentation](https://docs.rs/opentelemetry/badge.svg)](https://docs.rs/opentelemetry)
[![Crates.io](https://img.shields.io/crates/l/opentelemetry)](./LICENSE)

A Rust [OpenTelemetry](https://opentelemetry.io/) client.

## Quick Start

```rust
use opentelemetry::{api::{Provider, TracerGenerics}, global, sdk};

fn main() {
    global::set_provider(sdk::Provider::default());

    global::trace_provider().get_tracer("component-a").with_span("foo", |_span| {
        global::trace_provider().get_tracer("component-b").with_span("bar", |_span| {
            global::trace_provider().get_tracer("component-c").with_span("baz", |_span| {

            })
        })
    });
}
```

See the [opentelemetry-example-app](./examples/basic.rs) for a complete example.

## Release Schedule

OpenTelemetry Rust is under active development. Below is the release schedule for the Rust library. The first version
of a release isn't guaranteed to conform to a specific version of the specification, and future releases will not
attempt to maintain backwards compatibility with the alpha release.

| Component                   | Version | Target Date     |
| --------------------------- | ------- | --------------- |
| Tracing API                 | Alpha   | November 8 2019 |
| Tracing SDK                 | Alpha   | November 8 2019 |
| Metrics API                 | Alpha   | November 8 2019 |
| Metrics SDK                 | Alpha   | November 8 2019 |
| Zipkin Trace Exporter       | Alpha   | Unknown         |
| Jaeger Trace Exporter       | Alpha   | November 8 2019 |
| Prometheus Metrics Exporter | Alpha   | November 8 2019 |
| Trace Context Propagation   | Alpha   | Unknown         |
| OpenTracing Bridge          | Alpha   | Unknown         |
| OpenCensus Bridge           | Alpha   | Unknown         |
