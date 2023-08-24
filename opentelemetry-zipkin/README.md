![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

# OpenTelemetry Zipkin

[`Zipkin`] integration for applications instrumented with [`OpenTelemetry`].

[![Crates.io: opentelemetry-zipkin](https://img.shields.io/crates/v/opentelemetry-zipkin.svg)](https://crates.io/crates/opentelemetry-zipkin)
[![Documentation](https://docs.rs/opentelemetry-zipkin/badge.svg)](https://docs.rs/opentelemetry-zipkin)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-zipkin)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
[![Slack](https://img.shields.io/badge/slack-@cncf/otel/rust-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C03GDP0H023)


## Overview

[`OpenTelemetry`] is a collection of tools, APIs, and SDKs used to instrument,
generate, collect, and export telemetry data (metrics, logs, and traces) for
analysis in order to understand your software's performance and behavior. This
crate provides a trace pipeline and exporter for sending span information to a
Zipkin collector for processing and visualization.

*Compiler support: [requires `rustc` 1.64+][msrv]*

[`Zipkin`]: https://zipkin.io/
[`OpenTelemetry`]: https://crates.io/crates/opentelemetry
[msrv]: #supported-rust-versions

## Quickstart

First make sure you have a running version of the zipkin process you want to
send data to:

```shell
$ docker run -d -p 9411:9411 openzipkin/zipkin
```

Then install a new pipeline with the recommended defaults to start exporting
telemetry:

```rust
use opentelemetry::trace::Tracer;
use opentelemetry::global;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    global::set_text_map_propagator(opentelemetry_zipkin::Propagator::new());
    let tracer = opentelemetry_zipkin::new_pipeline().install_simple()?;

    tracer.in_span("doing_work", |cx| {
        // Traced app logic here...
    });
    
    global::shutdown_tracer_provider();

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
opentelemetry = { version = "*", features = ["rt-tokio"] }
opentelemetry-zipkin = { version = "*", features = ["reqwest-client"], default-features = false }
```

```rust
let tracer = opentelemetry_zipkin::new_pipeline()
    .install_batch(opentelemetry_sdk::runtime::Tokio)?;
```

[`rt-tokio`]: https://tokio.rs
[`async-std`]: https://async.rs

## Choosing an HTTP client

The HTTP client that this exporter will use can be overridden using features or
a manual implementation of the [`HttpClient`] trait. By default the
`reqwest-blocking-client` feature is enabled which will use the `reqwest` crate.
While this is compatible with both async and non-async projects, it is not
optimal for high-performance async applications as it will block the executor
thread. Consider using the `reqwest-client` (without blocking) or `surf-client`
features if you are in the `tokio` or `async-std` ecosystems respectively, or
select whichever client you prefer as shown below.

Note that async http clients may require a specific async runtime to be
available so be sure to match them appropriately.

[`HttpClient`]: https://docs.rs/opentelemetry/0.10/opentelemetry/exporter/trace/trait.HttpClient.html

## Kitchen Sink Full Configuration

[Example](https://docs.rs/opentelemetry-zipkin/latest/opentelemetry_zipkin/#kitchen-sink-full-configuration) showing how to override all configuration options. See the
[`ZipkinPipelineBuilder`] docs for details of each option.

[`ZipkinPipelineBuilder`]: struct.ZipkinPipelineBuilder.html

## Supported Rust Versions

OpenTelemetry is built against the latest stable release. The minimum supported
version is 1.64. The current OpenTelemetry version is not guaranteed to build on
Rust versions earlier than the minimum supported version.

The current stable Rust compiler and the three most recent minor versions before
it will always be supported. For example, if the current stable compiler version
is 1.64, the minimum supported version will not be increased past 1.46, three
minor versions prior. Increasing the minimum supported compiler version is not
considered a semver breaking change as long as doing so complies with this
policy.
