![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/master/assets/logo-text.png

# OpenTelemetry Zipkin

[`Zipkin`] integration for applications instrumented with [`OpenTelemetry`].

[![Crates.io: opentelemetry-zipkin](https://img.shields.io/crates/v/opentelemetry-zipkin.svg)](https://crates.io/crates/opentelemetry-zipkin)
[![Documentation](https://docs.rs/opentelemetry-zipkin/badge.svg)](https://docs.rs/opentelemetry-zipkin)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-zipkin)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amaster)
[![Gitter chat](https://img.shields.io/badge/gitter-join%20chat%20%E2%86%92-brightgreen.svg)](https://gitter.im/open-telemetry/opentelemetry-rust)

[Documentation](https://docs.rs/opentelemetry-zipkin) |
[Chat](https://gitter.im/open-telemetry/opentelemetry-rust)

## Overview

[`OpenTelemetry`] is a collection of tools, APIs, and SDKs used to instrument,
generate, collect, and export telemetry data (metrics, logs, and traces) for
analysis in order to understand your software's performance and behavior. This
crate provides a trace pipeline and exporter for sending span information to a
Zipkin collector for processing and visualization.

*Compiler support: [requires `rustc` 1.46+][msrv]*

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
    let (tracer, _uninstall) = opentelemetry_zipkin::new_pipeline().install()?;

    tracer.in_span("doing_work", |cx| {
        // Traced app logic here...
    });

    Ok(())
}
```

## Performance

For optimal performance, a batch exporter is recommended as the simple exporter
will export each span synchronously on drop. You can enable the [`tokio-support`] or
[`async-std`] features to have a batch exporter configured for you automatically
for either executor when you install the pipeline.

```toml
[dependencies]
opentelemetry = { version = "*", features = ["tokio-support"] }
opentelemetry-zipkin = { version = "*", features = ["reqwest-client"], default-features = false }
```

[`tokio-support`]: https://tokio.rs
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

Example showing how to override all configuration options. See the
[`ZipkinPipelineBuilder`] docs for details of each option.

[`ZipkinPipelineBuilder`]: struct.ZipkinPipelineBuilder.html

```rust
use opentelemetry::{KeyValue, trace::Tracer};
use opentelemetry::sdk::{trace::{self, IdGenerator, Sampler}, Resource};
use opentelemetry::sdk::export::trace::{ExportResult, HttpClient};
use opentelemetry::global;
use async_trait::async_trait;
use std::error::Error;

// `reqwest` and `surf` are supported through features, if you prefer an
// alternate http client you can add support by implementing `HttpClient` as
// shown here.
#[derive(Debug)]
struct IsahcClient(isahc::HttpClient);

#[async_trait]
impl HttpClient for IsahcClient {
  async fn send(&self, request: http::Request<Vec<u8>>) -> ExportResult {
    let result = self.0.send_async(request).await?;

    if result.status().is_success() {
      Ok(())
    } else {
      Err(result.status().as_str().into())
    }
  }
}

fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    global::set_text_map_propagator(opentelemetry_zipkin::Propagator::new());
    let (tracer, _uninstall) = opentelemetry_zipkin::new_pipeline()
        .with_http_client(IsahcClient(isahc::HttpClient::new()?))
        .with_service_name("my_app")
        .with_service_address("127.0.0.1:8080".parse()?)
        .with_collector_endpoint("http://localhost:9411/api/v2/spans")
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
version is 1.46. The current OpenTelemetry version is not guaranteed to build on
Rust versions earlier than the minimum supported version.

The current stable Rust compiler and the three most recent minor versions before
it will always be supported. For example, if the current stable compiler version
is 1.49, the minimum supported version will not be increased past 1.46, three
minor versions prior. Increasing the minimum supported compiler version is not
considered a semver breaking change as long as doing so complies with this
policy.
