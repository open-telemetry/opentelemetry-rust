use core::fmt;
use std::sync::Mutex;

use opentelemetry::{otel_debug, otel_warn};
use opentelemetry_proto::tonic::collector::metrics::v1::{
    metrics_service_client::MetricsServiceClient, ExportMetricsServiceRequest,
};
use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use tonic::{codegen::CompressionEncoding, service::Interceptor, transport::Channel, Request};

use super::BoxInterceptor;
use crate::metric::MetricsClient;

use crate::retry::RetryPolicy;
#[cfg(feature = "experimental-grpc-retry")]
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
        match super::tonic_retry_with_backoff(
            #[cfg(feature = "experimental-grpc-retry")]
            Tokio,
            #[cfg(not(feature = "experimental-grpc-retry"))]
            (),
            self.retry_policy.clone(),
            crate::retry_classification::grpc::classify_tonic_status,
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
                                    otel_warn!(
                                        name: "TonicMetricsClient.InterceptorFailed",
                                        grpc_code = format!("{:?}", e.code())
                                    );
                                    // grpc_message and grpc_details may contain sensitive information,
                                    // so log them at debug level only.
                                    otel_debug!(
                                        name: "TonicMetricsClient.InterceptorFailedDetails",
                                        grpc_message = e.message(),
                                        grpc_details = format!("{:?}", e.details())
                                    );
                                    tonic::Status::internal("Metrics export failed in interceptor")
                                })?
                                .into_parts();
                            Ok((inner.client.clone(), m, e))
                        }
                        None => Err(tonic::Status::failed_precondition(
                            "metrics exporter is already shut down",
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
                    .map(|response| {
                        otel_debug!(name: "TonicMetricsClient.ExportSucceeded");

                        // Handle partial success. As per spec, we log and _do not_ retry.
                        if let Some(partial_success) = response.into_inner().partial_success {
                            if partial_success.rejected_data_points > 0
                                || !partial_success.error_message.is_empty()
                            {
                                otel_warn!(
                                    name: "TonicMetricsClient.PartialSuccess",
                                    rejected_data_points = partial_success.rejected_data_points,
                                    error_message = partial_success.error_message.as_str(),
                                );
                            }
                        }
                    })
            },
        )
        .await
        {
            Ok(_) => Ok(()),
            Err(tonic_status) => {
                // For connection-related errors (Unavailable, Unknown, etc.), the message
                // typically contains safe, actionable information (e.g., "Connection refused").
                // For auth errors (Unauthenticated, PermissionDenied), the message may contain
                // sensitive information, so we only log the code at WARN level.
                let code = tonic_status.code();
                let is_connection_error = matches!(
                    code,
                    tonic::Code::Unavailable
                        | tonic::Code::Unknown
                        | tonic::Code::DeadlineExceeded
                        | tonic::Code::ResourceExhausted
                        | tonic::Code::Aborted
                        | tonic::Code::Cancelled
                );

                if is_connection_error {
                    otel_warn!(
                        name: "TonicMetricsClient.ExportFailed",
                        grpc_code = format!("{:?}", code),
                        grpc_message = tonic_status.message()
                    );
                } else {
                    // For potentially sensitive errors (Unauthenticated, PermissionDenied, etc.),
                    // only log the code at WARN level.
                    otel_warn!(
                        name: "TonicMetricsClient.ExportFailed",
                        grpc_code = format!("{:?}", code)
                    );
                    // Log message and details at debug level for sensitive error types.
                    otel_debug!(
                        name: "TonicMetricsClient.ExportFailedDetails",
                        grpc_message = tonic_status.message(),
                        grpc_details = format!("{:?}", tonic_status.details())
                    );
                }
                Err(OTelSdkError::InternalFailure(
                    "Metrics export failed".into(),
                ))
            }
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
