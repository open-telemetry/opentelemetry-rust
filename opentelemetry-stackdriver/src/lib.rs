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
    fmt,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use async_trait::async_trait;
use futures::stream::StreamExt;
use opentelemetry::{
    sdk::export::trace::{ExportResult, SpanData, SpanExporter},
    Value,
};

#[cfg(any(feature = "yup-authorizer", feature = "gcp_auth"))]
use tonic::metadata::MetadataValue;
use tonic::{
    transport::{Channel, ClientTlsConfig},
    Request,
};
#[cfg(feature = "yup-authorizer")]
use yup_oauth2::authenticator::Authenticator;

pub mod proto {
    pub mod google {
        pub mod api {
            tonic::include_proto!("google.api");
        }
        pub mod devtools {
            pub mod cloudtrace {
                pub mod v2 {
                    tonic::include_proto!("google.devtools.cloudtrace.v2");
                }
            }
        }
        pub mod logging {
            pub mod r#type {
                tonic::include_proto!("google.logging.r#type");
            }
            pub mod v2 {
                tonic::include_proto!("google.logging.v2");
            }
        }
        pub mod protobuf {
            tonic::include_proto!("google.protobuf");
        }
        pub mod rpc {
            tonic::include_proto!("google.rpc");
        }
    }
}

use proto::google::devtools::cloudtrace::v2::BatchWriteSpansRequest;
use proto::google::devtools::cloudtrace::v2::{
    span::{time_event::Annotation, Attributes, TimeEvent, TimeEvents},
    trace_service_client::TraceServiceClient,
    AttributeValue, Span, TruncatableString,
};
use proto::google::logging::v2::{
    log_entry::Payload, logging_service_v2_client::LoggingServiceV2Client, LogEntry,
    LogEntrySourceLocation, WriteLogEntriesRequest,
};

use proto::google::api::MonitoredResource;

#[cfg(feature = "tokio_adapter")]
pub mod tokio_adapter;

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

impl fmt::Debug for StackDriverExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[allow(clippy::unneeded_field_pattern)]
        let Self {
            maximum_shutdown_duration,
            pending_count,
            tx: _,
        } = self;
        f.debug_struct("StackDriverExporter")
            .field("tx", &"(elided)")
            .field("pending_count", pending_count)
            .field("maximum_shutdown_duration", maximum_shutdown_duration)
            .finish()
    }
}

impl StackDriverExporter {
    pub fn builder() -> Builder {
        Builder::default()
    }

    pub fn pending_count(&self) -> usize {
        self.pending_count.load(Ordering::Relaxed)
    }
}

#[async_trait]
impl SpanExporter for StackDriverExporter {
    async fn export(&mut self, batch: Vec<SpanData>) -> ExportResult {
        match self.tx.try_send(batch) {
            Err(e) => {
                log::error!("Unable to send to export_inner {:?}", e);
                Err(e.into())
            }
            Ok(()) => {
                self.pending_count.fetch_add(1, Ordering::Relaxed);
                Ok(())
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

    pub async fn build<S: futures::task::Spawn>(
        self,
        authenticator: impl Authorizer,
        spawn: &S,
    ) -> Result<StackDriverExporter, Box<dyn std::error::Error + Send + Sync>> {
        let Self {
            maximum_shutdown_duration,
            num_concurrent_requests,
            log_context,
        } = self;
        let uri = http::uri::Uri::from_static("https://cloudtrace.googleapis.com:443");

        let trace_channel = Channel::builder(uri)
            .tls_config(ClientTlsConfig::new())?
            .connect()
            .await?;

        let log_channel = Channel::builder(http::uri::Uri::from_static(
            "https://logging.googleapis.com:443",
        ))
        .tls_config(ClientTlsConfig::new())?
        .connect()
        .await?;

        let log_client = log_context.map(|log_context| LogClient {
            client: LoggingServiceV2Client::new(log_channel),
            context: Arc::new(log_context),
        });

        let (tx, rx) = futures::channel::mpsc::channel(64);
        let pending_count = Arc::new(AtomicUsize::new(0));
        let scopes = Arc::new(match log_client {
            Some(_) => vec![TRACE_APPEND, LOGGING_WRITE],
            None => vec![TRACE_APPEND],
        });

        let count_clone = pending_count.clone();
        spawn.spawn_obj(
            Box::new(async move {
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
            })
            .into(),
        )?;

        Ok(StackDriverExporter {
            tx,
            pending_count,
            maximum_shutdown_duration: maximum_shutdown_duration
                .unwrap_or_else(|| Duration::from_secs(5)),
        })
    }
}

struct ExporterContext<'a, A> {
    trace_client: TraceServiceClient<Channel>,
    log_client: Option<LogClient>,
    authorizer: &'a A,
    pending_count: Arc<AtomicUsize>,
    scopes: Arc<Vec<&'static str>>,
}

impl<A: Authorizer> ExporterContext<'_, A> {
    async fn export(mut self, batch: Vec<SpanData>) {
        use proto::google::devtools::cloudtrace::v2::span::time_event::Value;

        let mut entries = Vec::new();
        let mut spans = Vec::with_capacity(batch.len());
        for span in batch {
            let attribute_map = span
                .attributes
                .into_iter()
                .map(|(key, value)| (key.as_str().to_owned(), value.into()))
                .collect();

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

                        LogEntry {
                            log_name: format!(
                                "projects/{}/logs/{}",
                                self.authorizer.project_id(),
                                client.context.log_id,
                            ),
                            resource: Some(MonitoredResource {
                                r#type: client.context.resource.r#type.clone(),
                                labels: client.context.resource.labels.clone(),
                            }),
                            severity: level as i32,
                            timestamp: Some(event.timestamp.into()),
                            labels,
                            trace: trace_id.clone(),
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
                attributes: Some(Attributes {
                    attribute_map,
                    ..Default::default()
                }),
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
            log::error!("StackDriver authentication failed {}", e);
        } else if let Err(e) = self.trace_client.batch_write_spans(req).await {
            log::error!("StackDriver push failed {}", e);
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
            log::error!("StackDriver authentication failed {}", e);
        } else if let Err(e) = client.client.write_log_entries(req).await {
            log::error!("StackDriver push failed {}", e);
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
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let service_account_key = yup_oauth2::read_service_account_key(&credentials_path).await?;
        let project_id = service_account_key
            .project_id
            .as_ref()
            .ok_or("project_id is missing")?
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
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn project_id(&self) -> &str {
        &self.project_id
    }

    async fn authorize<T: Send + Sync>(
        &self,
        req: &mut Request<T>,
        scopes: &[&str],
    ) -> Result<(), Self::Error> {
        let token = self.authenticator.token(scopes).await?;
        req.metadata_mut().insert(
            "authorization",
            MetadataValue::from_str(&format!("Bearer {}", token.as_str())).unwrap(),
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
    pub async fn new() -> Result<Self, gcp_auth::Error> {
        let manager = gcp_auth::init().await?;
        let project_id = manager.project_id().await?;
        Ok(Self {
            manager,
            project_id,
        })
    }
}

#[cfg(feature = "gcp_auth")]
#[async_trait]
impl Authorizer for GcpAuthorizer {
    type Error = Box<dyn std::error::Error + Sync + Send>;

    fn project_id(&self) -> &str {
        &self.project_id
    }

    async fn authorize<T: Send + Sync>(
        &self,
        req: &mut Request<T>,
        scopes: &[&str],
    ) -> Result<(), Self::Error> {
        let token = self.manager.get_token(scopes).await?;
        req.metadata_mut().insert(
            "authorization",
            MetadataValue::from_str(&format!("Bearer {}", token.as_str())).unwrap(),
        );
        Ok(())
    }
}

#[async_trait]
pub trait Authorizer: Sync + Send + 'static {
    type Error: fmt::Display + fmt::Debug + Send;

    fn project_id(&self) -> &str;
    async fn authorize<T: Send + Sync>(
        &self,
        request: &mut Request<T>,
        scopes: &[&str],
    ) -> Result<(), Self::Error>;
}

impl From<Value> for AttributeValue {
    fn from(v: Value) -> AttributeValue {
        use proto::google::devtools::cloudtrace::v2::attribute_value;
        let new_value = match v {
            Value::Bool(v) => attribute_value::Value::BoolValue(v),
            Value::F64(v) => attribute_value::Value::StringValue(to_truncate(v.to_string())),
            Value::I64(v) => attribute_value::Value::IntValue(v),
            Value::String(v) => attribute_value::Value::StringValue(to_truncate(v.into_owned())),
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
    context: Arc<LogContext>,
}

#[derive(Clone)]
pub struct LogContext {
    pub log_id: String,
    pub resource: Resource,
}

#[derive(Clone)]
pub struct Resource {
    pub r#type: String,
    pub labels: HashMap<String, String>,
}

const TRACE_APPEND: &str = "https://www.googleapis.com/auth/trace.append";
const LOGGING_WRITE: &str = "https://www.googleapis.com/auth/logging.write";
