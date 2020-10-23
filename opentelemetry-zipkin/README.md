# OpenTelemetry Zipkin

Collects OpenTelemetry spans and reports them to a given Zipkin collector
endpoint. See the [Zipkin Docs] for details and deployment information.

*Compiler support: [requires `rustc` 1.42+][msrv]*

[Zipkin Docs]: https://zipkin.io/
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
use opentelemetry::api::trace::Tracer;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let (tracer, _uninstall) = opentelemetry_zipkin::new_pipeline().install()?;

    tracer.in_span("doing_work", |cx| {
        // Traced app logic here...
    });

    Ok(())
}
```

## Performance

For optimal performance, a batch exporter is recommended as the simple exporter
will export each span synchronously on drop. You can enable the [`tokio`] or
[`async-std`] features to have a batch exporter configured for you automatically
for either executor when you install the pipeline.

```toml
[dependencies]
opentelemetry = { version = "*", features = ["tokio"] }
opentelemetry-zipkin = { version = "*", features = ["reqwest-client"], default-features = false }
```

[`tokio`]: https://tokio.rs
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

[`HttpClient`]: https://docs.rs/opentelemetry/0.9/opentelemetry/exporter/trace/trait.HttpClient.html

## Kitchen Sink Full Configuration

Example showing how to override all configuration options. See the
[`ZipkinPipelineBuilder`] docs for details of each option.

[`ZipkinPipelineBuilder`]: struct.ZipkinPipelineBuilder.html

```rust
use opentelemetry::api::{KeyValue, trace::Tracer};
use opentelemetry::sdk::{trace::{self, IdGenerator, Sampler}, Resource};
use opentelemetry::exporter::trace::{ExportResult, HttpClient};
use async_trait::async_trait;
use std::error::Error;

// `reqwest` and `surf` are supported through features, if you prefer an
// alternate http client you can add support by implementing `HttpClient` as
// shown here.
#[derive(Debug)]
struct IsahcClient(isahc::HttpClient);

#[async_trait]
impl HttpClient for IsahcClient {
  async fn send(&self, request: http::Request<Vec<u8>>) -> Result<ExportResult, Box<dyn Error + Send + Sync + 'static>> {
    let result = self.0.send_async(request).await?;

    if result.status().is_success() {
      Ok(ExportResult::Success)
    } else {
      Ok(ExportResult::FailedNotRetryable)
    }
  }
}

fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
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
version is 1.42. The current OpenTelemetry version is not guaranteed to build on
Rust versions earlier than the minimum supported version.

The current stable Rust compiler and the three most recent minor versions before
it will always be supported. For example, if the current stable compiler version
is 1.45, the minimum supported version will not be increased past 1.42, three
minor versions prior. Increasing the minimum supported compiler version is not
considered a semver breaking change as long as doing so complies with this
policy.
