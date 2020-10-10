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
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let (tracer, _uninstall) = opentelemetry_zipkin::new_pipeline()
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
#![deny(missing_docs, unreachable_pub, missing_debug_implementations)]
#![cfg_attr(test, deny(warnings))]

#[macro_use]
extern crate typed_builder;

mod model;
mod uploader;

use async_trait::async_trait;
use http_client::http_types::url;
use model::endpoint::Endpoint;
use opentelemetry::{api::trace::TracerProvider, exporter::trace, global, sdk};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use http_client::h1::H1Client;

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
    fn new(
        local_endpoint: Endpoint,
        client: Box<dyn http_client::HttpClient + Send + Sync>,
        collector_endpoint: url::Url,
    ) -> Self {
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
    client: Box<dyn http_client::HttpClient + Send + Sync>,
}

impl Default for ZipkinPipelineBuilder {
    fn default() -> Self {
        ZipkinPipelineBuilder {
            service_name: DEFAULT_SERVICE_NAME.to_string(),
            service_addr: None,
            collector_endpoint: DEFAULT_COLLECTOR_ENDPOINT.to_string(),
            trace_config: None,
            client: Box::new(H1Client::new()),
        }
    }
}

impl ZipkinPipelineBuilder {
    /// Create `ExporterConfig` struct from current `ExporterConfigBuilder`
    pub fn install(mut self) -> Result<(sdk::trace::Tracer, Uninstall), Box<dyn Error>> {
        let endpoint = Endpoint::new(self.service_name, self.service_addr);
        let exporter = Exporter::new(endpoint, self.client, self.collector_endpoint.parse()?);

        let mut provider_builder = sdk::trace::TracerProvider::builder().with_exporter(exporter);
        if let Some(config) = self.trace_config.take() {
            provider_builder = provider_builder.with_config(config);
        }
        let provider = provider_builder.build();
        let tracer = provider.get_tracer("opentelemetry-zipkin", Some(env!("CARGO_PKG_VERSION")));
        let provider_guard = global::set_tracer_provider(provider);

        Ok((tracer, Uninstall(provider_guard)))
    }

    /// Assign the service name under which to group traces.
    pub fn with_service_name<T: Into<String>>(mut self, name: T) -> Self {
        self.service_name = name.into();
        self
    }

    /// Assign client implementation
    pub fn with_client<T: http_client::HttpClient + Send + Sync>(mut self, client: T) -> Self {
        self.client = Box::new(client);
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
impl<'a> trace::SpanExporter for Exporter {
    /// Export spans to Zipkin collector.
    async fn export(&self, batch: &[Arc<trace::SpanData>]) -> trace::ExportResult {
        let zipkin_spans = batch
            .iter()
            .map(|span| model::into_zipkin_span(self.local_endpoint.clone(), span))
            .collect();

        self.uploader.upload(zipkin_spans).await
    }
}

/// Uninstalls the Zipkin pipeline on drop.
#[derive(Debug)]
pub struct Uninstall(global::TracerProviderGuard);