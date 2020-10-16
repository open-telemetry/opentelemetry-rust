//! Collects OpenTelemetry spans and reports them to a given Jaeger
//! `agent` or `collector` endpoint. See the [Jaeger Docs] for details
//! about Jaeger and deployment information.
//!
//! [Jaeger Docs]: https://www.jaegertracing.io/docs/
//!
//! ### Quickstart
//!
//! First make sure you have a running version of the Jaeger instance
//! you want to send data to:
//!
//! ```shell
//! $ docker run -d -p6831:6831/udp -p6832:6832/udp -p16686:16686 -p14268:14268 jaegertracing/all-in-one:latest
//! ```
//!
//! Then install a new jaeger pipeline with the recommended defaults to start
//! exporting telemetry:
//!
//! ```no_run
//! use opentelemetry::api::trace::Tracer;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let (tracer, _uninstall) = opentelemetry_jaeger::new_pipeline().install()?;
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
//! opentelemetry-jaeger = { version = "*", features = ["tokio"] }
//! ```
//!
//! [`tokio`]: https://tokio.rs
//! [`async-std`]: https://async.rs
//!
//! ### Jaeger Exporter From Environment Variables
//!
//! The jaeger pipeline builder can be configured dynamically via the
//! [`from_env`] method. All variables are optinal, a full list of accepted
//! options can be found in the [jaeger variables spec].
//!
//! [`from_env`]: struct.PipelineBuilder.html#method.from_env
//! [jaeger variables spec]: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/sdk-environment-variables.md#jaeger-exporter
//!
//! ```no_run
//! use opentelemetry::api::trace::Tracer;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // export OTEL_SERVICE_NAME=my-service-name
//!     let (tracer, _uninstall) = opentelemetry_jaeger::new_pipeline().from_env().install()?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
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
//! opentelemetry-jaeger = { version = "..", features = ["collector_client"] }
//! ```
//!
//! Then you can use the [`with_collector_endpoint`] method to specify the endpoint:
//!
//! [`with_collector_endpoint`]: struct.PipelineBuilder.html#method.with_collector_endpoint
//!
//! ```ignore
//! // Note that this requires the `collector_client` feature.
//! use opentelemetry::api::trace::Tracer;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let (tracer, _uninstall) = opentelemetry_jaeger::new_pipeline()
//!         .with_collector_endpoint("http://localhost:14268/api/traces")
//!         // optionally set username and password as well.
//!         .with_collector_username("username")
//!         .with_collector_password("s3cr3t")
//!         .install()?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Kitchen Sink Full Configuration
//!
//! Example showing how to override all configuration options. See the
//! [`PipelineBuilder`] docs for details of each option.
//!
//! [`PipelineBuilder`]: struct.PipelineBuilder.html
//!
//! ```no_run
//! use opentelemetry::api::{KeyValue, trace::Tracer};
//! use opentelemetry::sdk::{trace::{self, IdGenerator, Sampler}, Resource};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let (tracer, _uninstall) = opentelemetry_jaeger::new_pipeline()
//!         .from_env()
//!         .with_agent_endpoint("localhost:6831")
//!         .with_service_name("my_app")
//!         .with_tags(vec![KeyValue::new("process_key", "process_value")])
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

mod agent;
#[cfg(feature = "collector_client")]
mod collector;
#[allow(clippy::all, unreachable_pub, dead_code)]
#[rustfmt::skip]
mod thrift;
mod env;
pub(crate) mod transport;
mod uploader;

use self::thrift::jaeger;
use agent::AgentAsyncClientUDP;
use async_trait::async_trait;
#[cfg(feature = "collector_client")]
use collector::CollectorAsyncClientHttp;
use opentelemetry::{
    api::{self, trace::TracerProvider},
    exporter::trace,
    global, sdk,
};
use std::error::Error;
use std::{
    net,
    time::{Duration, SystemTime},
};
use uploader::BatchUploader;

/// Default service name if no service is configured.
const DEFAULT_SERVICE_NAME: &str = "OpenTelemetry";

/// Default agent endpoint if none is provided
const DEFAULT_AGENT_ENDPOINT: &str = "127.0.0.1:6831";

/// Instrument Library name MUST be reported in Jaeger Span tags with the following key
const INSTRUMENTATION_LIBRARY_NAME: &str = "otel.library.name";

/// Instrument Library version MUST be reported in Jaeger Span tags with the following key
const INSTRUMENTATION_LIBRARY_VERSION: &str = "otel.library.version";

/// Create a new Jaeger exporter pipeline builder.
pub fn new_pipeline() -> PipelineBuilder {
    PipelineBuilder::default()
}

/// Guard that uninstalls the Jaeger trace pipeline when dropped
#[derive(Debug)]
pub struct Uninstall(global::TracerProviderGuard);

/// Jaeger span exporter
#[derive(Debug)]
pub struct Exporter {
    process: jaeger::Process,
    uploader: uploader::BatchUploader,
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

#[async_trait]
impl trace::SpanExporter for Exporter {
    /// Export spans to Jaeger
    async fn export(&self, batch: Vec<trace::SpanData>) -> trace::ExportResult {
        let mut jaeger_spans: Vec<jaeger::Span> = Vec::with_capacity(batch.len());
        let mut process = self.process.clone();

        for (idx, span) in batch.into_iter().enumerate() {
            if idx == 0 {
                if let Some(span_process_tags) = build_process_tags(&span) {
                    if let Some(process_tags) = &mut process.tags {
                        process_tags.extend(span_process_tags);
                    } else {
                        process.tags = Some(span_process_tags.collect())
                    }
                }
            }
            jaeger_spans.push(span.into());
        }

        self.uploader
            .upload(jaeger::Batch::new(process, jaeger_spans))
            .await
    }
}

/// Jaeger exporter builder
#[derive(Debug)]
pub struct PipelineBuilder {
    agent_endpoint: Vec<net::SocketAddr>,
    #[cfg(feature = "collector_client")]
    collector_endpoint: Option<http::Uri>,
    #[cfg(feature = "collector_client")]
    collector_username: Option<String>,
    #[cfg(feature = "collector_client")]
    collector_password: Option<String>,
    process: Process,
    config: Option<sdk::trace::Config>,
}

impl Default for PipelineBuilder {
    /// Return the default Exporter Builder.
    fn default() -> Self {
        PipelineBuilder {
            agent_endpoint: vec![DEFAULT_AGENT_ENDPOINT.parse().unwrap()],
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
            config: None,
        }
    }
}

impl PipelineBuilder {
    /// Assign builder attributes from environment variables.
    ///
    /// See the [jaeger variable spec] for full list.
    ///
    /// [jaeger variable spec]: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/sdk-environment-variables.md#jaeger-exporter
    #[allow(clippy::wrong_self_convention)]
    pub fn from_env(self) -> Self {
        env::assign_attrs(self)
    }

    /// Assign the agent endpoint.
    pub fn with_agent_endpoint<T: net::ToSocketAddrs>(self, agent_endpoint: T) -> Self {
        PipelineBuilder {
            agent_endpoint: agent_endpoint
                .to_socket_addrs()
                .map(|addrs| addrs.collect())
                .unwrap_or_default(),

            ..self
        }
    }

    /// Assign the collector endpoint.
    ///
    /// E.g. "http://localhost:14268/api/traces"
    #[cfg(feature = "collector_client")]
    pub fn with_collector_endpoint<T>(self, collector_endpoint: T) -> Self
    where
        http::Uri: core::convert::TryFrom<T>,
    {
        PipelineBuilder {
            collector_endpoint: core::convert::TryFrom::try_from(collector_endpoint).ok(),
            ..self
        }
    }

    /// Assign the collector username
    #[cfg(feature = "collector_client")]
    pub fn with_collector_username<S: Into<String>>(self, collector_username: S) -> Self {
        PipelineBuilder {
            collector_username: Some(collector_username.into()),
            ..self
        }
    }

    /// Assign the collector password
    #[cfg(feature = "collector_client")]
    pub fn with_collector_password<S: Into<String>>(self, collector_password: S) -> Self {
        PipelineBuilder {
            collector_password: Some(collector_password.into()),
            ..self
        }
    }

    /// Assign the process service name.
    pub fn with_service_name<T: Into<String>>(mut self, service_name: T) -> Self {
        self.process.service_name = service_name.into();
        self
    }

    /// Assign the process service tags.
    pub fn with_tags<T: IntoIterator<Item = api::KeyValue>>(mut self, tags: T) -> Self {
        self.process.tags = tags.into_iter().collect();
        self
    }

    /// Assign the SDK config for the exporter pipeline.
    pub fn with_trace_config(self, config: sdk::trace::Config) -> Self {
        PipelineBuilder {
            config: Some(config),
            ..self
        }
    }

    /// Install a Jaeger pipeline with the recommended defaults.
    pub fn install(self) -> Result<(sdk::trace::Tracer, Uninstall), Box<dyn Error>> {
        let tracer_provider = self.build()?;
        let tracer =
            tracer_provider.get_tracer("opentelemetry-jaeger", Some(env!("CARGO_PKG_VERSION")));

        let provider_guard = global::set_tracer_provider(tracer_provider);

        Ok((tracer, Uninstall(provider_guard)))
    }

    /// Build a configured `sdk::trace::TracerProvider` with the recommended defaults.
    pub fn build(mut self) -> Result<sdk::trace::TracerProvider, Box<dyn Error>> {
        let config = self.config.take();
        let exporter = self.init_exporter()?;

        let mut builder = sdk::trace::TracerProvider::builder().with_exporter(exporter);

        if let Some(config) = config {
            builder = builder.with_config(config)
        }

        Ok(builder.build())
    }

    /// Initialize a new exporter.
    ///
    /// This is useful if you are manually constructing a pipeline.
    pub fn init_exporter(self) -> Result<Exporter, Box<dyn Error>> {
        let (process, uploader) = self.init_uploader()?;

        Ok(Exporter {
            process: process.into(),
            uploader,
        })
    }

    #[cfg(not(feature = "collector_client"))]
    fn init_uploader(self) -> Result<(Process, BatchUploader), Box<dyn Error>> {
        let agent = AgentAsyncClientUDP::new(self.agent_endpoint.as_slice())?;
        Ok((self.process, BatchUploader::Agent(agent)))
    }

    #[cfg(feature = "collector_client")]
    fn init_uploader(self) -> Result<(Process, uploader::BatchUploader), Box<dyn Error>> {
        if let Some(collector_endpoint) = self.collector_endpoint {
            let collector = CollectorAsyncClientHttp::new(
                collector_endpoint,
                self.collector_username,
                self.collector_password,
            )?;
            Ok((self.process, uploader::BatchUploader::Collector(collector)))
        } else {
            let endpoint = self.agent_endpoint.as_slice();
            let agent = AgentAsyncClientUDP::new(endpoint)?;
            Ok((self.process, BatchUploader::Agent(agent)))
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

impl Into<jaeger::Log> for api::trace::Event {
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

impl Into<jaeger::Span> for trace::SpanData {
    /// Convert spans to jaeger thrift span for exporting.
    fn into(self) -> jaeger::Span {
        let trace_id = self.span_reference.trace_id().to_u128();
        let trace_id_high = (trace_id >> 64) as i64;
        let trace_id_low = trace_id as i64;
        jaeger::Span {
            trace_id_low,
            trace_id_high,
            span_id: self.span_reference.span_id().to_u64() as i64,
            parent_span_id: self.parent_span_id.to_u64() as i64,
            operation_name: self.name,
            references: links_to_references(self.links),
            flags: self.span_reference.trace_flags() as i32,
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
            tags: build_span_tags(
                self.attributes,
                self.instrumentation_lib,
                self.status_code,
                self.status_message,
                self.span_kind,
            ),
            logs: events_to_logs(self.message_events),
        }
    }
}

fn links_to_references(
    links: sdk::trace::EvictedQueue<api::trace::Link>,
) -> Option<Vec<jaeger::SpanRef>> {
    if !links.is_empty() {
        let refs = links
            .iter()
            .map(|link| {
                let span_reference = link.span_reference();
                let trace_id = span_reference.trace_id().to_u128();
                let trace_id_high = (trace_id >> 64) as i64;
                let trace_id_low = trace_id as i64;

                // TODO: properly set the reference type when specs are defined
                //  see https://github.com/open-telemetry/opentelemetry-specification/issues/65
                jaeger::SpanRef::new(
                    jaeger::SpanRefType::ChildOf,
                    trace_id_low,
                    trace_id_high,
                    span_reference.span_id().to_u64() as i64,
                )
            })
            .collect();
        Some(refs)
    } else {
        None
    }
}

fn build_process_tags(
    span_data: &trace::SpanData,
) -> Option<impl Iterator<Item = jaeger::Tag> + '_> {
    if span_data.resource.is_empty() {
        None
    } else {
        Some(
            span_data
                .resource
                .iter()
                .map(|(k, v)| api::KeyValue::new(k.clone(), v.clone()).into()),
        )
    }
}

fn build_span_tags(
    attrs: sdk::trace::EvictedHashMap,
    instrumentation_lib: sdk::InstrumentationLibrary,
    status_code: api::trace::StatusCode,
    status_message: String,
    kind: api::trace::SpanKind,
) -> Option<Vec<jaeger::Tag>> {
    let mut user_overrides = UserOverrides::default();
    // TODO determine if namespacing is required to avoid collisions with set attributes
    let mut tags = attrs
        .into_iter()
        .map(|(k, v)| {
            user_overrides.record_attr(k.as_str());
            api::KeyValue::new(k, v).into()
        })
        .collect::<Vec<_>>();

    // Set instrument library tags
    tags.push(api::KeyValue::new(INSTRUMENTATION_LIBRARY_NAME, instrumentation_lib.name).into());
    if let Some(version) = instrumentation_lib.version {
        tags.push(api::KeyValue::new(INSTRUMENTATION_LIBRARY_VERSION, version).into())
    }

    // Ensure error status is set
    if status_code != api::trace::StatusCode::OK && !user_overrides.error {
        tags.push(api::Key::new(ERROR).bool(true).into())
    }

    if !user_overrides.span_kind {
        tags.push(api::Key::new(SPAN_KIND).string(kind.to_string()).into());
    }

    if !user_overrides.status_code {
        tags.push(api::KeyValue::new(STATUS_CODE, status_code as i64).into());
    }

    if !user_overrides.status_message {
        tags.push(api::Key::new(STATUS_MESSAGE).string(status_message).into());
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

fn events_to_logs(events: sdk::trace::EvictedQueue<api::trace::Event>) -> Option<Vec<jaeger::Log>> {
    if events.is_empty() {
        None
    } else {
        Some(events.into_iter().map(Into::into).collect())
    }
}
