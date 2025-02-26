//! # OpenTelemetry Zipkin
//!
//! Collects OpenTelemetry spans and reports them to a given Zipkin collector
//! endpoint. See the [Zipkin Docs] for details and deployment information.
//!
//! *Compiler support: [requires `rustc` 1.64+][msrv]*
//!
//! [Zipkin Docs]: https://zipkin.io/
//! [msrv]: #supported-rust-versions
//!
//! ## Quickstart
//!
//! First make sure you have a running version of the zipkin process you want to
//! send data to:
//!
//! ```shell
//! $ docker run -d -p 9411:9411 openzipkin/zipkin
//! ```
//!
//! Then install a new pipeline with the recommended defaults to start exporting
//! telemetry:
//!
//! ```no_run
//! use opentelemetry::global;
//! use opentelemetry::trace::{Tracer, TraceError};
//! use opentelemetry_sdk::{trace::SdkTracerProvider, Resource};
//! use opentelemetry_zipkin::ZipkinExporter;
//!
//! fn main() -> Result<(), TraceError> {
//!     global::set_text_map_propagator(opentelemetry_zipkin::Propagator::new());
//!
//!     let exporter = ZipkinExporter::builder()
//!         .build()?;
//!     let provider = SdkTracerProvider::builder()
//!         .with_simple_exporter(exporter)
//!         .with_resource(
//!             Resource::builder_empty()
//!                 .with_service_name("trace-demo")
//!                 .build(),
//!         )
//!         .build();
//!     global::set_tracer_provider(provider.clone());
//!
//!     let tracer = global::tracer("zipkin-tracer");
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     provider.shutdown().expect("TracerProvider should shutdown successfully"); // sending remaining spans
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Performance
//!
//! For optimal performance, a batch exporter is recommended as the simple exporter
//! will export each span synchronously on drop. You can achieve this by creating a
//! `BatchSpanProcessor` and passing it to the trace provider.
//!
//! ```no_run
//! use opentelemetry_sdk::{
//!     trace::{
//!         BatchSpanProcessor,
//!         BatchConfigBuilder,
//!         SdkTracerProvider,
//!     },
//!     Resource,
//! };
//! use opentelemetry_zipkin::ZipkinExporter;
//!
//! fn main() -> Result<(), opentelemetry::trace::TraceError> {
//!     let exporter = ZipkinExporter::builder()
//!         .build()?;
//!
//!     let batch = BatchSpanProcessor::builder(exporter)
//!         .with_batch_config(BatchConfigBuilder::default().with_max_queue_size(4096).build())
//!         .build();
//!
//!     let provider = SdkTracerProvider::builder()
//!         .with_span_processor(batch)
//!         .with_resource(
//!             Resource::builder_empty()
//!                 .with_service_name("runtime-demo")
//!                 .build(),
//!         )
//!         .build();
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Choosing an HTTP client
//!
//! The HTTP client that this exporter will use can be overridden using features
//! or a manual implementation of the [`HttpClient`] trait. By default the
//! `reqwest-blocking-client` feature is enabled which will use the `reqwest`
//! crate. While this is compatible with both async and non-async projects, it
//! is not optimal for high-performance async applications as it will block the
//! executor thread. Consider using the `reqwest-client` (without blocking)
//! if you are in the `tokio` ecosystem.
//!
//! Note that async http clients may require a specific async runtime to be
//! available so be sure to match them appropriately.
//!
//! [`HttpClient`]: https://docs.rs/opentelemetry/0.10/opentelemetry/exporter/trace/trait.HttpClient.html
//!
//! ## Kitchen Sink Full Configuration
//!
//! Example showing how to override all configuration options. See the
//! [`ZipkinExporterBuilder`] docs for details of each option.
//!
//!
//! ```no_run
//! use opentelemetry::{global, InstrumentationScope, KeyValue, trace::{Tracer, TraceError}};
//! use opentelemetry_sdk::{trace::{self, RandomIdGenerator, Sampler}, Resource};
//! use opentelemetry_http::{HttpClient, HttpError};
//! use opentelemetry_zipkin::{Error as ZipkinError, ZipkinExporter};
//! use async_trait::async_trait;
//! use bytes::Bytes;
//! use futures_util::io::AsyncReadExt as _;
//! use http::{Request, Response};
//! use std::convert::TryInto as _;
//! use std::error::Error;
//! use http_body_util::{BodyExt, Full};
//! use hyper_util::{
//!     client::legacy::{Client, connect::HttpConnector},
//!     rt::tokio::TokioExecutor,
//! };
//!
//! // `reqwest` is supported through a feature, if you prefer an
//! // alternate http client you can add support by implementing `HttpClient` as
//! // shown here.
//! #[derive(Debug)]
//! struct HyperClient(Client<HttpConnector, Full<Bytes>>);
//!
//! #[async_trait]
//! impl HttpClient for HyperClient {
//!     async fn send_bytes(&self, req: Request<Bytes>) -> Result<Response<Bytes>, HttpError> {
//!         let resp = self
//!             .0
//!             .request(req.map(|v| Full::new(v)))
//!             .await?;
//!
//!         let response = Response::builder()
//!             .status(resp.status())
//!             .body({
//!                 resp.collect()
//!                     .await
//!                     .expect("cannot decode response")
//!                     .to_bytes()
//!             })
//!             .expect("cannot build response");
//!
//!         Ok(response)
//!     }
//! }
//!
//! fn init_traces() -> Result<trace::SdkTracerProvider, TraceError> {
//!     let exporter = ZipkinExporter::builder()
//!         .with_http_client(
//!             HyperClient(
//!                 Client::builder(TokioExecutor::new())
//!                     .build_http()
//!             )
//!         )
//!         .with_service_address(
//!             "127.0.0.1:8080"
//!                 .parse()
//!                 .map_err::<ZipkinError, _>(Into::into)?
//!         )
//!         .with_collector_endpoint("http://localhost:9411/api/v2/spans")
//!         .build()?;
//!
//!     Ok(trace::SdkTracerProvider::builder()
//!         .with_sampler(Sampler::AlwaysOn)
//!         .with_batch_exporter(exporter)
//!         .with_id_generator(RandomIdGenerator::default())
//!         .with_max_events_per_span(64)
//!         .with_max_attributes_per_span(16)
//!         .with_max_events_per_span(16)
//!         .with_resource(
//!             Resource::builder_empty()
//!                 .with_service_name("my_app")
//!                 .with_attribute(KeyValue::new("key", "value"))
//!                 .build()
//!         )
//!         .build())
//! }
//!
//! fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
//!     global::set_text_map_propagator(opentelemetry_zipkin::Propagator::new());
//!     let provider = init_traces()?;
//!     global::set_tracer_provider(provider.clone());
//!
//!     let common_scope_attributes = vec![KeyValue::new("scope-key", "scope-value")];
//!     let scope = InstrumentationScope::builder("opentelemetry-zipkin")
//!         .with_version(env!("CARGO_PKG_VERSION"))
//!         .with_attributes(common_scope_attributes)
//!         .build();
//!     let tracer = global::tracer_with_scope(scope.clone());
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     provider.shutdown()?; // sending remaining spans
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Crate Feature Flags
//!
//! The following crate feature flags are available:
//!
//! * `reqwest-blocking-client`: Export spans using the reqwest blocking http
//!   client (enabled by default).
//! * `reqwest-client`: Export spans using the reqwest non-blocking http client.
//!
//! ## Supported Rust Versions
//!
//! OpenTelemetry is built against the latest stable release. The minimum
//! supported version is 1.64. The current OpenTelemetry version is not
//! guaranteed to build on Rust versions earlier than the minimum supported
//! version.
//!
//! The current stable Rust compiler and the three most recent minor versions
//! before it will always be supported. For example, if the current stable
//! compiler version is 1.49, the minimum supported version will not be
//! increased past 1.46, three minor versions prior. Increasing the minimum
//! supported compiler version is not considered a semver breaking change as
//! long as doing so complies with this policy.
#![warn(
    future_incompatible,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    unreachable_pub,
    unused
)]
#![cfg_attr(
    docsrs,
    feature(doc_cfg, doc_auto_cfg),
    deny(rustdoc::broken_intra_doc_links)
)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo.svg"
)]
#![cfg_attr(test, deny(warnings))]

#[macro_use]
extern crate typed_builder;

mod exporter;
mod propagator;

pub use exporter::{Error, ZipkinExporter, ZipkinExporterBuilder};
pub use propagator::{B3Encoding, Propagator};
