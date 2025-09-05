use core::fmt;
use std::sync::Mutex;

use opentelemetry::otel_debug;
use opentelemetry_proto::tonic::collector::metrics::v1::{
    metrics_service_client::MetricsServiceClient, ExportMetricsServiceRequest,
};
use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use tonic::{codegen::CompressionEncoding, service::Interceptor, transport::Channel, Request};

use super::BoxInterceptor;
use crate::metric::MetricsClient;

use crate::retry_classification::grpc::classify_tonic_status;
use opentelemetry_sdk::retry::{retry_with_backoff, RetryPolicy};
use opentelemetry_sdk::runtime::Tokio;

pub(crate) struct TonicMetricsClient {
    inner: Mutex<Option<ClientInner>>,
    retry_policy: RetryPolicy,
}

struct ClientInner {
    client: MetricsServiceClient<Channel>,
    interceptor: BoxInterceptor,
}

impl fmt::Debug for TonicMetricsClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TonicMetricsClient")
    }
}

impl TonicMetricsClient {
    pub(super) fn new(
        channel: Channel,
        interceptor: BoxInterceptor,
        compression: Option<CompressionEncoding>,
        retry_policy: Option<RetryPolicy>,
    ) -> Self {
        let mut client = MetricsServiceClient::new(channel);
        if let Some(compression) = compression {
            client = client
                .send_compressed(compression)
                .accept_compressed(compression);
        }

        otel_debug!(name: "TonicsMetricsClientBuilt");

        TonicMetricsClient {
            inner: Mutex::new(Some(ClientInner {
                client,
                interceptor,
            })),
            retry_policy: retry_policy.unwrap_or(RetryPolicy {
                max_retries: 3,
                initial_delay_ms: 100,
                max_delay_ms: 1600,
                jitter_ms: 100,
            }),
        }
    }
}

impl MetricsClient for TonicMetricsClient {
    async fn export(&self, metrics: &ResourceMetrics) -> OTelSdkResult {
        match retry_with_backoff(
            Tokio,
            self.retry_policy.clone(),
            classify_tonic_status,
            "TonicMetricsClient.Export",
            || async {
                // Execute the export operation
                let (mut client, metadata, extensions) = self
                    .inner
                    .lock()
                    .map_err(|e| tonic::Status::internal(format!("Failed to acquire lock: {e:?}")))
                    .and_then(|mut inner| match &mut *inner {
                        Some(inner) => {
                            let (m, e, _) = inner
                                .interceptor
                                .call(Request::new(()))
                                .map_err(|e| {
                                    tonic::Status::internal(format!(
                                        "unexpected status while exporting {e:?}"
                                    ))
                                })?
                                .into_parts();
                            Ok((inner.client.clone(), m, e))
                        }
                        None => Err(tonic::Status::failed_precondition(
                            "exporter is already shut down",
                        )),
                    })?;

                otel_debug!(name: "TonicMetricsClient.ExportStarted");

                client
                    .export(Request::from_parts(
                        metadata,
                        extensions,
                        ExportMetricsServiceRequest::from(metrics),
                    ))
                    .await
                    .map(|_| {
                        otel_debug!(name: "TonicMetricsClient.ExportSucceeded");
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

    fn shutdown(&self) -> OTelSdkResult {
        self.inner
            .lock()
            .map_err(|e| OTelSdkError::InternalFailure(format!("Failed to acquire lock: {e}")))?
            .take();

        Ok(())
    }
}
