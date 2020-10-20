//! # OpenTelemetry Zipkin Exporter
//!
//! Collects OpenTelemetry spans and reports them to a given Zipkin collector
//! endpoint. See the [Zipkin Docs](https://zipkin.io/) for details and
//! deployment information.
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
//! use opentelemetry::api::trace::Tracer;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let (tracer, _uninstall) = opentelemetry_zipkin::new_pipeline().install()?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Performance
//!
//! For optimal performance, a batch exporter is recommended as the simple
//! exporter will export each span synchronously on drop. You can enable the
//! [`tokio`] or [`async-std`] features to have a batch exporter configured for
//! you automatically for either executor when you install the pipeline.
//!
//! ```toml
//! [dependencies]
//! opentelemetry = { version = "*", features = ["tokio"] }
//! opentelemetry-zipkin = "*"
//! ```
//!
//! [`tokio`]: https://tokio.rs
//! [`async-std`]: https://async.rs
//!
//! ## Bring your own http clients
//! Users can choose appropriate http clients to align with their runtime.
//!
//! Based on the feature enabled. The default http client will be different. If user doesn't specific
//! features or enabled `reqwest-blocking-client` feature. The blocking reqwest http client will be used as
//! default client. If `reqwest-client` feature is enabled. The async reqwest http client will be used. If
//! `surf-client` feature is enabled. The surf http client will be used.
//!
//! Note that async http clients may need specific runtime otherwise it will panic. User should make
//! sure the http client is running in appropriate runime.
//!
//! Users can always use their own http clients by implementing `HttpClient` trait.
//!
//! ## Kitchen Sink Full Configuration
//!
//! Example showing how to override all configuration options. See the
//! [`ZipkinPipelineBuilder`] docs for details of each option.
//!
//! [`ZipkinPipelineBuilder`]: struct.ZipkinPipelineBuilder.html
//!
//! ```no_run
//! use opentelemetry::api::{KeyValue, trace::Tracer};
//! use opentelemetry::sdk::{trace::{self, IdGenerator, Sampler}, Resource};
//! use opentelemetry::exporter::trace::ExportResult;
//! use opentelemetry::exporter::trace::HttpClient;
//! use async_trait::async_trait;
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
//!   async fn send(&self, request: http::Request<Vec<u8>>) -> Result<ExportResult, Box<dyn Error>> {
//!     let result = self.0.send_async(request).await?;
//!
//!     if result.status().is_success() {
//!       Ok(ExportResult::Success)
//!     } else {
//!       Ok(ExportResult::FailedNotRetryable)
//!     }
//!   }
//! }
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let (tracer, _uninstall) = opentelemetry_zipkin::new_pipeline()
//!         .with_http_client(IsahcClient(isahc::HttpClient::new()?))
//!         .with_service_name("my_app")
//!         .with_service_address("127.0.0.1:8080".parse()?)
//!         .with_collector_endpoint("http://localhost:9411/api/v2/spans")
//!         .with_trace_config(
//!             trace::config()
//!                 .with_default_sampler(Sampler::AlwaysOn)
//!                 .with_id_generator(IdGenerator::default())
//!                 .with_max_events_per_span(64)
//!                 .with_max_attributes_per_span(16)
//!                 .with_max_events_per_span(16)
//!                 .with_resource(Resource::new(vec![KeyValue::new("key", "value")])),
//!         )
//!         .install()?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     Ok(())
//! }
//! ```
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]
#![cfg_attr(test, deny(warnings))]

#[macro_use]
extern crate typed_builder;

mod model;
mod uploader;

use async_trait::async_trait;
use http::Uri;
use model::endpoint::Endpoint;
use opentelemetry::exporter::trace::HttpClient;
use opentelemetry::{api::trace::TracerProvider, exporter::trace, global, sdk};
use std::error::Error;
use std::io;
use std::net::SocketAddr;

/// Default Zipkin collector endpoint
const DEFAULT_COLLECTOR_ENDPOINT: &str = "http://127.0.0.1:9411/api/v2/spans";

/// Default service name if no service is configured.
const DEFAULT_SERVICE_NAME: &str = "OpenTelemetry";

/// Zipkin span exporter
#[derive(Debug)]
pub struct Exporter {
    local_endpoint: Endpoint,
    uploader: uploader::Uploader,
}

impl Exporter {
    fn new(local_endpoint: Endpoint, client: Box<dyn HttpClient>, collector_endpoint: Uri) -> Self {
        Exporter {
            local_endpoint,
            uploader: uploader::Uploader::new(client, collector_endpoint),
        }
    }
}

/// Create a new Zipkin exporter pipeline builder.
pub fn new_pipeline() -> ZipkinPipelineBuilder {
    ZipkinPipelineBuilder::default()
}

/// Builder for `ExporterConfig` struct.
#[derive(Debug)]
pub struct ZipkinPipelineBuilder {
    service_name: String,
    service_addr: Option<SocketAddr>,
    collector_endpoint: String,
    trace_config: Option<sdk::trace::Config>,
    client: Option<Box<dyn HttpClient>>,
}

impl Default for ZipkinPipelineBuilder {
    fn default() -> Self {
        ZipkinPipelineBuilder {
            #[cfg(feature = "reqwest-blocking-client")]
            client: Some(Box::new(reqwest::blocking::Client::new())),
            #[cfg(all(
                not(feature = "reqwest-blocking-client"),
                not(feature = "surf-client"),
                feature = "reqwest-client"
            ))]
            client: Some(Box::new(reqwest::Client::new())),
            #[cfg(all(
                not(feature = "reqwest-client"),
                not(feature = "reqwest-blocking-client"),
                feature = "surf-client"
            ))]
            client: Some(Box::new(surf::Client::new())),
            #[cfg(all(
                not(feature = "reqwest-client"),
                not(feature = "surf-client"),
                not(feature = "reqwest-blocking-client")
            ))]
            client: None,

            service_name: DEFAULT_SERVICE_NAME.to_string(),
            service_addr: None,
            collector_endpoint: DEFAULT_COLLECTOR_ENDPOINT.to_string(),
            trace_config: None,
        }
    }
}

impl ZipkinPipelineBuilder {
    /// Create `ExporterConfig` struct from current `ExporterConfigBuilder`
    pub fn install(mut self) -> Result<(sdk::trace::Tracer, Uninstall), Box<dyn Error>> {
        if let Some(client) = self.client {
            let endpoint = Endpoint::new(self.service_name, self.service_addr);
            let exporter = Exporter::new(endpoint, client, self.collector_endpoint.parse()?);

            let mut provider_builder =
                sdk::trace::TracerProvider::builder().with_exporter(exporter);
            if let Some(config) = self.trace_config.take() {
                provider_builder = provider_builder.with_config(config);
            }
            let provider = provider_builder.build();
            let tracer =
                provider.get_tracer("opentelemetry-zipkin", Some(env!("CARGO_PKG_VERSION")));
            let provider_guard = global::set_tracer_provider(provider);

            Ok((tracer, Uninstall(provider_guard)))
        } else {
            Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "http client must be set, users can enable reqwest or surf feature to use http \
                    client implementation within create",
            )))
        }
    }

    /// Assign the service name under which to group traces.
    pub fn with_service_name<T: Into<String>>(mut self, name: T) -> Self {
        self.service_name = name.into();
        self
    }

    /// Assign client implementation
    pub fn with_http_client<T: HttpClient + 'static>(mut self, client: T) -> Self {
        self.client = Some(Box::new(client));
        self
    }

    /// Assign the service name under which to group traces.
    pub fn with_service_address(mut self, addr: SocketAddr) -> Self {
        self.service_addr = Some(addr);
        self
    }

    /// Assign the Zipkin collector endpoint
    pub fn with_collector_endpoint<T: Into<String>>(mut self, endpoint: T) -> Self {
        self.collector_endpoint = endpoint.into();
        self
    }

    /// Assign the SDK trace configuration.
    pub fn with_trace_config(mut self, config: sdk::trace::Config) -> Self {
        self.trace_config = Some(config);
        self
    }
}

#[async_trait]
impl trace::SpanExporter for Exporter {
    /// Export spans to Zipkin collector.
    async fn export(&self, batch: Vec<trace::SpanData>) -> trace::ExportResult {
        let zipkin_spans = batch
            .into_iter()
            .map(|span| model::into_zipkin_span(self.local_endpoint.clone(), span))
            .collect();

        self.uploader.upload(zipkin_spans).await
    }
}

/// Uninstalls the Zipkin pipeline on drop.
#[derive(Debug)]
pub struct Uninstall(global::TracerProviderGuard);
