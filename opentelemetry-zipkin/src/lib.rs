//! # OpenTelemetry Zipkin
//!
//! Collects OpenTelemetry spans and reports them to a given Zipkin collector
//! endpoint. See the [Zipkin Docs] for details and deployment information.
//!
//! *Compiler support: [requires `rustc` 1.46+][msrv]*
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
//! use opentelemetry::trace::{Tracer, TraceError};
//! use opentelemetry::global;
//!
//! fn main() -> Result<(), TraceError> {
//!     global::set_text_map_propagator(opentelemetry_zipkin::Propagator::new());
//!     let tracer = opentelemetry_zipkin::new_pipeline().install_simple()?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     global::shutdown_tracer_provider(); // sending remaining spans
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Performance
//!
//! For optimal performance, a batch exporter is recommended as the simple exporter
//! will export each span synchronously on drop. You can enable the [`rt-tokio`],
//! [`rt-tokio-current-thread`] or [`rt-async-std`] features and specify a runtime
//! on the pipeline builder to have a batch exporter configured for you
//! automatically.
//!
//! ```toml
//! [dependencies]
//! opentelemetry = { version = "*", features = ["rt-tokio"] }
//! opentelemetry-zipkin = { version = "*", features = ["reqwest-client"], default-features = false }
//! ```
//!
//! ```no_run
//! # fn main() -> Result<(), opentelemetry::trace::TraceError> {
//! let tracer = opentelemetry_zipkin::new_pipeline()
//!     .install_batch(opentelemetry::runtime::Tokio)?;
//! # Ok(())
//! # }
//! ```
//!
//! [`rt-tokio`]: https://tokio.rs
//! [`async-std`]: https://async.rs
//!
//! ## Choosing an HTTP client
//!
//! The HTTP client that this exporter will use can be overridden using features
//! or a manual implementation of the [`HttpClient`] trait. By default the
//! `reqwest-blocking-client` feature is enabled which will use the `reqwest`
//! crate. While this is compatible with both async and non-async projects, it
//! is not optimal for high-performance async applications as it will block the
//! executor thread. Consider using the `reqwest-client` (without blocking)
//! or `surf-client` features if you are in the `tokio` or `async-std`
//! ecosystems respectively, or select whichever client you prefer as shown
//! below.
//!
//! Note that async http clients may require a specific async runtime to be
//! available so be sure to match them appropriately.
//!
//! [`HttpClient`]: https://docs.rs/opentelemetry/0.10/opentelemetry/exporter/trace/trait.HttpClient.html
//!
//! ## Kitchen Sink Full Configuration
//!
//! Example showing how to override all configuration options. See the
//! [`ZipkinPipelineBuilder`] docs for details of each option.
//!
//!
//! ```no_run
//! use opentelemetry::{KeyValue, trace::Tracer};
//! use opentelemetry::sdk::{trace::{self, IdGenerator, Sampler}, Resource};
//! use opentelemetry::sdk::export::trace::ExportResult;
//! use opentelemetry::global;
//! use opentelemetry_http::{HttpClient, HttpError};
//! use async_trait::async_trait;
//! use bytes::Bytes;
//! use futures_util::io::AsyncReadExt as _;
//! use http::{Request, Response};
//! use std::convert::TryInto as _;
//! use std::error::Error;
//!
//! // `reqwest` and `surf` are supported through features, if you prefer an
//! // alternate http client you can add support by implementing `HttpClient` as
//! // shown here.
//! #[derive(Debug)]
//! struct IsahcClient(isahc::HttpClient);
//!
//! #[async_trait]
//! impl HttpClient for IsahcClient {
//!     async fn send(&self, request: Request<Vec<u8>>) -> Result<Response<Bytes>, HttpError> {
//!         let mut response = self.0.send_async(request).await?;
//!         let status = response.status();
//!         let mut bytes = Vec::with_capacity(response.body().len().unwrap_or(0).try_into()?);
//!         isahc::AsyncReadResponseExt::copy_to(&mut response, &mut bytes).await?;
//!
//!         Ok(Response::builder()
//!             .status(response.status())
//!             .body(bytes.into())?)
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
//!     global::set_text_map_propagator(opentelemetry_zipkin::Propagator::new());
//!     let tracer = opentelemetry_zipkin::new_pipeline()
//!         .with_http_client(IsahcClient(isahc::HttpClient::new()?))
//!         .with_service_name("my_app")
//!         .with_service_address("127.0.0.1:8080".parse()?)
//!         .with_collector_endpoint("http://localhost:9411/api/v2/spans")
//!         .with_trace_config(
//!             trace::config()
//!                 .with_sampler(Sampler::AlwaysOn)
//!                 .with_id_generator(IdGenerator::default())
//!                 .with_max_events_per_span(64)
//!                 .with_max_attributes_per_span(16)
//!                 .with_max_events_per_span(16)
//!                 .with_resource(Resource::new(vec![KeyValue::new("key", "value")])),
//!         )
//!         .install_batch(opentelemetry::runtime::Tokio)?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     global::shutdown_tracer_provider(); // sending remaining spans
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
//! * `surf-client`: Export spans using the surf non-blocking http client.
//!
//! ## Supported Rust Versions
//!
//! OpenTelemetry is built against the latest stable release. The minimum
//! supported version is 1.46. The current OpenTelemetry version is not
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

pub use exporter::{new_pipeline, Error, Exporter, ZipkinPipelineBuilder};
pub use propagator::{B3Encoding, Propagator};
