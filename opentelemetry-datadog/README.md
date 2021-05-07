![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

# OpenTelemetry Datadog

Community supported vendor integrations for applications instrumented with [`OpenTelemetry`].

[![Crates.io: opentelemetry-datadog](https://img.shields.io/crates/v/opentelemetry-datadog.svg)](https://crates.io/crates/opentelemetry-datadog)
[![Documentation](https://docs.rs/opentelemetry-datadog/badge.svg)](https://docs.rs/opentelemetry-datadog)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-datadog)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
[![Gitter chat](https://img.shields.io/badge/gitter-join%20chat%20%E2%86%92-brightgreen.svg)](https://gitter.im/open-telemetry/opentelemetry-rust)

[Documentation](https://docs.rs/opentelemetry-datadog) |
[Chat](https://gitter.im/open-telemetry/opentelemetry-rust)

## Overview

[`OpenTelemetry`] is a collection of tools, APIs, and SDKs used to instrument,
generate, collect, and export telemetry data (metrics, logs, and traces) for
analysis in order to understand your software's performance and behavior. This
crate provides additional propagators and exporters for sending telemetry data
to [`Datadog`].

## Features

`opentelemetry-datadog` supports following features:

- `reqwest-blocking-client`: use `reqwest` blocking http client to send spans.
- `reqwest-client`: use `reqwest` http client to send spans.
- `surf-client`: use `surf` http client to send spans.


## Kitchen Sink Full Configuration

 Example showing how to override all configuration options. See the
 [`DatadogPipelineBuilder`] docs for details of each option.

 [`DatadogPipelineBuilder`]: struct.DatadogPipelineBuilder.html

 ```no_run
 use opentelemetry::{KeyValue, trace::Tracer};
 use opentelemetry::sdk::{trace::{self, IdGenerator, Sampler}, Resource};
 use opentelemetry::sdk::export::trace::ExportResult;
 use opentelemetry_datadog::{new_pipeline, ApiVersion, Error};
 use opentelemetry_http::HttpClient;
 use async_trait::async_trait;

 // reqwest and surf are supported through features, if you prefer an
 // alternate http client you can add support by implementing HttpClient as
 // shown here.
 #[derive(Debug)]
 struct IsahcClient(isahc::HttpClient);

 #[async_trait]
 impl HttpClient for IsahcClient {
   async fn send(&self, request: http::Request<Vec<u8>>) -> ExportResult {
     let result = self.0.send_async(request).await.map_err(|err| Error::Other(err.to_string()))?;

     if result.status().is_success() {
       Ok(())
     } else {
       Err(Error::Other(result.status().to_string()).into())
     }
   }
 }

 fn main() -> Result<(), opentelemetry::trace::TraceError> {
     let tracer = new_pipeline()
         .with_service_name("my_app")
         .with_version(ApiVersion::Version05)
         .with_agent_endpoint("http://localhost:8126")
         .with_trace_config(
             trace::config()
                 .with_sampler(Sampler::AlwaysOn)
                 .with_id_generator(IdGenerator::default())
         )
         .install_batch(opentelemetry::runtime::Tokio)?;

     tracer.in_span("doing_work", |cx| {
         // Traced app logic here...
     });
     
     opentelemetry::global::shutdown_tracer_provider();

     Ok(())
 }
 ```

[`Datadog`]: https://www.datadoghq.com/
[`OpenTelemetry`]: https://crates.io/crates/opentelemetry
