//! # Jaeger Exporter
//!
mod agent;
#[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
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
#[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
use collector::CollectorAsyncClientHttp;

#[cfg(feature = "isahc_collector_client")]
use isahc::prelude::Configurable;

#[cfg(feature = "collector_client")]
use opentelemetry::sdk::export::trace::HttpClient;
use opentelemetry::sdk::export::ExportError;
use opentelemetry::trace::TraceError;
use opentelemetry::{
    global, sdk,
    sdk::export::trace,
    trace::{Event, Link, SpanKind, StatusCode, TracerProvider},
    Key, KeyValue, Value,
};
use std::{
    net,
    time::{Duration, SystemTime},
};
use uploader::BatchUploader;

#[cfg(all(
    any(
        feature = "reqwest_collector_client",
        feature = "reqwest_blocking_collector_client"
    ),
    not(feature = "surf_collector_client"),
    not(feature = "isahc_collector_client")
))]
use headers::authorization::Credentials;

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
#[must_use]
#[derive(Debug)]
pub struct Uninstall(global::TracerProviderGuard);

/// Jaeger span exporter
#[derive(Debug)]
pub struct Exporter {
    process: jaeger::Process,
    /// Whether or not to export instrumentation information.
    export_instrumentation_lib: bool,
    uploader: uploader::BatchUploader,
}

/// Jaeger process configuration
#[derive(Debug, Default)]
pub struct Process {
    /// Jaeger service name
    pub service_name: String,
    /// Jaeger tags
    pub tags: Vec<KeyValue>,
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
    async fn export(&mut self, batch: Vec<trace::SpanData>) -> trace::ExportResult {
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
    collector_endpoint: Option<http::Uri>,
    #[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
    collector_username: Option<String>,
    #[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
    collector_password: Option<String>,
    #[cfg(feature = "collector_client")]
    client: Option<Box<dyn opentelemetry::sdk::export::trace::HttpClient>>,
    export_instrument_library: bool,
    process: Process,
    config: Option<sdk::trace::Config>,
}

impl Default for PipelineBuilder {
    /// Return the default Exporter Builder.
    fn default() -> Self {
        PipelineBuilder {
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
    {
        PipelineBuilder {
            collector_endpoint: core::convert::TryFrom::try_from(collector_endpoint).ok(),
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

    /// Assign the process service name.
    pub fn with_service_name<T: Into<String>>(mut self, service_name: T) -> Self {
        self.process.service_name = service_name.into();
        self
    }

    /// Assign the process service tags.
    pub fn with_tags<T: IntoIterator<Item = KeyValue>>(mut self, tags: T) -> Self {
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

    /// Assign the http client to use
    #[cfg(feature = "collector_client")]
    pub fn with_http_client<T: HttpClient + 'static>(mut self, client: T) -> Self {
        self.client = Some(Box::new(client));
        self
    }

    /// Install a Jaeger pipeline with the recommended defaults.
    pub fn install(self) -> Result<(sdk::trace::Tracer, Uninstall), TraceError> {
        let tracer_provider = self.build()?;
        let tracer =
            tracer_provider.get_tracer("opentelemetry-jaeger", Some(env!("CARGO_PKG_VERSION")));

        let provider_guard = global::set_tracer_provider(tracer_provider);

        Ok((tracer, Uninstall(provider_guard)))
    }

    /// Build a configured `sdk::trace::TracerProvider` with the recommended defaults.
    pub fn build(mut self) -> Result<sdk::trace::TracerProvider, TraceError> {
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
    pub fn init_exporter(self) -> Result<Exporter, TraceError> {
        let export_instrumentation_lib = self.export_instrument_library;
        let (process, uploader) = self.init_uploader()?;

        Ok(Exporter {
            process: process.into(),
            export_instrumentation_lib,
            uploader,
        })
    }

    #[cfg(not(any(feature = "collector_client", feature = "wasm_collector_client")))]
    fn init_uploader(self) -> Result<(Process, BatchUploader), TraceError> {
        let agent = AgentAsyncClientUDP::new(self.agent_endpoint.as_slice())
            .map_err::<Error, _>(Into::into)?;
        Ok((self.process, BatchUploader::Agent(agent)))
    }

    #[cfg(feature = "collector_client")]
    fn init_uploader(self) -> Result<(Process, uploader::BatchUploader), TraceError> {
        if let Some(collector_endpoint) = self.collector_endpoint {
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

                Box::new(IsahcHttpClient(builder.build().map_err(|err| {
                    crate::Error::ThriftAgentError(::thrift::Error::from(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        err.to_string(),
                    )))
                })?))
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

            let collector = CollectorAsyncClientHttp::new(collector_endpoint, client)
                .map_err::<Error, _>(Into::into)?;
            Ok((self.process, uploader::BatchUploader::Collector(collector)))
        } else {
            let endpoint = self.agent_endpoint.as_slice();
            let agent = AgentAsyncClientUDP::new(endpoint).map_err::<Error, _>(Into::into)?;
            Ok((self.process, BatchUploader::Agent(agent)))
        }
    }

    #[cfg(all(feature = "wasm_collector_client", not(feature = "collector_client")))]
    fn init_uploader(self) -> Result<(Process, uploader::BatchUploader), TraceError> {
        if let Some(collector_endpoint) = self.collector_endpoint {
            let collector = CollectorAsyncClientHttp::new(
                collector_endpoint,
                self.collector_username,
                self.collector_password,
            )
            .map_err::<Error, _>(Into::into)?;
            Ok((self.process, uploader::BatchUploader::Collector(collector)))
        } else {
            let endpoint = self.agent_endpoint.as_slice();
            let agent = AgentAsyncClientUDP::new(endpoint).map_err::<Error, _>(Into::into)?;
            Ok((self.process, BatchUploader::Agent(agent)))
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

#[derive(Debug)]
#[cfg(feature = "isahc_collector_client")]
struct IsahcHttpClient(isahc::HttpClient);

#[async_trait]
#[cfg(feature = "isahc_collector_client")]
impl opentelemetry::sdk::export::trace::HttpClient for IsahcHttpClient {
    async fn send(
        &self,
        request: http::Request<Vec<u8>>,
    ) -> opentelemetry::sdk::export::trace::ExportResult {
        let res = self
            .0
            .send_async(request)
            .await
            .map_err::<crate::Error, _>(|err| {
                ::thrift::Error::from(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    err.to_string(),
                ))
                .into()
            })
            .map_err::<TraceError, _>(TraceError::from)?;

        if !res.status().is_success() {
            return Err(crate::Error::ThriftAgentError(::thrift::Error::from(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Expected success response, got {:?}", res.status()),
                ),
            ))
            .into());
        }

        Ok(())
    }
}

#[rustfmt::skip]
impl Into<jaeger::Tag> for KeyValue {
    fn into(self) -> jaeger::Tag {
        let KeyValue { key, value } = self;
        match value {
            Value::String(s) => jaeger::Tag::new(key.into(), jaeger::TagType::String, Some(s.into()), None, None, None, None),
            Value::F64(f) => jaeger::Tag::new(key.into(), jaeger::TagType::Double, None, Some(f.into()), None, None, None),
            Value::Bool(b) => jaeger::Tag::new(key.into(), jaeger::TagType::Bool, None, None, Some(b), None, None),
            Value::I64(i) => jaeger::Tag::new(key.into(), jaeger::TagType::Long, None, None, None, Some(i), None),
            // TODO: better Array handling, jaeger thrift doesn't support arrays
            v @ Value::Array(_) => jaeger::Tag::new(key.into(), jaeger::TagType::String, Some(v.to_string()), None, None, None, None),
        }
    }
}

impl Into<jaeger::Log> for Event {
    fn into(self) -> jaeger::Log {
        let timestamp = self
            .timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_micros() as i64;
        let mut event_set_via_attribute = false;
        let mut fields = self
            .attributes
            .into_iter()
            .map(|attr| {
                if attr.key.as_str() == "event" {
                    event_set_via_attribute = true;
                };
                attr.into()
            })
            .collect::<Vec<_>>();

        if !event_set_via_attribute {
            fields.push(Key::new("event").string(self.name).into());
        }

        jaeger::Log::new(timestamp, fields)
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
        operation_name: span.name,
        references: links_to_references(span.links),
        flags: span.span_context.trace_flags() as i32,
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
        tags: build_span_tags(
            span.attributes,
            if export_instrument_lib {
                Some(span.instrumentation_lib)
            } else {
                None
            },
            span.status_code,
            span.status_message,
            span.span_kind,
        ),
        logs: events_to_logs(span.message_events),
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
                .map(|(k, v)| KeyValue::new(k.clone(), v.clone()).into()),
        )
    }
}

fn build_span_tags(
    attrs: sdk::trace::EvictedHashMap,
    instrumentation_lib: Option<sdk::InstrumentationLibrary>,
    status_code: StatusCode,
    status_message: String,
    kind: SpanKind,
) -> Option<Vec<jaeger::Tag>> {
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

    if !user_overrides.span_kind {
        tags.push(Key::new(SPAN_KIND).string(kind.to_string()).into());
    }

    if !user_overrides.status_code {
        tags.push(KeyValue::new(STATUS_CODE, status_code as i64).into());
    }

    if status_code != StatusCode::Unset {
        // Ensure error status is set
        if status_code == StatusCode::Error {
            tags.push(Key::new(ERROR).bool(true).into());
        }
        tags.push(
            Key::new(OTEL_STATUS_CODE)
                .string::<&'static str>(status_code.as_str())
                .into(),
        );
        // set status message if there is one
        if !status_message.is_empty() {
            if !user_overrides.status_message {
                tags.push(
                    Key::new(STATUS_MESSAGE)
                        .string(status_message.clone())
                        .into(),
                );
            }
            tags.push(
                Key::new(OTEL_STATUS_DESCRIPTION)
                    .string(status_message)
                    .into(),
            );
        }
    }

    Some(tags)
}

const ERROR: &str = "error";
const SPAN_KIND: &str = "span.kind";
const STATUS_CODE: &str = "status.code";
const STATUS_MESSAGE: &str = "status.message";
const OTEL_STATUS_CODE: &str = "otel.status_code";
const OTEL_STATUS_DESCRIPTION: &str = "otel.status_description";

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
}

impl ExportError for Error {
    fn exporter_name(&self) -> &'static str {
        "jaeger"
    }
}

#[cfg(test)]
#[cfg(feature = "collector_client")]
mod collector_client_tests {
    use crate::exporter::thrift::jaeger::Batch;
    use crate::new_pipeline;
    use opentelemetry::trace::TraceError;

    mod test_http_client {
        use async_trait::async_trait;
        use http::Request;
        use opentelemetry::sdk::export::trace::{ExportResult, HttpClient};
        use opentelemetry::trace::TraceError;
        use std::fmt::Debug;

        pub(crate) struct TestHttpClient;

        impl Debug for TestHttpClient {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("test http client")
            }
        }

        #[async_trait]
        impl HttpClient for TestHttpClient {
            async fn send(&self, _request: Request<Vec<u8>>) -> ExportResult {
                Err(TraceError::from("wrong uri set in http client"))
            }
        }
    }

    #[test]
    fn test_bring_your_own_client() -> Result<(), TraceError> {
        let (process, mut uploader) = new_pipeline()
            .with_collector_endpoint("localhost:6831")
            .with_http_client(test_http_client::TestHttpClient)
            .init_uploader()?;
        let res = futures::executor::block_on(async {
            uploader
                .upload(Batch::new(process.into(), Vec::new()))
                .await
        });
        assert_eq!(
            format!("{:?}", res.err().unwrap()),
            "Other(Custom(\"wrong uri set in http client\"))"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::exporter::thrift::jaeger::Tag;
    use crate::exporter::{build_span_tags, OTEL_STATUS_CODE, OTEL_STATUS_DESCRIPTION};
    use opentelemetry::sdk::trace::EvictedHashMap;
    use opentelemetry::trace::{SpanKind, StatusCode};

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
    fn test_set_status() -> Result<(), Box<dyn std::error::Error>> {
        for (status_code, error_msg, status_tag_val, msg_tag_val) in get_error_tag_test_data() {
            let tags = build_span_tags(
                EvictedHashMap::new(20, 20),
                None,
                status_code,
                error_msg,
                SpanKind::Client,
            )
            .unwrap_or_default();
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
        Ok(())
    }
}
