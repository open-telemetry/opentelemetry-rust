//! # OpenTelemetry Jaeger Exporter
//!
//! Collects OpenTelemetry spans and reports them to a given Jaeger
//! `agent` or `collector` endpoint. See the [Jaeger Docs] for details
//! and deployment information.
//!
//! ### Jaeger Exporter Example
//!
//! This example expects a Jaeger agent running on `localhost:6831`.
//!
//! ```rust,no_run
//! use opentelemetry::{api::Key, global, sdk};
//!
//! fn init_tracer() -> thrift::Result<()> {
//!     let exporter = opentelemetry_jaeger::Exporter::builder()
//!         .with_agent_endpoint("localhost:6831".parse().unwrap())
//!         .with_process(opentelemetry_jaeger::Process {
//!             service_name: "trace-demo".to_string(),
//!             tags: vec![
//!                 Key::new("exporter").string("jaeger"),
//!                 Key::new("float").f64(312.23),
//!             ],
//!         })
//!         .init()?;
//!     let provider = sdk::Provider::builder()
//!         .with_simple_exporter(exporter)
//!         .with_config(sdk::Config {
//!             default_sampler: Box::new(sdk::Sampler::AlwaysOn),
//!             ..Default::default()
//!         })
//!         .build();
//!     global::set_provider(provider);
//!
//!     Ok(())
//! }
//!
//! fn main() -> thrift::Result<()> {
//!     init_tracer()?;
//!     // Use configured tracer
//!     Ok(())
//! }
//! ```
//!
//! ### Jaeger Collector Example
//!
//! If you want to skip the agent and submit spans directly to a Jaeger collector,
//! you can enable the optional `collector_client` feature for this crate. This
//! example expects a Jaeger collector running on `http://localhost:14268`.
//!
//! ```toml
//! [dependencies]
//! opentelemetry-jaeger = { version = "0.1", features = ["collector_client"] }
//! ```
//!
//! Then you can use the [`with_collector_endpoint`] method to specify the endpoint:
//!
//! ```rust,ignore
//! // Note that this requires the `collector_client` feature.
//!
//! use opentelemetry::{api::Key, global, sdk};
//!
//! fn init_tracer() -> thrift::Result<()> {
//!     let exporter = opentelemetry_jaeger::Exporter::builder()
//!         .with_collector_endpoint("http://localhost:14268/api/traces".to_string())
//!         .with_process(opentelemetry_jaeger::Process {
//!             service_name: "trace-demo".to_string(),
//!             tags: vec![
//!                 Key::new("exporter").string("jaeger"),
//!                 Key::new("float").f64(312.23),
//!             ],
//!         })
//!         .init()?;
//!     let provider = sdk::Provider::builder()
//!         .with_simple_exporter(exporter)
//!         .with_config(sdk::Config {
//!             default_sampler: Box::new(sdk::Sampler::AlwaysOn),
//!             ..Default::default()
//!         })
//!         .build();
//!     global::set_provider(provider);
//!
//!     Ok(())
//! }
//!
//! fn main() -> thrift::Result<()> {
//!     init_tracer()?;
//!     // Use configured tracer
//!     Ok(())
//! }
//! ```
//!
//! [Jaeger Docs]: https://www.jaegertracing.io/docs/
//! [`with_collector_endpoint`]: struct.Builder.html#with_collector_endpoint
#![deny(missing_docs, unreachable_pub, missing_debug_implementations)]
#![cfg_attr(test, deny(warnings))]
mod agent;
#[cfg(feature = "collector_client")]
mod collector;
#[allow(clippy::all, unreachable_pub, dead_code)]
mod thrift;
pub(crate) mod transport;
mod uploader;

use self::thrift::jaeger;
use opentelemetry::{api, exporter::trace, sdk};
use std::sync::{Arc, Mutex};
use std::{
    net,
    time::{Duration, SystemTime},
};

/// Default service name if no service is configured.
static DEFAULT_SERVICE_NAME: &str = "OpenTelemetry";
/// Default agent endpoint if none is provided
static DEFAULT_AGENT_ENDPOINT: &str = "127.0.0.1:6831";

/// Jaeger span exporter
#[derive(Debug)]
pub struct Exporter {
    process: jaeger::Process,
    uploader: Mutex<uploader::BatchUploader>,
}

/// Jaeger process configuration
#[derive(Debug, Default)]
pub struct Process {
    /// Jaeger service name
    pub service_name: String,
    /// Jaeger tags
    pub tags: Vec<api::KeyValue>,
}

impl Into<jaeger::Process> for Process {
    fn into(self) -> jaeger::Process {
        jaeger::Process::new(
            self.service_name,
            Some(self.tags.into_iter().map(Into::into).collect()),
        )
    }
}

impl Exporter {
    /// Create a new exporter builder.
    pub fn builder() -> Builder<String> {
        Builder::default()
    }

    /// Default `Exporter` with initialized uploader.
    pub fn init_default() -> Result<Self, ::thrift::Error> {
        Exporter::builder()
            .with_agent_endpoint(DEFAULT_AGENT_ENDPOINT.parse().unwrap())
            .init()
    }
}

impl trace::SpanExporter for Exporter {
    /// Export spans to Jaeger
    fn export(&self, batch: Vec<Arc<trace::SpanData>>) -> trace::ExportResult {
        match self.uploader.lock() {
            Ok(mut uploader) => {
                let jaeger_spans = batch.into_iter().map(Into::into).collect();
                uploader.upload(jaeger::Batch::new(self.process.clone(), jaeger_spans))
            }
            Err(_) => trace::ExportResult::FailedNotRetryable,
        }
    }

    /// Ignored for now.
    fn shutdown(&self) {}
}

/// Jaeger exporter builder
#[derive(Debug)]
pub struct Builder<T: net::ToSocketAddrs> {
    agent_endpoint: Option<T>,
    #[cfg(feature = "collector_client")]
    collector_endpoint: Option<String>,
    #[cfg(feature = "collector_client")]
    collector_username: Option<String>,
    #[cfg(feature = "collector_client")]
    collector_password: Option<String>,
    process: Process,
}

impl<T: net::ToSocketAddrs> Default for Builder<T> {
    /// Return the default Exporter Builder.
    fn default() -> Self {
        Builder {
            agent_endpoint: None,
            #[cfg(feature = "collector_client")]
            collector_endpoint: None,
            #[cfg(feature = "collector_client")]
            collector_username: None,
            #[cfg(feature = "collector_client")]
            collector_password: None,
            process: Process {
                service_name: DEFAULT_SERVICE_NAME.to_string(),
                tags: Vec::new(),
            },
        }
    }
}

impl<T: net::ToSocketAddrs> Builder<T> {
    /// Assign the agent endpoint.
    pub fn with_agent_endpoint(self, agent_endpoint: T) -> Self {
        Builder {
            agent_endpoint: Some(agent_endpoint),
            ..self
        }
    }

    /// Assign the collector endpoint.
    #[cfg(feature = "collector_client")]
    pub fn with_collector_endpoint<S: Into<String>>(self, collector_endpoint: S) -> Self {
        Builder {
            collector_endpoint: Some(collector_endpoint.into()),
            ..self
        }
    }

    /// Assign the collector username
    #[cfg(feature = "collector_client")]
    pub fn with_collector_username<S: Into<String>>(self, collector_username: S) -> Self {
        Builder {
            collector_username: Some(collector_username.into()),
            ..self
        }
    }

    /// Assign the collector password
    #[cfg(feature = "collector_client")]
    pub fn with_collector_password<S: Into<String>>(self, collector_password: S) -> Self {
        Builder {
            collector_password: Some(collector_password.into()),
            ..self
        }
    }

    /// Assign the exporter process config.
    pub fn with_process(self, process: Process) -> Self {
        Builder { process, ..self }
    }

    /// Create a new exporter from the builder
    pub fn init(self) -> ::thrift::Result<Exporter> {
        let (process, uploader) = self.init_uploader()?;

        Ok(Exporter {
            process: process.into(),
            uploader: Mutex::new(uploader),
        })
    }

    #[cfg(not(feature = "collector_client"))]
    fn init_uploader(self) -> ::thrift::Result<(Process, uploader::BatchUploader)> {
        let agent = if let Some(endpoint) = self.agent_endpoint {
            agent::AgentSyncClientUDP::new(endpoint, None)?
        } else {
            agent::AgentSyncClientUDP::new(
                DEFAULT_AGENT_ENDPOINT.parse::<net::SocketAddr>().unwrap(),
                None,
            )?
        };

        Ok((self.process, uploader::BatchUploader::Agent(agent)))
    }

    #[cfg(feature = "collector_client")]
    fn init_uploader(self) -> ::thrift::Result<(Process, uploader::BatchUploader)> {
        if self.agent_endpoint.is_some() {
            let agent = agent::AgentSyncClientUDP::new(self.agent_endpoint.unwrap(), None)?;
            Ok((self.process, uploader::BatchUploader::Agent(agent)))
        } else if self.collector_endpoint.is_some() {
            let collector = collector::CollectorSyncClientHttp::new(
                self.collector_endpoint.unwrap(),
                self.collector_username,
                self.collector_password,
            )?;
            Ok((
                self.process,
                uploader::BatchUploader::Collector(Box::new(collector)),
            ))
        } else {
            Err(::thrift::Error::from(
                "Collector endpoint or agent endpoint must be set",
            ))
        }
    }
}

#[rustfmt::skip]
impl Into<jaeger::Tag> for api::KeyValue {
    fn into(self) -> jaeger::Tag {
        let api::KeyValue { key, value } = self;
        match value {
            api::Value::String(s) => jaeger::Tag::new(key.into(), jaeger::TagType::String, Some(s), None, None, None, None),
            api::Value::F64(f) => jaeger::Tag::new(key.into(), jaeger::TagType::Double, None, Some(f.into()), None, None, None),
            api::Value::Bool(b) => jaeger::Tag::new(key.into(), jaeger::TagType::Bool, None, None, Some(b), None, None),
            api::Value::I64(i) => jaeger::Tag::new(key.into(), jaeger::TagType::Long, None, None, None, Some(i), None),
            api::Value::Bytes(b) => jaeger::Tag::new(key.into(), jaeger::TagType::Binary, None, None, None, None, Some(b)),
            // TODO: better u64 handling, jaeger thrift only has i64 support
            api::Value::U64(u) => jaeger::Tag::new(key.into(), jaeger::TagType::String, Some(u.to_string()), None, None, None, None),
            // TODO: better Array handling, jaeger thrift doesn't support arrays
            v @ api::Value::Array(_) => jaeger::Tag::new(key.into(), jaeger::TagType::String, Some(v.into()), None, None, None, None),
        }
    }
}

impl Into<jaeger::Log> for api::Event {
    fn into(self) -> jaeger::Log {
        let timestamp = self
            .timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_micros() as i64;
        let mut fields = self
            .attributes
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        fields.push(api::Key::new("name").string(self.name).into());

        jaeger::Log::new(timestamp, fields)
    }
}

impl Into<jaeger::Span> for Arc<trace::SpanData> {
    /// Convert spans to jaeger thrift span for exporting.
    fn into(self) -> jaeger::Span {
        let trace_id = self.span_context.trace_id().to_u128();
        let trace_id_high = (trace_id >> 64) as i64;
        let trace_id_low = trace_id as i64;
        jaeger::Span {
            trace_id_low,
            trace_id_high,
            span_id: self.span_context.span_id().to_u64() as i64,
            parent_span_id: self.parent_span_id.to_u64() as i64,
            operation_name: self.name.clone(),
            references: links_to_references(&self.links),
            flags: self.span_context.trace_flags() as i32,
            start_time: self
                .start_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_else(|_| Duration::from_secs(0))
                .as_micros() as i64,
            duration: self
                .end_time
                .duration_since(self.start_time)
                .unwrap_or_else(|_| Duration::from_secs(0))
                .as_micros() as i64,
            tags: build_tags(&self),
            logs: events_to_logs(&self.message_events),
        }
    }
}

fn links_to_references(links: &sdk::EvictedQueue<api::Link>) -> Option<Vec<jaeger::SpanRef>> {
    if !links.is_empty() {
        let refs = links
            .iter()
            .map(|link| {
                let span_context = link.span_context();
                let trace_id = span_context.trace_id().to_u128();
                let trace_id_high = (trace_id >> 64) as i64;
                let trace_id_low = trace_id as i64;

                // TODO: properly set the reference type when specs are defined
                //  see https://github.com/open-telemetry/opentelemetry-specification/issues/65
                jaeger::SpanRef::new(
                    jaeger::SpanRefType::ChildOf,
                    trace_id_low,
                    trace_id_high,
                    span_context.span_id().to_u64() as i64,
                )
            })
            .collect();
        Some(refs)
    } else {
        None
    }
}

fn build_tags(span_data: &Arc<trace::SpanData>) -> Option<Vec<jaeger::Tag>> {
    let mut user_overrides = UserOverrides::default();
    // TODO determine if namespacing is required to avoid collisions with set attributes
    let mut tags = span_data
        .attributes
        .iter()
        .map(|(k, v)| {
            user_overrides.record_attr(k.as_str());
            api::KeyValue::new(k.clone(), v.clone()).into()
        })
        .chain(
            span_data
                .resource
                .iter()
                .map(|(k, v)| api::KeyValue::new(k.clone(), v.clone()).into()),
        )
        .collect::<Vec<_>>();

    // Ensure error status is set
    if span_data.status_code != api::StatusCode::OK && !user_overrides.error {
        tags.push(api::Key::new(ERROR).bool(true).into())
    }

    if !user_overrides.span_kind {
        tags.push(
            api::Key::new(SPAN_KIND)
                .string(span_data.span_kind.to_string())
                .into(),
        );
    }

    if !user_overrides.status_code {
        tags.push(api::KeyValue::new(STATUS_CODE, span_data.status_code.clone() as i64).into());
    }

    if !user_overrides.status_message {
        tags.push(
            api::Key::new(STATUS_MESSAGE)
                .string(span_data.status_message.clone())
                .into(),
        );
    }

    Some(tags)
}

const ERROR: &str = "error";
const SPAN_KIND: &str = "span.kind";
const STATUS_CODE: &str = "status.code";
const STATUS_MESSAGE: &str = "status.message";

#[derive(Default)]
struct UserOverrides {
    error: bool,
    span_kind: bool,
    status_code: bool,
    status_message: bool,
}

impl UserOverrides {
    fn record_attr(&mut self, attr: &str) {
        match attr {
            ERROR => self.error = true,
            SPAN_KIND => self.span_kind = true,
            STATUS_CODE => self.status_code = true,
            STATUS_MESSAGE => self.status_message = true,
            _ => (),
        }
    }
}

fn events_to_logs(events: &sdk::EvictedQueue<api::Event>) -> Option<Vec<jaeger::Log>> {
    if events.is_empty() {
        None
    } else {
        Some(events.iter().cloned().map(Into::into).collect())
    }
}
