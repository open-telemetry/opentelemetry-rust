#![cfg(not(doctest))]
// unfortunately the proto code includes comments from the google proto files
// that are interpreted as "doc tests" and will fail to build.
// When this PR is merged we should be able to remove this attribute:
// https://github.com/danburkert/prost/pull/291
#![allow(
    rustdoc::bare_urls,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_rust_codeblocks
)]

use std::{
    collections::HashMap,
    convert::TryFrom,
    fmt,
    future::Future,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use async_trait::async_trait;
use futures::{future::BoxFuture, stream::StreamExt};
use opentelemetry::{
    global::handle_error,
    sdk::{
        export::{
            trace::{ExportResult, SpanData, SpanExporter},
            ExportError,
        },
        trace::EvictedHashMap,
    },
    trace::TraceError,
    Key, Value,
};
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use opentelemetry_semantic_conventions::trace::{
    HTTP_METHOD, HTTP_ROUTE, HTTP_STATUS_CODE, HTTP_TARGET, HTTP_URL, HTTP_USER_AGENT,
};
use thiserror::Error;
#[cfg(any(feature = "yup-authorizer", feature = "gcp_auth"))]
use tonic::metadata::MetadataValue;
use tonic::{
    transport::{Channel, ClientTlsConfig},
    Request,
};
#[cfg(feature = "yup-authorizer")]
use yup_oauth2::authenticator::Authenticator;

#[allow(clippy::derive_partial_eq_without_eq)] // tonic doesn't derive Eq for generated types
pub mod proto;

const HTTP_HOST: Key = Key::from_static_str("http.host");

use proto::devtools::cloudtrace::v2::BatchWriteSpansRequest;
use proto::devtools::cloudtrace::v2::{
    span::{time_event::Annotation, Attributes, TimeEvent, TimeEvents},
    trace_service_client::TraceServiceClient,
    AttributeValue, Span, TruncatableString,
};
use proto::logging::v2::{
    log_entry::Payload, logging_service_v2_client::LoggingServiceV2Client, LogEntry,
    LogEntrySourceLocation, WriteLogEntriesRequest,
};

/// Exports opentelemetry tracing spans to Google StackDriver.
///
/// As of the time of this writing, the opentelemetry crate exposes no link information
/// so this struct does not send link information.
#[derive(Clone)]
pub struct StackDriverExporter {
    tx: futures::channel::mpsc::Sender<Vec<SpanData>>,
    pending_count: Arc<AtomicUsize>,
    maximum_shutdown_duration: Duration,
}

impl StackDriverExporter {
    pub fn builder() -> Builder {
        Builder::default()
    }

    pub fn pending_count(&self) -> usize {
        self.pending_count.load(Ordering::Relaxed)
    }
}

impl SpanExporter for StackDriverExporter {
    fn export(&mut self, batch: Vec<SpanData>) -> BoxFuture<'static, ExportResult> {
        match self.tx.try_send(batch) {
            Err(e) => Box::pin(std::future::ready(Err(e.into()))),
            Ok(()) => {
                self.pending_count.fetch_add(1, Ordering::Relaxed);
                Box::pin(std::future::ready(Ok(())))
            }
        }
    }

    fn shutdown(&mut self) {
        let start = Instant::now();
        while (Instant::now() - start) < self.maximum_shutdown_duration && self.pending_count() > 0
        {
            std::thread::yield_now();
            // Spin for a bit and give the inner export some time to upload, with a timeout.
        }
    }
}

impl fmt::Debug for StackDriverExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[allow(clippy::unneeded_field_pattern)]
        let Self {
            tx: _,
            pending_count,
            maximum_shutdown_duration,
        } = self;
        f.debug_struct("StackDriverExporter")
            .field("tx", &"(elided)")
            .field("pending_count", pending_count)
            .field("maximum_shutdown_duration", maximum_shutdown_duration)
            .finish()
    }
}

/// Helper type to build a `StackDriverExporter`.
#[derive(Clone, Default)]
pub struct Builder {
    maximum_shutdown_duration: Option<Duration>,
    num_concurrent_requests: Option<usize>,
    log_context: Option<LogContext>,
}

impl Builder {
    /// Set the number of concurrent requests to send to StackDriver.
    pub fn maximum_shutdown_duration(mut self, duration: Duration) -> Self {
        self.maximum_shutdown_duration = Some(duration);
        self
    }

    /// Set the number of concurrent requests.
    ///
    /// If `num_concurrent_requests` is set to `0` or `None` then no limit is enforced.
    pub fn num_concurrent_requests(mut self, num_concurrent_requests: usize) -> Self {
        self.num_concurrent_requests = Some(num_concurrent_requests);
        self
    }

    /// Enable writing log entries with the given `log_context`.
    pub fn log_context(mut self, log_context: LogContext) -> Self {
        self.log_context = Some(log_context);
        self
    }

    pub async fn build<A: Authorizer>(
        self,
        authenticator: A,
    ) -> Result<(StackDriverExporter, impl Future<Output = ()>), Error>
    where
        Error: From<A::Error>,
    {
        let Self {
            maximum_shutdown_duration,
            num_concurrent_requests,
            log_context,
        } = self;
        let uri = http::uri::Uri::from_static("https://cloudtrace.googleapis.com:443");

        let trace_channel = Channel::builder(uri)
            .tls_config(ClientTlsConfig::new())
            .map_err(|e| Error::Transport(e.into()))?
            .connect()
            .await
            .map_err(|e| Error::Transport(e.into()))?;

        let log_client = match log_context {
            Some(log_context) => {
                let log_channel = Channel::builder(http::uri::Uri::from_static(
                    "https://logging.googleapis.com:443",
                ))
                .tls_config(ClientTlsConfig::new())
                .map_err(|e| Error::Transport(e.into()))?
                .connect()
                .await
                .map_err(|e| Error::Transport(e.into()))?;

                Some(LogClient {
                    client: LoggingServiceV2Client::new(log_channel),
                    context: Arc::new(InternalLogContext::from(log_context)),
                })
            }
            None => None,
        };

        let (tx, rx) = futures::channel::mpsc::channel(64);
        let pending_count = Arc::new(AtomicUsize::new(0));
        let scopes = Arc::new(match log_client {
            Some(_) => vec![TRACE_APPEND, LOGGING_WRITE],
            None => vec![TRACE_APPEND],
        });

        let count_clone = pending_count.clone();
        let future = async move {
            let trace_client = TraceServiceClient::new(trace_channel);
            let authorizer = &authenticator;
            let log_client = log_client.clone();
            rx.for_each_concurrent(num_concurrent_requests, move |batch| {
                let trace_client = trace_client.clone();
                let log_client = log_client.clone();
                let pending_count = count_clone.clone();
                let scopes = scopes.clone();
                ExporterContext {
                    trace_client,
                    log_client,
                    authorizer,
                    pending_count,
                    scopes,
                }
                .export(batch)
            })
            .await
        };

        let exporter = StackDriverExporter {
            tx,
            pending_count,
            maximum_shutdown_duration: maximum_shutdown_duration
                .unwrap_or_else(|| Duration::from_secs(5)),
        };

        Ok((exporter, future))
    }
}

struct ExporterContext<'a, A> {
    trace_client: TraceServiceClient<Channel>,
    log_client: Option<LogClient>,
    authorizer: &'a A,
    pending_count: Arc<AtomicUsize>,
    scopes: Arc<Vec<&'static str>>,
}

impl<A: Authorizer> ExporterContext<'_, A>
where
    Error: From<A::Error>,
{
    async fn export(mut self, batch: Vec<SpanData>) {
        use proto::devtools::cloudtrace::v2::span::time_event::Value;

        let mut entries = Vec::new();
        let mut spans = Vec::with_capacity(batch.len());
        for span in batch {
            let trace_id = hex::encode(span.span_context.trace_id().to_bytes());
            let span_id = hex::encode(span.span_context.span_id().to_bytes());
            let time_event = match &self.log_client {
                None => span
                    .events
                    .into_iter()
                    .map(|event| TimeEvent {
                        time: Some(event.timestamp.into()),
                        value: Some(Value::Annotation(Annotation {
                            description: Some(to_truncate(event.name.into_owned())),
                            ..Default::default()
                        })),
                    })
                    .collect(),
                Some(client) => {
                    entries.extend(span.events.into_iter().map(|event| {
                        let (mut level, mut target, mut labels) =
                            (LogSeverity::Default, None, HashMap::default());
                        for kv in event.attributes {
                            match kv.key.as_str() {
                                "level" => {
                                    level = match kv.value.as_str().as_ref() {
                                        "DEBUG" | "TRACE" => LogSeverity::Debug,
                                        "INFO" => LogSeverity::Info,
                                        "WARN" => LogSeverity::Warning,
                                        "ERROR" => LogSeverity::Error,
                                        _ => LogSeverity::Default, // tracing::Level is limited to the above 5
                                    }
                                }
                                "target" => target = Some(kv.value.as_str().into_owned()),
                                key => {
                                    labels.insert(key.to_owned(), kv.value.as_str().into_owned());
                                }
                            }
                        }
                        let project_id = self.authorizer.project_id();
                        let log_id = &client.context.log_id;
                        LogEntry {
                            log_name: format!("projects/{project_id}/logs/{log_id}"),
                            resource: Some(client.context.resource.clone()),
                            severity: level as i32,
                            timestamp: Some(event.timestamp.into()),
                            labels,
                            trace: format!("projects/{project_id}/traces/{trace_id}"),
                            span_id: span_id.clone(),
                            source_location: target.map(|target| LogEntrySourceLocation {
                                file: String::new(),
                                line: 0,
                                function: target,
                            }),
                            payload: Some(Payload::TextPayload(event.name.into_owned())),
                            // severity, source_location, text_payload
                            ..Default::default()
                        }
                    }));

                    vec![]
                }
            };

            spans.push(Span {
                name: format!(
                    "projects/{}/traces/{}/spans/{}",
                    self.authorizer.project_id(),
                    hex::encode(span.span_context.trace_id().to_bytes()),
                    hex::encode(span.span_context.span_id().to_bytes())
                ),
                display_name: Some(to_truncate(span.name.into_owned())),
                span_id: hex::encode(span.span_context.span_id().to_bytes()),
                parent_span_id: hex::encode(span.parent_span_id.to_bytes()),
                start_time: Some(span.start_time.into()),
                end_time: Some(span.end_time.into()),
                attributes: Some(span.attributes.into()),
                time_events: Some(TimeEvents {
                    time_event,
                    ..Default::default()
                }),
                ..Default::default()
            });
        }

        let mut req = Request::new(BatchWriteSpansRequest {
            name: format!("projects/{}", self.authorizer.project_id()),
            spans,
        });

        self.pending_count.fetch_sub(1, Ordering::Relaxed);
        if let Err(e) = self.authorizer.authorize(&mut req, &self.scopes).await {
            handle_error(TraceError::from(Error::Authorizer(e.into())));
        } else if let Err(e) = self.trace_client.batch_write_spans(req).await {
            handle_error(TraceError::from(Error::Transport(e.into())));
        }

        let client = match &mut self.log_client {
            Some(client) => client,
            None => return,
        };

        let mut req = Request::new(WriteLogEntriesRequest {
            log_name: format!(
                "projects/{}/logs/{}",
                self.authorizer.project_id(),
                client.context.log_id,
            ),
            entries,
            dry_run: false,
            labels: HashMap::default(),
            partial_success: true,
            resource: None,
        });

        if let Err(e) = self.authorizer.authorize(&mut req, &self.scopes).await {
            handle_error(TraceError::from(Error::from(e)));
        } else if let Err(e) = client.client.write_log_entries(req).await {
            handle_error(TraceError::from(Error::Transport(e.into())));
        }
    }
}

#[cfg(feature = "yup-authorizer")]
pub struct YupAuthorizer {
    authenticator: Authenticator<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    project_id: String,
}

#[cfg(feature = "yup-authorizer")]
impl YupAuthorizer {
    pub async fn new(
        credentials_path: impl AsRef<std::path::Path>,
        persistent_token_file: impl Into<Option<std::path::PathBuf>>,
    ) -> Result<Self, Error> {
        let service_account_key = yup_oauth2::read_service_account_key(&credentials_path).await?;
        let project_id = service_account_key
            .project_id
            .as_ref()
            .ok_or_else(|| Error::Other("project_id is missing".into()))?
            .clone();
        let mut authenticator =
            yup_oauth2::ServiceAccountAuthenticator::builder(service_account_key);
        if let Some(persistent_token_file) = persistent_token_file.into() {
            authenticator = authenticator.persist_tokens_to_disk(persistent_token_file);
        }

        Ok(Self {
            authenticator: authenticator.build().await?,
            project_id,
        })
    }
}

#[cfg(feature = "yup-authorizer")]
#[async_trait]
impl Authorizer for YupAuthorizer {
    type Error = Error;

    fn project_id(&self) -> &str {
        &self.project_id
    }

    async fn authorize<T: Send + Sync>(
        &self,
        req: &mut Request<T>,
        scopes: &[&str],
    ) -> Result<(), Self::Error> {
        let token = self
            .authenticator
            .token(scopes)
            .await
            .map_err(|e| Error::Authorizer(e.into()))?;

        let token = match token.token() {
            Some(token) => token,
            None => return Err(Error::Other("unable to access token contents".into())),
        };

        req.metadata_mut().insert(
            "authorization",
            MetadataValue::try_from(format!("Bearer {}", token)).unwrap(),
        );
        Ok(())
    }
}

#[cfg(feature = "gcp_auth")]
pub struct GcpAuthorizer {
    manager: gcp_auth::AuthenticationManager,
    project_id: String,
}

#[cfg(feature = "gcp_auth")]
impl GcpAuthorizer {
    pub async fn new() -> Result<Self, Error> {
        let manager = gcp_auth::AuthenticationManager::new()
            .await
            .map_err(|e| Error::Authorizer(e.into()))?;

        let project_id = manager
            .project_id()
            .await
            .map_err(|e| Error::Authorizer(e.into()))?;

        Ok(Self {
            manager,
            project_id,
        })
    }
}

#[cfg(feature = "gcp_auth")]
#[async_trait]
impl Authorizer for GcpAuthorizer {
    type Error = Error;

    fn project_id(&self) -> &str {
        &self.project_id
    }

    async fn authorize<T: Send + Sync>(
        &self,
        req: &mut Request<T>,
        scopes: &[&str],
    ) -> Result<(), Self::Error> {
        let token = self
            .manager
            .get_token(scopes)
            .await
            .map_err(|e| Error::Authorizer(e.into()))?;

        req.metadata_mut().insert(
            "authorization",
            MetadataValue::try_from(format!("Bearer {}", token.as_str())).unwrap(),
        );

        Ok(())
    }
}

#[async_trait]
pub trait Authorizer: Sync + Send + 'static {
    type Error: std::error::Error + fmt::Debug + Send + Sync;

    fn project_id(&self) -> &str;
    async fn authorize<T: Send + Sync>(
        &self,
        request: &mut Request<T>,
        scopes: &[&str],
    ) -> Result<(), Self::Error>;
}

impl From<Value> for AttributeValue {
    fn from(v: Value) -> AttributeValue {
        use proto::devtools::cloudtrace::v2::attribute_value;
        let new_value = match v {
            Value::Bool(v) => attribute_value::Value::BoolValue(v),
            Value::F64(v) => attribute_value::Value::StringValue(to_truncate(v.to_string())),
            Value::I64(v) => attribute_value::Value::IntValue(v),
            Value::String(v) => attribute_value::Value::StringValue(to_truncate(v.to_string())),
            Value::Array(_) => attribute_value::Value::StringValue(to_truncate(v.to_string())),
        };
        AttributeValue {
            value: Some(new_value),
        }
    }
}

fn to_truncate(s: String) -> TruncatableString {
    TruncatableString {
        value: s,
        ..Default::default()
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("authorizer error: {0}")]
    Authorizer(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("tonic error: {0}")]
    Transport(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl ExportError for Error {
    fn exporter_name(&self) -> &'static str {
        "stackdriver"
    }
}

/// As defined in https://cloud.google.com/logging/docs/reference/v2/rpc/google.logging.type#google.logging.type.LogSeverity.
enum LogSeverity {
    Default = 0,
    Debug = 100,
    Info = 200,
    Warning = 400,
    Error = 500,
}

#[derive(Clone)]
struct LogClient {
    client: LoggingServiceV2Client<Channel>,
    context: Arc<InternalLogContext>,
}

struct InternalLogContext {
    log_id: String,
    resource: proto::api::MonitoredResource,
}

#[derive(Clone)]
pub struct LogContext {
    pub log_id: String,
    pub resource: MonitoredResource,
}

impl From<LogContext> for InternalLogContext {
    fn from(cx: LogContext) -> Self {
        let mut labels = HashMap::default();
        let resource = match cx.resource {
            MonitoredResource::CloudRunRevision {
                project_id,
                service_name,
                revision_name,
                location,
                configuration_name,
            } => {
                labels.insert("project_id".to_string(), project_id);
                if let Some(service_name) = service_name {
                    labels.insert("service_name".to_string(), service_name);
                }
                if let Some(revision_name) = revision_name {
                    labels.insert("revision_name".to_string(), revision_name);
                }
                if let Some(location) = location {
                    labels.insert("location".to_string(), location);
                }
                if let Some(configuration_name) = configuration_name {
                    labels.insert("configuration_name".to_string(), configuration_name);
                }

                proto::api::MonitoredResource {
                    r#type: "cloud_run_revision".to_owned(),
                    labels,
                }
            }
            MonitoredResource::GenericNode {
                project_id,
                location,
                namespace,
                node_id,
            } => {
                labels.insert("project_id".to_string(), project_id);
                if let Some(location) = location {
                    labels.insert("location".to_string(), location);
                }
                if let Some(namespace) = namespace {
                    labels.insert("namespace".to_string(), namespace);
                }
                if let Some(node_id) = node_id {
                    labels.insert("node_id".to_string(), node_id);
                }

                proto::api::MonitoredResource {
                    r#type: "generic_node".to_owned(),
                    labels,
                }
            }
            MonitoredResource::GenericTask {
                project_id,
                location,
                namespace,
                job,
                task_id,
            } => {
                labels.insert("project_id".to_owned(), project_id);
                if let Some(location) = location {
                    labels.insert("location".to_owned(), location);
                }
                if let Some(namespace) = namespace {
                    labels.insert("namespace".to_owned(), namespace);
                }
                if let Some(job) = job {
                    labels.insert("job".to_owned(), job);
                }
                if let Some(task_id) = task_id {
                    labels.insert("task_id".to_owned(), task_id);
                }

                proto::api::MonitoredResource {
                    r#type: "generic_task".to_owned(),
                    labels,
                }
            }
            MonitoredResource::Global { project_id } => {
                labels.insert("project_id".to_owned(), project_id);
                proto::api::MonitoredResource {
                    r#type: "global".to_owned(),
                    labels,
                }
            }
        };

        Self {
            log_id: cx.log_id,
            resource,
        }
    }
}

/// A description of a `MonitoredResource`.
///
/// Possible values are listed in the [API documentation](https://cloud.google.com/logging/docs/api/v2/resource-list).
/// Please submit an issue or pull request if you want to use a resource type not listed here.
#[derive(Clone)]
pub enum MonitoredResource {
    Global {
        project_id: String,
    },
    GenericNode {
        project_id: String,
        location: Option<String>,
        namespace: Option<String>,
        node_id: Option<String>,
    },
    GenericTask {
        project_id: String,
        location: Option<String>,
        namespace: Option<String>,
        job: Option<String>,
        task_id: Option<String>,
    },
    CloudRunRevision {
        project_id: String,
        service_name: Option<String>,
        revision_name: Option<String>,
        location: Option<String>,
        configuration_name: Option<String>,
    },
}

impl From<EvictedHashMap> for Attributes {
    fn from(attributes: EvictedHashMap) -> Self {
        let mut dropped_attributes_count: i32 = 0;
        let attribute_map = attributes
            .into_iter()
            .flat_map(|(k, v)| {
                let key = k.as_str();
                if key.len() > 128 {
                    dropped_attributes_count += 1;
                    return None;
                }

                if k == SERVICE_NAME {
                    return Some((GCP_SERVICE_NAME.to_owned(), v.into()));
                } else if key == HTTP_PATH_ATTRIBUTE {
                    return Some((GCP_HTTP_PATH.to_owned(), v.into()));
                }

                for (otel_key, gcp_key) in KEY_MAP {
                    if otel_key == &k {
                        return Some((gcp_key.to_owned(), v.into()));
                    }
                }

                Some((key.to_owned(), v.into()))
            })
            .collect();
        Attributes {
            attribute_map,
            dropped_attributes_count,
        }
    }
}

// Map conventional OpenTelemetry keys to their GCP counterparts.
const KEY_MAP: [(&Key, &str); 7] = [
    (&HTTP_HOST, "/http/host"),
    (&HTTP_METHOD, "/http/method"),
    (&HTTP_TARGET, "/http/path"),
    (&HTTP_URL, "/http/url"),
    (&HTTP_USER_AGENT, "/http/user_agent"),
    (&HTTP_STATUS_CODE, "/http/status_code"),
    (&HTTP_ROUTE, "/http/route"),
];

const TRACE_APPEND: &str = "https://www.googleapis.com/auth/trace.append";
const LOGGING_WRITE: &str = "https://www.googleapis.com/auth/logging.write";
const HTTP_PATH_ATTRIBUTE: &str = "http.path";
const GCP_HTTP_PATH: &str = "/http/path";
const GCP_SERVICE_NAME: &str = "g.co/gae/app/module";

#[cfg(test)]
mod tests {
    use super::*;

    use opentelemetry::{sdk::trace::EvictedHashMap, KeyValue, Value};
    use opentelemetry_semantic_conventions as semcov;

    #[test]
    fn test_attributes_mapping() {
        let capacity = 10;
        let mut attributes = EvictedHashMap::new(capacity, 0);

        //	hostAttribute       = "http.host"
        attributes.insert(HTTP_HOST.string("example.com:8080"));

        // 	methodAttribute     = "http.method"
        attributes.insert(semcov::trace::HTTP_METHOD.string("POST"));

        // 	pathAttribute       = "http.path"
        attributes.insert(KeyValue::new(
            "http.path",
            Value::String("/path/12314/?q=ddds#123".into()),
        ));

        // 	urlAttribute        = "http.url"
        attributes.insert(
            semcov::trace::HTTP_URL.string("https://example.com:8080/webshop/articles/4?s=1"),
        );

        // 	userAgentAttribute  = "http.user_agent"
        attributes
            .insert(semcov::trace::HTTP_USER_AGENT.string("CERN-LineMode/2.15 libwww/2.17b3"));

        // 	statusCodeAttribute = "http.status_code"
        attributes.insert(semcov::trace::HTTP_STATUS_CODE.i64(200));

        // 	statusCodeAttribute = "http.route"
        attributes.insert(semcov::trace::HTTP_ROUTE.string("/webshop/articles/:article_id"));

        // 	serviceAttribute    = "service.name"
        attributes.insert(semcov::resource::SERVICE_NAME.string("Test Service Name"));

        let actual: Attributes = attributes.into();

        assert_eq!(actual.attribute_map.len(), 8);
        assert_eq!(actual.dropped_attributes_count, 0);
        assert_eq!(
            actual.attribute_map.get("/http/host"),
            Some(&AttributeValue::from(Value::String(
                "example.com:8080".into()
            )))
        );
        assert_eq!(
            actual.attribute_map.get("/http/method"),
            Some(&AttributeValue::from(Value::String("POST".into()))),
        );
        assert_eq!(
            actual.attribute_map.get("/http/path"),
            Some(&AttributeValue::from(Value::String(
                "/path/12314/?q=ddds#123".into()
            ))),
        );
        assert_eq!(
            actual.attribute_map.get("/http/route"),
            Some(&AttributeValue::from(Value::String(
                "/webshop/articles/:article_id".into()
            ))),
        );
        assert_eq!(
            actual.attribute_map.get("/http/url"),
            Some(&AttributeValue::from(Value::String(
                "https://example.com:8080/webshop/articles/4?s=1".into(),
            ))),
        );
        assert_eq!(
            actual.attribute_map.get("/http/user_agent"),
            Some(&AttributeValue::from(Value::String(
                "CERN-LineMode/2.15 libwww/2.17b3".into()
            ))),
        );
        assert_eq!(
            actual.attribute_map.get("/http/status_code"),
            Some(&AttributeValue::from(Value::I64(200))),
        );
        assert_eq!(
            actual.attribute_map.get("g.co/gae/app/module"),
            Some(&AttributeValue::from(Value::String(
                "Test Service Name".into()
            ))),
        );
    }

    #[test]
    fn test_attributes_mapping_http_target() {
        let capacity = 10;
        let mut attributes = EvictedHashMap::new(capacity, 0);

        //	hostAttribute       = "http.target"
        attributes.insert(semcov::trace::HTTP_TARGET.string("/path/12314/?q=ddds#123"));

        let actual: Attributes = attributes.into();

        assert_eq!(actual.attribute_map.len(), 1);
        assert_eq!(actual.dropped_attributes_count, 0);
        assert_eq!(
            actual.attribute_map.get("/http/path"),
            Some(&AttributeValue::from(Value::String(
                "/path/12314/?q=ddds#123".into()
            ))),
        );
    }

    #[test]
    fn test_attributes_mapping_dropped_attributes_count() {
        let capacity = 10;
        let mut attributes = EvictedHashMap::new(capacity, 0);
        attributes.insert(KeyValue::new("answer", Value::I64(42)));
        attributes.insert(KeyValue::new("long_attribute_key_dvwmacxpeefbuemoxljmqvldjxmvvihoeqnuqdsyovwgljtnemouidabhkmvsnauwfnaihekcfwhugejboiyfthyhmkpsaxtidlsbwsmirebax", Value::String("Some value".into())));

        let actual: Attributes = attributes.into();
        assert_eq!(
            actual,
            Attributes {
                attribute_map: HashMap::from([(
                    "answer".into(),
                    AttributeValue::from(Value::I64(42))
                ),]),
                dropped_attributes_count: 1,
            }
        );
        assert_eq!(actual.attribute_map.len(), 1);
        assert_eq!(actual.dropped_attributes_count, 1);
    }
}
