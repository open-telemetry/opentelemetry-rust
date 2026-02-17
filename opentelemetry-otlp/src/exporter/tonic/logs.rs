use core::fmt;
use opentelemetry::{otel_debug, otel_warn};
use opentelemetry_proto::tonic::collector::logs::v1::{
    logs_service_client::LogsServiceClient, ExportLogsServiceRequest,
};
use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use opentelemetry_sdk::logs::{LogBatch, LogExporter};
use std::sync::{Arc, Mutex};
use std::time;
use tonic::{codegen::CompressionEncoding, service::Interceptor, transport::Channel, Request};

use opentelemetry_proto::transform::logs::tonic::group_logs_by_resource_and_scope;

use super::BoxInterceptor;

use crate::retry::RetryPolicy;
#[cfg(feature = "experimental-grpc-retry")]
use opentelemetry_sdk::runtime::Tokio;

pub(crate) struct TonicLogsClient {
    inner: Mutex<Option<ClientInner>>,
    retry_policy: RetryPolicy,
    #[allow(dead_code)]
    // <allow dead> would be removed once we support set_resource for metrics.
    resource: opentelemetry_proto::transform::common::tonic::ResourceAttributesWithSchema,
}

struct ClientInner {
    client: LogsServiceClient<Channel>,
    interceptor: BoxInterceptor,
}

impl fmt::Debug for TonicLogsClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TonicLogsClient")
    }
}

impl TonicLogsClient {
    pub(super) fn new(
        channel: Channel,
        interceptor: BoxInterceptor,
        compression: Option<CompressionEncoding>,
        retry_policy: Option<RetryPolicy>,
    ) -> Self {
        let mut client = LogsServiceClient::new(channel);
        if let Some(compression) = compression {
            client = client
                .send_compressed(compression)
                .accept_compressed(compression);
        }

        otel_debug!(name: "TonicsLogsClientBuilt");

        TonicLogsClient {
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
            resource: Default::default(),
        }
    }
}

impl LogExporter for TonicLogsClient {
    async fn export(&self, batch: LogBatch<'_>) -> OTelSdkResult {
        let batch = Arc::new(batch);

        match super::tonic_retry_with_backoff(
            #[cfg(feature = "experimental-grpc-retry")]
            Tokio,
            #[cfg(not(feature = "experimental-grpc-retry"))]
            (),
            self.retry_policy.clone(),
            crate::retry_classification::grpc::classify_tonic_status,
            "TonicLogsClient.Export",
            || async {
                let batch_clone = Arc::clone(&batch);

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
                                        name: "TonicLogsClient.InterceptorFailed",
                                        grpc_code = format!("{:?}", e.code())
                                    );
                                    // grpc_message and grpc_details may contain sensitive information,
                                    // so log them at debug level only.
                                    otel_debug!(
                                        name: "TonicLogsClient.InterceptorFailedDetails",
                                        grpc_message = e.message(),
                                        grpc_details = format!("{:?}", e.details())
                                    );
                                    // Convert interceptor errors to tonic::Status for retry classification
                                    tonic::Status::internal(&format!(
                                        "Logs export failed in interceptor with gRPC code: {:?}",
                                        e.code()
                                    ))
                                })?
                                .into_parts();
                            Ok((inner.client.clone(), m, e))
                        }
                        None => Err(tonic::Status::failed_precondition(
                            "log exporter is already shut down",
                        )),
                    })?;

                let resource_logs = group_logs_by_resource_and_scope(&batch_clone, &self.resource);

                otel_debug!(name: "TonicLogsClient.ExportStarted");

                client
                    .export(Request::from_parts(
                        metadata,
                        extensions,
                        ExportLogsServiceRequest { resource_logs },
                    ))
                    .await
                    .map(|response| {
                        otel_debug!(name: "TonicLogsClient.ExportSucceeded");

                        // Handle partial success. As per spec, we log and _do not_ retry.
                        if let Some(partial_success) = response.into_inner().partial_success {
                            if partial_success.rejected_log_records > 0
                                || !partial_success.error_message.is_empty()
                            {
                                otel_warn!(
                                    name: "TonicLogsClient.PartialSuccess",
                                    rejected_log_records = partial_success.rejected_log_records,
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
                        name: "TonicLogsClient.ExportFailed",
                        grpc_code = format!("{:?}", code),
                        grpc_message = tonic_status.message()
                    );
                } else {
                    // For potentially sensitive errors (Unauthenticated, PermissionDenied, etc.),
                    // only log the code at WARN level.
                    otel_warn!(
                        name: "TonicLogsClient.ExportFailed",
                        grpc_code = format!("{:?}", code)
                    );
                    // Log message and details at debug level for sensitive error types.
                    otel_debug!(
                        name: "TonicLogsClient.ExportFailedDetails",
                        grpc_message = tonic_status.message(),
                        grpc_details = format!("{:?}", tonic_status.details())
                    );
                }
                Err(OTelSdkError::InternalFailure(format!(
                    "Logs export failed with gRPC code: {:?}",
                    code
                )))
            }
        }
    }

    fn shutdown_with_timeout(&self, _timeout: time::Duration) -> OTelSdkResult {
        self.inner
            .lock()
            .map_err(|e| OTelSdkError::InternalFailure(format!("Failed to acquire lock: {e}")))?
            .take();

        Ok(())
    }

    fn set_resource(&mut self, resource: &opentelemetry_sdk::Resource) {
        self.resource = resource.into();
    }
}
