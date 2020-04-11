//! # OpenTelemetry Zipkin Exporter
//!
//! Collects OpenTelemetry spans and reports them to a given Zipkin
//! `collector` endpoint. See the [Zipkin Docs](https://zipkin.io/) for details
//! and deployment information.
//!
//! ### Zipkin collector example
//!
//! This example expects a Zipkin collector running on `localhost:9411`.
//!
//! ```rust,no_run
//! use opentelemetry::{api::Key, global, sdk};
//! use opentelemetry_zipkin::ExporterConfig;
//! use std::net::{SocketAddr, IpAddr, Ipv4Addr};
//!
//! fn init_tracer() {
//!     let exporter = opentelemetry_zipkin::Exporter::from_config(
//!        ExporterConfig::builder()
//!            .with_service_name("opentelemetry-backend".to_owned())
//!            .with_service_endpoint(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080))
//!            .build());
//!     let provider = sdk::Provider::builder()
//!         .with_simple_exporter(exporter)
//!         .with_config(sdk::Config {
//!             default_sampler: Box::new(sdk::Sampler::Always),
//!             ..Default::default()
//!         })
//!         .build();
//!
//!     global::set_provider(provider);
//! }
//! ```
//!
#![deny(missing_docs, unreachable_pub, missing_debug_implementations)]
#![cfg_attr(test, deny(warnings))]

#[macro_use]
extern crate typed_builder;

mod model;
mod uploader;

use core::any;
use model::{annotation, endpoint, span};
use opentelemetry::api;
use opentelemetry::exporter::trace;
use std::collections::HashMap;
use std::net;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Default Zipkin collector endpoint if none specified
static DEFAULT_COLLECTOR_ENDPOINT: &str = "127.0.0.1:9411";

/// Zipkin span exporter
#[derive(Debug)]
pub struct Exporter {
    config: ExporterConfig,
    uploader: uploader::Uploader,
}

/// Zipkin-specific configuration used to initialize the `Exporter`.
#[derive(Clone, Debug)]
pub struct ExporterConfig {
    local_endpoint: endpoint::Endpoint,
    collector_endpoint: String,
}

/// Builder for `ExporterConfig` struct.
#[derive(Debug)]
pub struct ExporterConfigBuilder {
    service_name: Option<String>,
    service_endpoint: Option<net::SocketAddr>,
    collector_endpoint: Option<String>,
}

impl Default for ExporterConfigBuilder {
    fn default() -> Self {
        ExporterConfigBuilder {
            collector_endpoint: None,
            service_name: None,
            service_endpoint: None,
        }
    }
}

impl ExporterConfig {
    /// Create an export config builder
    pub fn builder() -> ExporterConfigBuilder {
        ExporterConfigBuilder::default()
    }
}

impl ExporterConfigBuilder {
    /// Create `ExporterConfig` struct from current `ExporterConfigBuilder`
    pub fn build(&self) -> ExporterConfig {
        let local_endpoint: endpoint::Endpoint;
        let service_name = self
            .service_name
            .clone()
            .unwrap_or_else(|| "DEFAULT".to_owned());
        match self.service_endpoint {
            Some(socket_addr) => {
                match socket_addr.ip() {
                    net::IpAddr::V4(addr) => {
                        local_endpoint = endpoint::Endpoint::builder()
                            .service_name(service_name)
                            .ipv4(addr)
                            .port(socket_addr.port())
                            .build()
                    }
                    net::IpAddr::V6(addr) => {
                        local_endpoint = endpoint::Endpoint::builder()
                            .service_name(service_name)
                            .ipv6(addr)
                            .port(socket_addr.port())
                            .build()
                    }
                };
            }
            None => {
                local_endpoint = endpoint::Endpoint::builder()
                    .service_name(service_name)
                    .build()
            }
        };
        ExporterConfig {
            collector_endpoint: self
                .collector_endpoint
                .clone()
                .unwrap_or_else(|| DEFAULT_COLLECTOR_ENDPOINT.parse().unwrap()),
            local_endpoint,
        }
    }

    /// Assign the service name for `ConfigBuilder`
    pub fn with_service_name(&mut self, name: String) -> &mut Self {
        self.service_name = Some(name);
        self
    }

    /// Assign the service endpoint for `ConfigBuilder`
    pub fn with_service_endpoint(&mut self, endpoint: net::SocketAddr) -> &mut Self {
        self.service_endpoint = Some(endpoint);
        self
    }
}

impl Exporter {
    /// Creates new `Exporter` from a given `ExporterConfig`.
    pub fn from_config(config: ExporterConfig) -> Self {
        Exporter {
            config: config.clone(),
            uploader: uploader::Uploader::new(
                config.collector_endpoint,
                uploader::UploaderFormat::HTTP,
            ),
        }
    }
}

impl trace::SpanExporter for Exporter {
    /// Export spans to Zipkin collector.
    fn export(&self, batch: Vec<Arc<trace::SpanData>>) -> trace::ExportResult {
        let zipkin_spans: Vec<span::Span> = batch
            .into_iter()
            .map(|span| into_zipkin_span(&self.config, span))
            .collect();
        self.uploader.upload(span::ListOfSpans(zipkin_spans))
    }

    fn shutdown(&self) {}

    fn as_any(&self) -> &dyn any::Any {
        self
    }
}

/// Converts `api::Event` into an `annotation::Annotation`
impl Into<annotation::Annotation> for api::Event {
    fn into(self) -> annotation::Annotation {
        let timestamp = self
            .timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_micros() as u64;

        annotation::Annotation::builder()
            .timestamp(timestamp)
            .value(self.name)
            .build()
    }
}

/// Converts `api::SpanKind` into an `Option<span::Kind>`
fn into_zipkin_span_kind(kind: api::SpanKind) -> Option<span::Kind> {
    match kind {
        api::SpanKind::Client => Some(span::Kind::Client),
        api::SpanKind::Server => Some(span::Kind::Server),
        api::SpanKind::Producer => Some(span::Kind::Producer),
        api::SpanKind::Consumer => Some(span::Kind::Consumer),
        api::SpanKind::Internal => None,
    }
}

/// Converts a `trace::SpanData` to a `span::SpanData` for a given `ExporterConfig`, which can then
/// be ingested into a Zipkin collector.
fn into_zipkin_span(config: &ExporterConfig, span_data: Arc<trace::SpanData>) -> span::Span {
    span::Span::builder()
        .trace_id(format!("{:032x}", span_data.context.trace_id().to_u128()))
        .parent_id(format!("{:016x}", span_data.parent_span_id.to_u64()))
        .id(format!("{:016x}", span_data.context.span_id().to_u64()))
        .name(span_data.name.clone())
        .kind(into_zipkin_span_kind(span_data.span_kind.clone()))
        .timestamp(
            span_data
                .start_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_micros() as u64,
        )
        .duration(
            span_data
                .end_time
                .duration_since(span_data.start_time)
                .unwrap_or(Duration::from_secs(0))
                .as_micros() as u64,
        )
        .local_endpoint(config.local_endpoint.clone())
        .annotations(
            span_data
                .message_events
                .iter()
                .cloned()
                .map(Into::into)
                .collect(),
        )
        .tags(map_from_kvs(
            span_data
                .attributes
                .iter()
                .map(|(k, v)| api::KeyValue::new(k.clone(), v.clone()))
                .chain(
                    span_data
                        .resource
                        .iter()
                        .map(|(k, v)| api::KeyValue::new(k.clone(), v.clone())),
                ),
        ))
        .build()
}

fn map_from_kvs<T>(kvs: T) -> HashMap<String, String>
where
    T: IntoIterator<Item = api::KeyValue>,
{
    let mut map: HashMap<String, String> = HashMap::new();
    for kv in kvs {
        map.insert(kv.key.into(), kv.value.into());
    }
    map
}
