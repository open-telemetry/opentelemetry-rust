![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/master/assets/logo-text.png

# OpenTelemetry Jaeger

[`Jaeger`] integration for applications instrumented with [`OpenTelemetry`]. This includes a jaeger exporter and a jaeger propagator.

[![Crates.io: opentelemetry-jaeger](https://img.shields.io/crates/v/opentelemetry-jaeger.svg)](https://crates.io/crates/opentelemetry-jaeger)
[![Documentation](https://docs.rs/opentelemetry-jaeger/badge.svg)](https://docs.rs/opentelemetry-jaeger)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-jaeger)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amaster)
[![Gitter chat](https://img.shields.io/badge/gitter-join%20chat%20%E2%86%92-brightgreen.svg)](https://gitter.im/open-telemetry/opentelemetry-rust)

[Documentation](https://docs.rs/opentelemetry-jaeger) |
[Chat](https://gitter.im/open-telemetry/opentelemetry-rust)

## Overview

[`OpenTelemetry`] is a collection of tools, APIs, and SDKs used to instrument,
generate, collect, and export telemetry data (metrics, logs, and traces) for
analysis in order to understand your software's performance and behavior. This
crate provides a trace pipeline and exporter for sending span information to a
Jaeger `agent` or `collector` endpoint for processing and visualization.

*Compiler support: [requires `rustc` 1.46+][msrv]*

[`Jaeger`]: https://www.jaegertracing.io/
[`OpenTelemetry`]: https://crates.io/crates/opentelemetry
[msrv]: #supported-rust-versions

### Quickstart

First make sure you have a running version of the Jaeger instance you want to
send data to:

```shell
$ docker run -d -p6831:6831/udp -p6832:6832/udp -p16686:16686 -p14268:14268 jaegertracing/all-in-one:latest
```

Then install a new jaeger pipeline with the recommended defaults to start
exporting telemetry:

```rust
use opentelemetry::tracer;
use opentelemetry::global;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
    let (tracer, _uninstall) = opentelemetry_jaeger::new_pipeline().install()?;

    tracer.in_span("doing_work", |cx| {
        // Traced app logic here...
    });

    Ok(())
}
```

![Jaeger UI](https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/master/opentelemetry-jaeger/trace.png)

## Performance

For optimal performance, a batch exporter is recommended as the simple exporter
will export each span synchronously on drop. You can enable the [`tokio-support`] or
[`async-std`] features to have a batch exporter configured for you automatically
for either executor when you install the pipeline.

```toml
[dependencies]
opentelemetry = { version = "*", features = ["tokio-support"] }
opentelemetry-jaeger = { version = "*", features = ["tokio"] }
```

[`tokio-support`]: https://tokio.rs
[`async-std`]: https://async.rs

### Jaeger Exporter From Environment Variables

The jaeger pipeline builder can be configured dynamically via the [`from_env`]
method. All variables are optional, a full list of accepted options can be found
in the [jaeger variables spec].

[`from_env`]: https://docs.rs/opentelemetry-jaeger/latest/opentelemetry_jaeger/struct.PipelineBuilder.html#method.from_env
[jaeger variables spec]: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/sdk-environment-variables.md#jaeger-exporter

```rust
use opentelemetry::tracer;
use opentelemetry::global;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
    // export OTEL_SERVICE_NAME=my-service-name
    let (tracer, _uninstall) = opentelemetry_jaeger::new_pipeline().from_env().install()?;

    tracer.in_span("doing_work", |cx| {
        // Traced app logic here...
    });

    Ok(())
}
```

### Jaeger Collector Example

If you want to skip the agent and submit spans directly to a Jaeger collector,
you can enable the optional `collector_client` feature for this crate. This
example expects a Jaeger collector running on `http://localhost:14268`.

```toml
[dependencies]
opentelemetry-jaeger = { version = "..", features = ["isahc_collector_client"] }
```

Then you can use the [`with_collector_endpoint`] method to specify the endpoint:

[`with_collector_endpoint`]: https://docs.rs/opentelemetry-jaeger/latest/opentelemetry_jaeger/struct.PipelineBuilder.html#method.with_collector_endpoint

```rust
// Note that this requires one of the following features enabled so that there is a default http client implementation
// * surf_collector_client
// * reqwest_collector_client
// * reqwest_blocking_collector_client
// * isahc_collector_client

// You can also provide your own implementation by enable
// `collecor_client` and set it with
// new_pipeline().with_http_client() method.
use opentelemetry::tracer;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let (tracer, _uninstall) = opentelemetry_jaeger::new_pipeline()
        .with_collector_endpoint("http://localhost:14268/api/traces")
        // optionally set username and password as well.
        .with_collector_username("username")
        .with_collector_password("s3cr3t")
        .install()?;

    tracer.in_span("doing_work", |cx| {
        // Traced app logic here...
    });

    Ok(())
}
```

## Kitchen Sink Full Configuration

Example showing how to override all configuration options. See the
[`PipelineBuilder`] docs for details of each option.

[`PipelineBuilder`]: https://docs.rs/opentelemetry-jaeger/latest/opentelemetry_jaeger/struct.PipelineBuilder.html

```rust
use opentelemetry::{KeyValue, Tracer};
use opentelemetry::sdk::{trace, IdGenerator, Resource, Sampler};
use opentelemetry::global;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
    let (tracer, _uninstall) = opentelemetry_jaeger::new_pipeline()
        .from_env()
        .with_agent_endpoint("localhost:6831")
        .with_service_name("my_app")
        .with_tags(vec![KeyValue::new("process_key", "process_value")])
        .with_max_packet_size(65_000)
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

## Supported Rust Versions

OpenTelemetry is built against the latest stable release. The minimum supported
version is 1.46. The current OpenTelemetry version is not guaranteed to build
on Rust versions earlier than the minimum supported version.

The current stable Rust compiler and the three most recent minor versions
before it will always be supported. For example, if the current stable compiler
version is 1.49, the minimum supported version will not be increased past 1.46,
three minor versions prior. Increasing the minimum supported compiler version
is not considered a semver breaking change as long as doing so complies with
this policy.
