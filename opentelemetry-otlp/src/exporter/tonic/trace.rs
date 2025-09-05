use core::fmt;
use std::sync::Arc;
use tokio::sync::Mutex;

use opentelemetry::otel_debug;
use opentelemetry_proto::tonic::collector::trace::v1::{
    trace_service_client::TraceServiceClient, ExportTraceServiceRequest,
};
use opentelemetry_proto::transform::trace::tonic::group_spans_by_resource_and_scope;
use opentelemetry_sdk::error::OTelSdkError;
use opentelemetry_sdk::{
    error::OTelSdkResult,
    trace::{SpanData, SpanExporter},
};
use tonic::{codegen::CompressionEncoding, service::Interceptor, transport::Channel, Request};

use super::BoxInterceptor;

use crate::retry_classification::grpc::classify_tonic_status;
use opentelemetry_sdk::retry::{retry_with_backoff, RetryPolicy};
use opentelemetry_sdk::runtime::Tokio;

pub(crate) struct TonicTracesClient {
    inner: Option<ClientInner>,
    retry_policy: RetryPolicy,
    #[allow(dead_code)]
    // <allow dead> would be removed once we support set_resource for metrics.
    resource: opentelemetry_proto::transform::common::tonic::ResourceAttributesWithSchema,
}

struct ClientInner {
    client: TraceServiceClient<Channel>,
    interceptor: Mutex<BoxInterceptor>,
}

impl fmt::Debug for TonicTracesClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TonicTracesClient")
    }
}

impl TonicTracesClient {
    pub(super) fn new(
        channel: Channel,
        interceptor: BoxInterceptor,
        compression: Option<CompressionEncoding>,
        retry_policy: Option<RetryPolicy>,
    ) -> Self {
        let mut client = TraceServiceClient::new(channel);
        if let Some(compression) = compression {
            client = client
                .send_compressed(compression)
                .accept_compressed(compression);
        }

        otel_debug!(name: "TonicsTracesClientBuilt");

        TonicTracesClient {
            inner: Some(ClientInner {
                client,
                interceptor: Mutex::new(interceptor),
            }),
            retry_policy: retry_policy.unwrap_or(RetryPolicy {
                max_retries: 3,
                initial_delay_ms: 100,
                max_delay_ms: 1600,
                jitter_ms: 100,
            }),
            resource: Default::default(),
        }
    }
}

impl SpanExporter for TonicTracesClient {
    async fn export(&self, batch: Vec<SpanData>) -> OTelSdkResult {
        let batch = Arc::new(batch);

        match retry_with_backoff(
            Tokio,
            self.retry_policy.clone(),
            classify_tonic_status,
            "TonicTracesClient.Export",
            || async {
                let batch_clone = Arc::clone(&batch);

                // Execute the export operation
                let (mut client, metadata, extensions) = match &self.inner {
                    Some(inner) => {
                        let (m, e, _) = inner
                            .interceptor
                            .lock()
                            .await // tokio::sync::Mutex doesn't return a poisoned error, so we can safely use the interceptor here
                            .call(Request::new(()))
                            .map_err(|e| {
                                // Convert interceptor errors to tonic::Status for retry classification
                                tonic::Status::internal(format!("interceptor error: {e:?}"))
                            })?
                            .into_parts();
                        (inner.client.clone(), m, e)
                    }
                    None => {
                        return Err(tonic::Status::failed_precondition(
                            "exporter already shutdown",
                        ))
                    }
                };

                let resource_spans =
                    group_spans_by_resource_and_scope((*batch_clone).clone(), &self.resource);

                otel_debug!(name: "TonicTracesClient.ExportStarted");

                client
                    .export(Request::from_parts(
                        metadata,
                        extensions,
                        ExportTraceServiceRequest { resource_spans },
                    ))
                    .await
                    .map(|_| {
                        otel_debug!(name: "TonicTracesClient.ExportSucceeded");
                    })
            },
        )
        .await
        {
            Ok(_) => Ok(()),
            Err(tonic_status) => Err(OTelSdkError::InternalFailure(format!(
                "export error: {tonic_status:?}"
            ))),
        }
    }

    fn shutdown(&mut self) -> OTelSdkResult {
        match self.inner.take() {
            Some(_) => Ok(()), // Successfully took `inner`, indicating a successful shutdown.
            None => Err(OTelSdkError::AlreadyShutdown), // `inner` was already `None`, meaning it's already shut down.
        }
    }

    fn set_resource(&mut self, resource: &opentelemetry_sdk::Resource) {
        self.resource = resource.into();
    }
}
