![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

# OpenTelemetry Jaeger

[`Jaeger`] integration for applications instrumented with [`OpenTelemetry`]. This includes a jaeger exporter and a jaeger propagator.

[![Crates.io: opentelemetry-jaeger](https://img.shields.io/crates/v/opentelemetry-jaeger.svg)](https://crates.io/crates/opentelemetry-jaeger)
[![Documentation](https://docs.rs/opentelemetry-jaeger/badge.svg)](https://docs.rs/opentelemetry-jaeger)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-jaeger)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
[![Slack](https://img.shields.io/badge/slack-@cncf/otel/rust-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C03GDP0H023)

**WARNING**
[Jaeger](https://www.jaegertracing.io/) supports the OpenTelemetry Protocol (OTLP) as of [v1.35.0](https://github.com/jaegertracing/jaeger/releases/tag/v1.35.0) and as a result, language specific Jaeger exporters within OpenTelemetry SDKs are [recommended for deprecation by the OpenTelemetry project](https://opentelemetry.io/blog/2022/jaeger-native-otlp/). More information and examples of using OTLP with Jaeger can be found in [Introducing native support for OpenTelemetry in Jaeger](https://medium.com/jaegertracing/introducing-native-support-for-opentelemetry-in-jaeger-eb661be8183c) and [Exporting OTLP traces to Jaeger](https://github.com/open-telemetry/opentelemetry-rust/tree/main/examples/tracing-jaeger).

The opentelemetry-jaeger crate previously contained both a Jaeger exporter and a Jaeger propagator. To prepare for the deprecation of the Jaeger exporter, the Jaeger propagator implementation has been migrated to [opentelemetry-jaeger-propagator](../opentelemetry-jaeger-propagator/).

## Overview

[`OpenTelemetry`] is a collection of tools, APIs, and SDKs used to instrument,
generate, collect, and export telemetry data (metrics, logs, and traces) for
analysis in order to understand your software's performance and behavior. This
crate provides a trace pipeline and exporter for sending span information to a
Jaeger `agent` or `collector` endpoint for processing and visualization.

*Compiler support: [requires `rustc` 1.64+][msrv]*

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
use opentelemetry::global;
use opentelemetry::trace::Tracer;
use opentelemetry_jaeger_propagator;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    global::set_text_map_propagator(opentelemetry_jaeger_propagator::Propagator::new());
    let tracer = opentelemetry_jaeger::new_agent_pipeline().install_simple()?;

    tracer.in_span("doing_work", |cx| {
        // Traced app logic here...
    });

    global::shutdown_tracer_provider(); // sending remaining spans

    Ok(())
}
```

![Jaeger UI](https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/opentelemetry-jaeger/trace.png)

## Performance

For optimal performance, a batch exporter is recommended as the simple exporter
will export each span synchronously on drop. You can enable the [`rt-tokio`],
[`rt-tokio-current-thread`] or [`rt-async-std`] features and specify a runtime
on the pipeline builder to have a batch exporter configured for you
automatically.

```toml
[dependencies]
opentelemetry_sdk = { version = "*", features = ["rt-tokio"] }
opentelemetry-jaeger = { version = "*", features = ["rt-tokio"] }
```

```rust
let tracer = opentelemetry_jaeger::new_agent_pipeline()
    .install_batch(opentelemetry_sdk::runtime::Tokio)?;
```

[`rt-tokio`]: https://tokio.rs
[`rt-tokio-current-thread`]: https://tokio.rs
[`rt-async-std`]: https://async.rs

### Jaeger Exporter From Environment Variables

The jaeger pipeline builder can be configured dynamically via environment
variables. All variables are optional, a full list of accepted options can be
found in the [jaeger variables spec].

[jaeger variables spec]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/configuration/sdk-environment-variables.md

### Jaeger Collector Example

If you want to skip the agent and submit spans directly to a Jaeger collector,
you can enable the optional `collector_client` feature for this crate. This
example expects a Jaeger collector running on `http://localhost:14268`.

```toml
[dependencies]
opentelemetry-jaeger = { version = "..", features = ["isahc_collector_client"] }
```

Then you can use the [`with_collector_endpoint`] method to specify the endpoint:

[`with_collector_endpoint`]: https://docs.rs/opentelemetry-jaeger/latest/opentelemetry_jaeger/config/collector/struct.CollectorPipeline.html#method.with_endpoint

```rust
// Note that this requires one of the following features enabled so that there is a default http client implementation
// * hyper_collector_client
// * reqwest_collector_client
// * reqwest_blocking_collector_client
// * reqwest_rustls_collector_client
// * isahc_collector_client

// You can also provide your own implementation by enable
// `collector_client` and set it with
// new_pipeline().with_http_client() method.
use opentelemetry::trace::Tracer;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let tracer = opentelemetry_jaeger::new_collector_pipeline()
        .with_endpoint("http://localhost:14268/api/traces")
        // optionally set username and password as well.
        .with_username("username")
        .with_password("s3cr3t")
        .install_batch()?;

    tracer.in_span("doing_work", |cx| {
        // Traced app logic here...
    });

    opentelemetry::global::shutdown_tracer_provider(); // sending remaining spans

    Ok(())
}
```

## Kitchen Sink Full Configuration

[`Example`] showing how to override all configuration options. See the
[`AgentPipeline`] docs for details of each option.

[`Example`]: https://docs.rs/opentelemetry-jaeger/latest/opentelemetry_jaeger/#kitchen-sink-full-configuration
[`AgentPipeline`]: https://docs.rs/opentelemetry-jaeger/latest/opentelemetry_jaeger/config/agent/struct.AgentPipeline.html

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
