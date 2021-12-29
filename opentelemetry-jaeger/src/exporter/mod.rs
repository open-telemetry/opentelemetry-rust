//! # Jaeger Exporter
//!
mod agent;
#[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
mod collector;
pub(crate) mod runtime;
#[allow(clippy::all, unreachable_pub, dead_code)]
#[rustfmt::skip]
mod thrift;
mod env;
pub(crate) mod transport;
mod uploader;

use self::runtime::JaegerTraceRuntime;
use self::thrift::jaeger;
use agent::AgentAsyncClientUdp;
use async_trait::async_trait;
#[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
use collector::CollectorAsyncClientHttp;
use opentelemetry_semantic_conventions as semcov;

#[cfg(feature = "isahc_collector_client")]
#[allow(unused_imports)] // this is actually used to configure authentication
use isahc::prelude::Configurable;

use opentelemetry::sdk::export::ExportError;
use opentelemetry::trace::TraceError;
use opentelemetry::{
    global, sdk,
    sdk::export::trace,
    trace::{Event, Link, SpanKind, StatusCode, TracerProvider},
    Key, KeyValue,
};
#[cfg(feature = "collector_client")]
use opentelemetry_http::HttpClient;
use std::collections::HashSet;
use std::{
    net,
    time::{Duration, SystemTime},
};
use uploader::{AsyncUploader, SyncUploader, Uploader};

#[cfg(all(
    any(
        feature = "reqwest_collector_client",
        feature = "reqwest_blocking_collector_client"
    ),
    not(feature = "surf_collector_client"),
    not(feature = "isahc_collector_client")
))]
use headers::authorization::Credentials;
use opentelemetry::sdk::trace::Config;
use opentelemetry::sdk::Resource;
use std::sync::Arc;

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

/// Jaeger span exporter
#[derive(Debug)]
pub struct Exporter {
    process: jaeger::Process,
    /// Whether or not to export instrumentation information.
    export_instrumentation_lib: bool,
    uploader: Box<dyn Uploader>,
}

/// Jaeger process configuration
#[derive(Debug, Default)]
pub struct Process {
    /// Jaeger service name
    pub service_name: String,
    /// Jaeger tags
    pub tags: Vec<KeyValue>,
}

#[async_trait]
impl trace::SpanExporter for Exporter {
    /// Export spans to Jaeger
    async fn export(&mut self, batch: Vec<trace::SpanData>) -> trace::ExportResult {
        let mut jaeger_spans: Vec<jaeger::Span> = Vec::with_capacity(batch.len());
        let process = self.process.clone();

        for span in batch.into_iter() {
            jaeger_spans.push(convert_otel_span_into_jaeger_span(
                span,
                self.export_instrumentation_lib,
            ));
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
    #[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
    collector_endpoint: Option<Result<http::Uri, http::uri::InvalidUri>>,
    #[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
    collector_username: Option<String>,
    #[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
    collector_password: Option<String>,
    #[cfg(feature = "collector_client")]
    client: Option<Box<dyn HttpClient>>,
    export_instrument_library: bool,
    service_name: Option<String>,
    tags: Option<Vec<KeyValue>>,
    max_packet_size: Option<usize>,
    auto_split: bool,
    config: Option<sdk::trace::Config>,
}

impl Default for PipelineBuilder {
    /// Return the default Exporter Builder.
    fn default() -> Self {
        let builder_defaults = PipelineBuilder {
            agent_endpoint: vec![DEFAULT_AGENT_ENDPOINT.parse().unwrap()],
            #[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
            collector_endpoint: None,
            #[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
            collector_username: None,
            #[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
            collector_password: None,
            #[cfg(feature = "collector_client")]
            client: None,
            export_instrument_library: true,
            service_name: None,
            tags: None,
            max_packet_size: None,
            auto_split: false,
            config: None,
        };

        // Override above defaults with env vars if set
        env::assign_attrs(builder_defaults)
    }
}

impl PipelineBuilder {
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

    /// Config whether to export information of instrumentation library.
    pub fn with_instrumentation_library_tags(self, export: bool) -> Self {
        PipelineBuilder {
            export_instrument_library: export,
            ..self
        }
    }

    /// Assign the collector endpoint.
    ///
    /// E.g. "http://localhost:14268/api/traces"
    #[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "collector_client", feature = "wasm_collector_client")))
    )]
    pub fn with_collector_endpoint<T>(self, collector_endpoint: T) -> Self
    where
        http::Uri: core::convert::TryFrom<T>,
        <http::Uri as core::convert::TryFrom<T>>::Error: Into<http::uri::InvalidUri>,
    {
        PipelineBuilder {
            collector_endpoint: Some(
                core::convert::TryFrom::try_from(collector_endpoint).map_err(Into::into),
            ),
            ..self
        }
    }

    /// Assign the collector username
    #[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
    #[cfg_attr(
        docsrs,
        doc(any(feature = "collector_client", feature = "wasm_collector_client"))
    )]
    pub fn with_collector_username<S: Into<String>>(self, collector_username: S) -> Self {
        PipelineBuilder {
            collector_username: Some(collector_username.into()),
            ..self
        }
    }

    /// Get collector's username set in the builder. Default to be the value of
    /// `OTEL_EXPORTER_JAEGER_USER` environment variable.
    ///
    /// If users uses custom http client. This function can help retrieve the value of
    /// `OTEL_EXPORTER_JAEGER_USER` environment variable.
    #[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
    #[cfg_attr(
        docsrs,
        doc(any(feature = "collector_client", feature = "wasm_collector_client"))
    )]
    pub fn collector_username(&self) -> Option<String> {
        (&self.collector_username).clone()
    }

    /// Assign the collector password
    #[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
    #[cfg_attr(
        docsrs,
        doc(any(feature = "collector_client", feature = "wasm_collector_client"))
    )]
    pub fn with_collector_password<S: Into<String>>(self, collector_password: S) -> Self {
        PipelineBuilder {
            collector_password: Some(collector_password.into()),
            ..self
        }
    }

    /// Get the collector's password set in the builder. Default to be the value of
    /// `OTEL_EXPORTER_JAEGER_PASSWORD` environment variable.
    ///
    /// If users uses custom http client. This function can help retrieve the value of
    /// `OTEL_EXPORTER_JAEGER_PASSWORD` environment variable.
    #[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
    #[cfg_attr(
        docsrs,
        doc(any(feature = "collector_client", feature = "wasm_collector_client"))
    )]
    pub fn collector_password(self) -> Option<String> {
        (&self.collector_password).clone()
    }

    /// Assign the process service name.
    pub fn with_service_name<T: Into<String>>(mut self, service_name: T) -> Self {
        self.service_name = Some(service_name.into());
        self
    }

    /// Assign the process tags.
    ///
    /// Note that resource in trace [Config](sdk::trace::Config) is also reported as process tags
    /// in jaeger. If there is duplicate tags between resource and tags. Resource's value take
    /// priority even if it's empty.
    #[deprecated(
        since = "0.16.0",
        note = "please pass those tags as resource in sdk::trace::Config. Then use with_trace_config \
        method to pass the config. All key value pairs in resources will be reported as process tags"
    )]
    pub fn with_tags<T: IntoIterator<Item = KeyValue>>(mut self, tags: T) -> Self {
        self.tags = Some(tags.into_iter().collect());
        self
    }

    /// Assign the max packet size in bytes. Jaeger defaults is 65000.
    pub fn with_max_packet_size(mut self, max_packet_size: usize) -> Self {
        self.max_packet_size = Some(max_packet_size);
        self
    }

    /// Config whether to auto split batches.
    ///
    /// When auto split is set to true, the exporter will try to split the
    /// batch into smaller ones so that there will be minimal data loss. It
    /// will impact the performance.
    ///
    /// Note that if one span is too large to export, other spans within the
    /// same batch may or may not be exported. In this case, exporter will
    /// return errors as we cannot split spans.
    pub fn with_auto_split_batch(mut self, auto_split: bool) -> Self {
        self.auto_split = auto_split;
        self
    }

    /// Assign the SDK config for the exporter pipeline.
    ///
    /// # Examples
    /// Set service name via resource.
    /// ```rust
    /// use opentelemetry_jaeger::PipelineBuilder;
    /// use opentelemetry::sdk;
    /// use opentelemetry::sdk::Resource;
    /// use opentelemetry::KeyValue;
    ///
    /// let pipeline = PipelineBuilder::default()
    ///                 .with_trace_config(
    ///                       sdk::trace::Config::default()
    ///                         .with_resource(Resource::new(vec![KeyValue::new("service.name", "my-service")]))
    ///                 );
    ///
    /// ```
    pub fn with_trace_config(self, config: sdk::trace::Config) -> Self {
        PipelineBuilder {
            config: Some(config),
            ..self
        }
    }

    /// Assign the http client to use
    #[cfg(feature = "collector_client")]
    pub fn with_http_client<T: HttpClient + 'static>(mut self, client: T) -> Self {
        self.client = Some(Box::new(client));
        self
    }

    /// Install a Jaeger pipeline with a simple span processor.
    pub fn install_simple(self) -> Result<sdk::trace::Tracer, TraceError> {
        let tracer_provider = self.build_simple()?;
        let tracer = tracer_provider.versioned_tracer(
            "opentelemetry-jaeger",
            Some(env!("CARGO_PKG_VERSION")),
            None,
        );
        let _ = global::set_tracer_provider(tracer_provider);
        Ok(tracer)
    }

    /// Install a Jaeger pipeline with a batch span processor using the specified runtime.
    pub fn install_batch<R: JaegerTraceRuntime>(
        self,
        runtime: R,
    ) -> Result<sdk::trace::Tracer, TraceError> {
        let tracer_provider = self.build_batch(runtime)?;
        let tracer = tracer_provider.versioned_tracer(
            "opentelemetry-jaeger",
            Some(env!("CARGO_PKG_VERSION")),
            None,
        );
        let _ = global::set_tracer_provider(tracer_provider);
        Ok(tracer)
    }

    // To reduce the overhead of copying service name in every spans. We convert resource into jaeger tags
    // and store them into process. And set the resource in trace config to empty.
    //
    // There are multiple ways to set the service name. A `service.name` tag will be always added
    // to the process tags.
    fn build_config_and_process(&mut self, sdk_provided_resource: Resource) -> (Config, Process) {
        let (config, resource) = if let Some(mut config) = self.config.take() {
            let resource =
                if let Some(resource) = config.resource.replace(Arc::new(Resource::empty())) {
                    sdk_provided_resource.merge(resource)
                } else {
                    sdk_provided_resource
                };

            (config, resource)
        } else {
            (Config::default(), sdk_provided_resource)
        };

        let service_name = self.service_name.clone().unwrap_or_else(|| {
            resource
                .get(semcov::resource::SERVICE_NAME)
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unknown_service".to_string())
        });

        // merge the tags and resource. Resources take priority.
        let mut tags = resource
            .into_iter()
            .filter(|(key, _)| *key != semcov::resource::SERVICE_NAME)
            .map(|(key, value)| KeyValue::new(key, value))
            .collect::<Vec<KeyValue>>();

        tags.push(KeyValue::new(
            semcov::resource::SERVICE_NAME,
            service_name.clone(),
        ));

        // if users provide key list
        if let Some(provided_tags) = self.tags.take() {
            let key_set: HashSet<Key> = tags
                .iter()
                .map(|key_value| key_value.key.clone())
                .collect::<HashSet<Key>>();
            for tag in provided_tags.into_iter() {
                if !key_set.contains(&tag.key) {
                    tags.push(tag)
                }
            }
        }

        (config, Process { service_name, tags })
    }

    /// Build a configured `sdk::trace::TracerProvider` with a simple span processor.
    pub fn build_simple(mut self) -> Result<sdk::trace::TracerProvider, TraceError> {
        let mut builder = sdk::trace::TracerProvider::builder();
        let (config, process) = self.build_config_and_process(builder.sdk_provided_resource());
        let exporter = self.init_sync_exporter_with_process(process)?;
        builder = builder.with_simple_exporter(exporter);
        builder = builder.with_config(config);

        Ok(builder.build())
    }

    /// Build a configured `sdk::trace::TracerProvider` with a batch span processor using the
    /// specified runtime.
    pub fn build_batch<R: JaegerTraceRuntime>(
        mut self,
        runtime: R,
    ) -> Result<sdk::trace::TracerProvider, TraceError> {
        let mut builder = sdk::trace::TracerProvider::builder();
        let (config, process) = self.build_config_and_process(builder.sdk_provided_resource());
        let exporter = self.init_async_exporter_with_process(process, runtime.clone())?;
        builder = builder.with_batch_exporter(exporter, runtime);
        builder = builder.with_config(config);

        Ok(builder.build())
    }

    /// Initialize a new simple exporter.
    ///
    /// This is useful if you are manually constructing a pipeline.
    pub fn init_sync_exporter(mut self) -> Result<Exporter, TraceError> {
        let builder = sdk::trace::TracerProvider::builder();
        let (_, process) = self.build_config_and_process(builder.sdk_provided_resource());
        self.init_sync_exporter_with_process(process)
    }

    fn init_sync_exporter_with_process(self, process: Process) -> Result<Exporter, TraceError> {
        let export_instrumentation_lib = self.export_instrument_library;
        let uploader = self.init_sync_uploader()?;

        Ok(Exporter {
            process: process.into(),
            export_instrumentation_lib,
            uploader,
        })
    }

    /// Initialize a new exporter.
    ///
    /// This is useful if you are manually constructing a pipeline.
    pub fn init_async_exporter<R: JaegerTraceRuntime>(
        mut self,
        runtime: R,
    ) -> Result<Exporter, TraceError> {
        let builder = sdk::trace::TracerProvider::builder();
        let (_, process) = self.build_config_and_process(builder.sdk_provided_resource());
        self.init_async_exporter_with_process(process, runtime)
    }

    fn init_async_exporter_with_process<R: JaegerTraceRuntime>(
        self,
        process: Process,
        runtime: R,
    ) -> Result<Exporter, TraceError> {
        let export_instrumentation_lib = self.export_instrument_library;
        let uploader = self.init_async_uploader(runtime)?;

        Ok(Exporter {
            process: process.into(),
            export_instrumentation_lib,
            uploader,
        })
    }

    fn init_sync_uploader(self) -> Result<Box<dyn Uploader>, TraceError> {
        let agent = agent::AgentSyncClientUdp::new(
            self.agent_endpoint.as_slice(),
            self.max_packet_size,
            self.auto_split,
        )
        .map_err::<Error, _>(Into::into)?;
        Ok(Box::new(SyncUploader::Agent(agent)))
    }

    #[cfg(not(any(feature = "collector_client", feature = "wasm_collector_client")))]
    fn init_async_uploader<R: JaegerTraceRuntime>(
        self,
        runtime: R,
    ) -> Result<Box<dyn Uploader>, TraceError> {
        let agent = AgentAsyncClientUdp::new(
            self.agent_endpoint.as_slice(),
            self.max_packet_size,
            runtime,
            self.auto_split,
        )
        .map_err::<Error, _>(Into::into)?;
        Ok(Box::new(AsyncUploader::Agent(agent)))
    }

    #[cfg(feature = "collector_client")]
    fn init_async_uploader<R: JaegerTraceRuntime>(
        self,
        runtime: R,
    ) -> Result<Box<dyn Uploader>, TraceError> {
        if let Some(collector_endpoint) = self
            .collector_endpoint
            .transpose()
            .map_err::<Error, _>(Into::into)?
        {
            #[cfg(all(
                not(feature = "isahc_collector_client"),
                not(feature = "surf_collector_client"),
                not(feature = "reqwest_collector_client"),
                not(feature = "reqwest_blocking_collector_client")
            ))]
            let client = self.client.ok_or(crate::Error::NoHttpClient)?;

            #[cfg(feature = "isahc_collector_client")]
            let client = self.client.unwrap_or({
                let mut builder = isahc::HttpClient::builder();
                if let (Some(username), Some(password)) =
                    (self.collector_username, self.collector_password)
                {
                    builder = builder
                        .authentication(isahc::auth::Authentication::basic())
                        .credentials(isahc::auth::Credentials::new(username, password));
                }

                Box::new(builder.build().map_err(|err| {
                    crate::Error::ThriftAgentError(::thrift::Error::from(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        err.to_string(),
                    )))
                })?)
            });

            #[cfg(all(
                not(feature = "isahc_collector_client"),
                not(feature = "surf_collector_client"),
                any(
                    feature = "reqwest_collector_client",
                    feature = "reqwest_blocking_collector_client"
                )
            ))]
            let client = self.client.unwrap_or({
                #[cfg(feature = "reqwest_collector_client")]
                let mut builder = reqwest::ClientBuilder::new();
                #[cfg(all(
                    not(feature = "reqwest_collector_client"),
                    feature = "reqwest_blocking_collector_client"
                ))]
                let mut builder = reqwest::blocking::ClientBuilder::new();
                if let (Some(username), Some(password)) =
                    (self.collector_username, self.collector_password)
                {
                    let mut map = http::HeaderMap::with_capacity(1);
                    let auth_header_val =
                        headers::Authorization::basic(username.as_str(), password.as_str());
                    map.insert(http::header::AUTHORIZATION, auth_header_val.0.encode());
                    builder = builder.default_headers(map);
                }
                let client: Box<dyn HttpClient> =
                    Box::new(builder.build().map_err::<crate::Error, _>(Into::into)?);
                client
            });

            #[cfg(all(
                not(feature = "isahc_collector_client"),
                feature = "surf_collector_client",
                not(feature = "reqwest_collector_client"),
                not(feature = "reqwest_blocking_collector_client")
            ))]
            let client = self.client.unwrap_or({
                let client = if let (Some(username), Some(password)) =
                    (self.collector_username, self.collector_password)
                {
                    let auth = surf::http::auth::BasicAuth::new(username, password);
                    surf::Client::new().with(BasicAuthMiddleware(auth))
                } else {
                    surf::Client::new()
                };

                Box::new(client)
            });

            let collector = CollectorAsyncClientHttp::new(collector_endpoint, client);
            let uploader: AsyncUploader<R> = AsyncUploader::Collector(collector);
            Ok(Box::new(uploader))
        } else {
            let endpoint = self.agent_endpoint.as_slice();
            let agent =
                AgentAsyncClientUdp::new(endpoint, self.max_packet_size, runtime, self.auto_split)
                    .map_err::<Error, _>(Into::into)?;
            Ok(Box::new(AsyncUploader::Agent(agent)))
        }
    }

    #[cfg(all(feature = "wasm_collector_client", not(feature = "collector_client")))]
    fn init_async_uploader<R: JaegerTraceRuntime>(
        self,
        runtime: R,
    ) -> Result<Box<dyn Uploader>, TraceError> {
        if let Some(collector_endpoint) = self
            .collector_endpoint
            .transpose()
            .map_err::<Error, _>(Into::into)?
        {
            let collector = CollectorAsyncClientHttp::new(
                collector_endpoint,
                self.collector_username,
                self.collector_password,
            )
            .map_err::<Error, _>(Into::into)?;
            Ok(Box::new(AsyncUploader::Collector(collector)))
        } else {
            let endpoint = self.agent_endpoint.as_slice();
            let agent = AgentAsyncClientUdp::new(endpoint, self.max_packet_size, self.auto_split)
                .map_err::<Error, _>(Into::into)?;
            Ok(Box::new(AsyncUploader::Agent(agent)))
        }
    }
}

#[derive(Debug)]
#[cfg(feature = "surf_collector_client")]
struct BasicAuthMiddleware(surf::http::auth::BasicAuth);

#[async_trait]
#[cfg(feature = "surf_collector_client")]
impl surf::middleware::Middleware for BasicAuthMiddleware {
    async fn handle(
        &self,
        mut req: surf::Request,
        client: surf::Client,
        next: surf::middleware::Next<'_>,
    ) -> surf::Result<surf::Response> {
        req.insert_header(self.0.name(), self.0.value());
        next.run(req, client).await
    }
}

fn links_to_references(links: sdk::trace::EvictedQueue<Link>) -> Option<Vec<jaeger::SpanRef>> {
    if !links.is_empty() {
        let refs = links
            .iter()
            .map(|link| {
                let span_context = link.span_context();
                let trace_id = span_context.trace_id().to_u128();
                let trace_id_high = (trace_id >> 64) as i64;
                let trace_id_low = trace_id as i64;

                jaeger::SpanRef::new(
                    jaeger::SpanRefType::FollowsFrom,
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

/// Convert spans to jaeger thrift span for exporting.
fn convert_otel_span_into_jaeger_span(
    span: trace::SpanData,
    export_instrument_lib: bool,
) -> jaeger::Span {
    let trace_id = span.span_context.trace_id().to_u128();
    let trace_id_high = (trace_id >> 64) as i64;
    let trace_id_low = trace_id as i64;
    jaeger::Span {
        trace_id_low,
        trace_id_high,
        span_id: span.span_context.span_id().to_u64() as i64,
        parent_span_id: span.parent_span_id.to_u64() as i64,
        operation_name: span.name.into_owned(),
        references: links_to_references(span.links),
        flags: span.span_context.trace_flags().to_u8() as i32,
        start_time: span
            .start_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_micros() as i64,
        duration: span
            .end_time
            .duration_since(span.start_time)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_micros() as i64,
        tags: Some(build_span_tags(
            span.attributes,
            if export_instrument_lib {
                Some(span.instrumentation_lib)
            } else {
                None
            },
            span.status_code,
            span.status_message.into_owned(),
            span.span_kind,
        )),
        logs: events_to_logs(span.events),
    }
}

fn build_span_tags(
    attrs: sdk::trace::EvictedHashMap,
    instrumentation_lib: Option<sdk::InstrumentationLibrary>,
    status_code: StatusCode,
    status_description: String,
    kind: SpanKind,
) -> Vec<jaeger::Tag> {
    let mut user_overrides = UserOverrides::default();
    // TODO determine if namespacing is required to avoid collisions with set attributes
    let mut tags = attrs
        .into_iter()
        .map(|(k, v)| {
            user_overrides.record_attr(k.as_str());
            KeyValue::new(k, v).into()
        })
        .collect::<Vec<_>>();

    if let Some(instrumentation_lib) = instrumentation_lib {
        // Set instrument library tags
        tags.push(KeyValue::new(INSTRUMENTATION_LIBRARY_NAME, instrumentation_lib.name).into());
        if let Some(version) = instrumentation_lib.version {
            tags.push(KeyValue::new(INSTRUMENTATION_LIBRARY_VERSION, version).into())
        }
    }

    if !user_overrides.span_kind && kind != SpanKind::Internal {
        tags.push(Key::new(SPAN_KIND).string(kind.to_string()).into());
    }

    if status_code != StatusCode::Unset {
        // Ensure error status is set unless user has already overrided it
        if status_code == StatusCode::Error && !user_overrides.error {
            tags.push(Key::new(ERROR).bool(true).into());
        }
        if !user_overrides.status_code {
            tags.push(
                Key::new(OTEL_STATUS_CODE)
                    .string::<&'static str>(status_code.as_str())
                    .into(),
            );
        }
        // set status message if there is one
        if !status_description.is_empty() && !user_overrides.status_description {
            tags.push(
                Key::new(OTEL_STATUS_DESCRIPTION)
                    .string(status_description)
                    .into(),
            );
        }
    }

    tags
}

const ERROR: &str = "error";
const SPAN_KIND: &str = "span.kind";
const OTEL_STATUS_CODE: &str = "otel.status_code";
const OTEL_STATUS_DESCRIPTION: &str = "otel.status_description";

#[derive(Default)]
struct UserOverrides {
    error: bool,
    span_kind: bool,
    status_code: bool,
    status_description: bool,
}

impl UserOverrides {
    fn record_attr(&mut self, attr: &str) {
        match attr {
            ERROR => self.error = true,
            SPAN_KIND => self.span_kind = true,
            OTEL_STATUS_CODE => self.status_code = true,
            OTEL_STATUS_DESCRIPTION => self.status_description = true,
            _ => (),
        }
    }
}

fn events_to_logs(events: sdk::trace::EvictedQueue<Event>) -> Option<Vec<jaeger::Log>> {
    if events.is_empty() {
        None
    } else {
        Some(events.into_iter().map(Into::into).collect())
    }
}

/// Wrap type for errors from opentelemetry jaeger
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error from thrift agents.
    #[error("thrift agent failed with {0}")]
    ThriftAgentError(#[from] ::thrift::Error),
    /// No http client provided.
    #[cfg(feature = "collector_client")]
    #[error(
        "No http client provided. Consider enable one of the `surf_collector_client`, \
        `reqwest_collector_client`, `reqwest_blocking_collector_client`, `isahc_collector_client` \
        feature to have a default implementation. Or use with_http_client method in pipeline to \
        provide your own implementation."
    )]
    NoHttpClient,
    /// reqwest client errors
    #[error("reqwest failed with {0}")]
    #[cfg(any(
        feature = "reqwest_collector_client",
        feature = "reqwest_blocking_collector_client"
    ))]
    ReqwestClientError(#[from] reqwest::Error),

    /// invalid collector uri is provided.
    #[error("collector uri is invalid, {0}")]
    #[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
    InvalidUri(#[from] http::uri::InvalidUri),
}

impl ExportError for Error {
    fn exporter_name(&self) -> &'static str {
        "jaeger"
    }
}

#[cfg(test)]
#[cfg(all(feature = "collector_client", feature = "rt-tokio"))]
mod collector_client_tests {
    use crate::exporter::thrift::jaeger::Batch;
    use crate::new_pipeline;
    use opentelemetry::runtime::Tokio;
    use opentelemetry::sdk::Resource;
    use opentelemetry::trace::TraceError;
    use opentelemetry::KeyValue;

    mod test_http_client {
        use async_trait::async_trait;
        use bytes::Bytes;
        use http::{Request, Response};
        use opentelemetry_http::{HttpClient, HttpError};
        use std::fmt::Debug;

        pub(crate) struct TestHttpClient;

        impl Debug for TestHttpClient {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("test http client")
            }
        }

        #[async_trait]
        impl HttpClient for TestHttpClient {
            async fn send(&self, _request: Request<Vec<u8>>) -> Result<Response<Bytes>, HttpError> {
                Err("wrong uri set in http client".into())
            }
        }
    }

    #[test]
    fn test_bring_your_own_client() -> Result<(), TraceError> {
        let mut builder = new_pipeline()
            .with_collector_endpoint("localhost:6831")
            .with_http_client(test_http_client::TestHttpClient);
        let sdk_provided_resource =
            Resource::new(vec![KeyValue::new("service.name", "unknown_service")]);
        let (_, process) = builder.build_config_and_process(sdk_provided_resource);
        let mut uploader = builder.init_async_uploader(Tokio)?;
        let res = futures_executor::block_on(async {
            uploader
                .upload(Batch::new(process.into(), Vec::new()))
                .await
        });
        assert_eq!(
            format!("{:?}", res.err().unwrap()),
            "Other(\"wrong uri set in http client\")"
        );

        Ok(())
    }

    #[test]
    #[cfg(any(
        feature = "isahc_collector_client",
        feature = "surf_collector_client",
        feature = "reqwest_collector_client",
        feature = "reqwest_blocking_collector_client"
    ))]
    fn test_set_collector_endpoint() {
        let invalid_uri = new_pipeline()
            .with_collector_endpoint("127.0.0.1:14268/api/traces")
            .init_async_uploader(Tokio);
        assert!(invalid_uri.is_err());
        assert_eq!(
            format!("{:?}", invalid_uri.err().unwrap()),
            "ExportFailed(InvalidUri(InvalidUri(InvalidFormat)))"
        );

        let valid_uri = new_pipeline()
            .with_collector_endpoint("http://127.0.0.1:14268/api/traces")
            .init_async_uploader(Tokio);

        assert!(valid_uri.is_ok());
    }
}

#[cfg(test)]
mod tests {
    use super::SPAN_KIND;
    use crate::exporter::thrift::jaeger::Tag;
    use crate::exporter::{build_span_tags, OTEL_STATUS_CODE, OTEL_STATUS_DESCRIPTION};
    use opentelemetry::sdk::trace::{Config, EvictedHashMap};
    use opentelemetry::sdk::Resource;
    use opentelemetry::trace::{SpanKind, StatusCode};
    use opentelemetry::KeyValue;
    use std::env;
    use std::sync::Arc;

    fn assert_tag_contains(tags: Vec<Tag>, key: &'static str, expect_val: &'static str) {
        assert_eq!(
            tags.into_iter()
                .filter(|tag| tag.key.as_str() == key
                    && tag.v_str.as_deref().unwrap_or("") == expect_val)
                .count(),
            1,
            "Expect a tag {} with value {} but found nothing",
            key,
            expect_val
        );
    }

    fn assert_tag_not_contains(tags: Vec<Tag>, key: &'static str) {
        assert_eq!(
            tags.into_iter()
                .filter(|tag| tag.key.as_str() == key)
                .count(),
            0,
            "Not expect tag {}, but found something",
            key
        );
    }

    fn get_error_tag_test_data() -> Vec<(
        StatusCode,
        String,
        Option<&'static str>,
        Option<&'static str>,
    )> {
        // StatusCode, error message, OTEL_STATUS_CODE tag value, OTEL_STATUS_DESCRIPTION tag value
        vec![
            (StatusCode::Error, "".into(), Some("ERROR"), None),
            (StatusCode::Unset, "".into(), None, None),
            // When status is ok, no description should be in span data. This should be ensured by Otel API
            (StatusCode::Ok, "".into(), Some("OK"), None),
            (
                StatusCode::Error,
                "have message".into(),
                Some("ERROR"),
                Some("have message"),
            ),
            (StatusCode::Unset, "have message".into(), None, None),
        ]
    }

    #[test]
    fn test_set_status() {
        for (status_code, error_msg, status_tag_val, msg_tag_val) in get_error_tag_test_data() {
            let tags = build_span_tags(
                EvictedHashMap::new(20, 20),
                None,
                status_code,
                error_msg,
                SpanKind::Client,
            );
            if let Some(val) = status_tag_val {
                assert_tag_contains(tags.clone(), OTEL_STATUS_CODE, val);
            } else {
                assert_tag_not_contains(tags.clone(), OTEL_STATUS_CODE);
            }

            if let Some(val) = msg_tag_val {
                assert_tag_contains(tags.clone(), OTEL_STATUS_DESCRIPTION, val);
            } else {
                assert_tag_not_contains(tags.clone(), OTEL_STATUS_DESCRIPTION);
            }
        }
    }

    #[test]
    fn ignores_user_set_values() {
        let mut attributes = EvictedHashMap::new(20, 20);
        let user_error = true;
        let user_kind = "server";
        let user_status_code = StatusCode::Error;
        let user_status_description = "Something bad happened";
        attributes.insert(KeyValue::new("error", user_error));
        attributes.insert(KeyValue::new(SPAN_KIND, user_kind));
        attributes.insert(KeyValue::new(OTEL_STATUS_CODE, user_status_code.as_str()));
        attributes.insert(KeyValue::new(
            OTEL_STATUS_DESCRIPTION,
            user_status_description,
        ));
        let tags = build_span_tags(
            attributes,
            None,
            user_status_code,
            user_status_description.to_string(),
            SpanKind::Client,
        );

        assert!(tags
            .iter()
            .filter(|tag| tag.key.as_str() == "error")
            .all(|tag| tag.v_bool.unwrap()));
        assert_tag_contains(tags.clone(), SPAN_KIND, user_kind);
        assert_tag_contains(tags.clone(), OTEL_STATUS_CODE, user_status_code.as_str());
        assert_tag_contains(tags, OTEL_STATUS_DESCRIPTION, user_status_description);
    }

    #[test]
    fn test_set_service_name() {
        let service_name = "halloween_service";

        // set via builder's service name, it has highest priority
        let mut builder = crate::PipelineBuilder::default();
        builder = builder.with_service_name(service_name);
        let (_, process) = builder.build_config_and_process(Resource::empty());
        assert_eq!(process.service_name, service_name);

        // make sure the tags in resource are moved to process
        builder = crate::PipelineBuilder::default();
        builder = builder.with_service_name(service_name);
        builder = builder.with_trace_config(
            Config::default()
                .with_resource(Resource::new(vec![KeyValue::new("test-key", "test-value")])),
        );
        let (config, process) = builder.build_config_and_process(Resource::empty());
        assert_eq!(config.resource, Some(Arc::new(Resource::empty())));
        assert_eq!(process.tags.len(), 2);

        // sdk provided resource can override service name if users didn't provided service name to builder
        builder = crate::PipelineBuilder::default();
        let (_, process) = builder.build_config_and_process(Resource::new(vec![KeyValue::new(
            "service.name",
            "halloween_service",
        )]));
        assert_eq!(process.service_name, "halloween_service");

        // users can also provided service.name from config's resource, in this case, it will override the
        // sdk provided service name
        builder = crate::PipelineBuilder::default();
        builder = builder.with_trace_config(Config::default().with_resource(Resource::new(vec![
            KeyValue::new("service.name", "override_service"),
        ])));
        let (_, process) = builder.build_config_and_process(Resource::new(vec![KeyValue::new(
            "service.name",
            "halloween_service",
        )]));

        assert_eq!(process.service_name, "override_service");
        assert_eq!(process.tags.len(), 1);
        assert_eq!(
            process.tags[0],
            KeyValue::new("service.name", "override_service")
        );

        // OTEL_SERVICE_NAME env var also works
        env::set_var("OTEL_SERVICE_NAME", "test service");
        builder = crate::PipelineBuilder::default();
        let exporter = builder.init_sync_exporter().unwrap();
        assert_eq!(exporter.process.service_name, "test service");
        env::set_var("OTEL_SERVICE_NAME", "")
    }
}
