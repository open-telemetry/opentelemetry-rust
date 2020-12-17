/*
   Copyright 2020 Vivint Smarthome

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

#![cfg(not(doctest))]
// unfortunately the proto code includes comments from the google proto files
// that are interpreted as "doc tests" and will fail to build.
// When this PR is merged we should be able to remove this attribute:
// https://github.com/danburkert/prost/pull/291

use async_trait::async_trait;
use futures::stream::StreamExt;
use opentelemetry::{
  exporter::trace::{ExportResult, SpanData, SpanExporter},
  Value,
};
use proto::google::devtools::cloudtrace::v2::BatchWriteSpansRequest;
use std::{
  fmt,
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
  },
  time::{Duration, Instant},
};
#[cfg(feature = "yup-authorizer")]
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
    pub mod protobuf {
      tonic::include_proto!("google.protobuf");
    }
    pub mod rpc {
      tonic::include_proto!("google.rpc");
    }
  }
}

use proto::google::devtools::cloudtrace::v2::{
  span::{time_event::Annotation, TimeEvent},
  trace_service_client::TraceServiceClient,
  AttributeValue, TruncatableString,
};

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
  /// If `num_concurrent_requests` is set to `0` or `None` then no limit is enforced.
  pub async fn connect<S: futures::task::Spawn>(
    authenticator: impl Authorizer,
    spawn: &S,
    maximum_shutdown_duration: Option<Duration>,
    num_concurrent_requests: impl Into<Option<usize>>,
  ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
    let num_concurrent_requests = num_concurrent_requests.into();
    let uri = http::uri::Uri::from_static("https://cloudtrace.googleapis.com:443");

    let mut rustls_config = rustls::ClientConfig::new();
    rustls_config.root_store.add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
    rustls_config.set_protocols(&[Vec::from("h2".as_bytes())]);
    let tls_config = ClientTlsConfig::new().rustls_client_config(rustls_config);

    let channel = Channel::builder(uri).tls_config(tls_config)?.connect().await?;
    let (tx, rx) = futures::channel::mpsc::channel(64);
    let pending_count = Arc::new(AtomicUsize::new(0));
    spawn.spawn_obj(
      Box::new(Self::export_inner(
        TraceServiceClient::new(channel),
        authenticator,
        rx,
        pending_count.clone(),
        num_concurrent_requests,
      ))
      .into(),
    )?;

    Ok(Self {
      tx,
      pending_count,
      maximum_shutdown_duration: maximum_shutdown_duration.unwrap_or(Duration::from_secs(5)),
    })
  }

  pub fn pending_count(&self) -> usize {
    self.pending_count.load(Ordering::Relaxed)
  }

  async fn export_inner(
    client: TraceServiceClient<Channel>,
    authorizer: impl Authorizer,
    rx: futures::channel::mpsc::Receiver<Vec<SpanData>>,
    pending_count: Arc<AtomicUsize>,
    num_concurrent: impl Into<Option<usize>>,
  ) {
    let authorizer = &authorizer;
    rx.for_each_concurrent(num_concurrent, move |batch| {
      let mut client = client.clone(); // This clone is cheap and allows for concurrent requests (see https://github.com/hyperium/tonic/issues/285#issuecomment-595880400)
      let pending_count = pending_count.clone();
      async move {
        use proto::google::devtools::cloudtrace::v2::{
          span::{time_event::Value, Attributes, TimeEvents},
          Span,
        };

        let spans = batch
          .into_iter()
          .map(|span| {
            let new_attributes = Attributes {
              attribute_map: span
                .attributes
                .iter()
                .map(|(key, value)| (key.as_str().to_owned(), value.clone().into()))
                .collect(),
              ..Default::default()
            };
            let new_time_events = TimeEvents {
              time_event: span
                .message_events
                .into_iter()
                .map(|event| TimeEvent {
                  time: Some(event.timestamp.into()),
                  value: Some(Value::Annotation(Annotation {
                    description: Some(to_truncate(event.name)),
                    ..Default::default()
                  })),
                })
                .collect(),
              ..Default::default()
            };

            Span {
              name: format!(
                "projects/{}/traces/{}/spans/{}",
                authorizer.project_id(),
                hex::encode(span.span_context.trace_id().to_u128().to_be_bytes()),
                hex::encode(span.span_context.span_id().to_u64().to_be_bytes())
              ),
              display_name: Some(to_truncate(span.name.clone())),
              span_id: hex::encode(span.span_context.span_id().to_u64().to_be_bytes()),
              parent_span_id: hex::encode(span.parent_span_id.to_u64().to_be_bytes()),
              start_time: Some(span.start_time.into()),
              end_time: Some(span.end_time.into()),
              attributes: Some(new_attributes),
              time_events: Some(new_time_events),
              ..Default::default()
            }
          })
          .collect::<Vec<_>>();

        let mut req = Request::new(BatchWriteSpansRequest {
          name: format!("projects/{}", authorizer.project_id()),
          spans,
        });

        pending_count.fetch_sub(1, Ordering::Relaxed);
        if let Err(e) = authorizer.authorize(&mut req).await {
          log::error!("StackDriver authentication failed {}", e);
          return;
        } else if let Err(e) = client.batch_write_spans(req).await {
          log::error!("StackDriver push failed {}", e);
          return;
        }
      }
    })
    .await;
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
    while (Instant::now() - start) < self.maximum_shutdown_duration && self.pending_count() > 0 {
      std::thread::yield_now();
      // Spin for a bit and give the inner export some time to upload, with a timeout.
    }
  }
}

#[async_trait]
pub trait Authorizer: Sync + Send + 'static {
  type Error: fmt::Display + fmt::Debug + Send;

  fn project_id(&self) -> &str;
  async fn authorize<T: Send + Sync>(&self, request: &mut Request<T>) -> Result<(), Self::Error>;
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
    let project_id = service_account_key.project_id.as_ref().ok_or("project_id is missing")?.clone();
    let mut authenticator = yup_oauth2::ServiceAccountAuthenticator::builder(service_account_key);
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

  async fn authorize<T: Send + Sync>(&self, req: &mut Request<T>) -> Result<(), Self::Error> {
    let scopes = &["https://www.googleapis.com/auth/trace.append"];
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
    Ok(Self { manager, project_id })
  }
}

#[cfg(feature = "gcp_auth")]
#[async_trait]
impl Authorizer for GcpAuthorizer {
  type Error = Box<dyn std::error::Error + Sync + Send>;

  fn project_id(&self) -> &str {
    &self.project_id
  }

  async fn authorize<T: Send + Sync>(&self, req: &mut Request<T>) -> Result<(), Self::Error> {
    let token = self.manager.get_token(&["https://www.googleapis.com/auth/trace.append"]).await?;
    req.metadata_mut().insert(
      "authorization",
      MetadataValue::from_str(&format!("Bearer {}", token.as_str())).unwrap(),
    );
    Ok(())
  }
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
    AttributeValue { value: Some(new_value) }
  }
}

fn to_truncate(s: String) -> TruncatableString {
  TruncatableString {
    value: s,
    ..Default::default()
  }
}
