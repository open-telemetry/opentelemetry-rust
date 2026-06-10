use core::fmt;
use opentelemetry::{otel_debug, otel_warn};
use opentelemetry_proto::tonic::collector::logs::v1::{
    logs_service_client::LogsServiceClient, ExportLogsServiceRequest,
};
use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use opentelemetry_sdk::logs::{LogBatch, LogExporter};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time;
use std::time::Instant;
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
    component_name: String,
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

        let component_name = {
            static INSTANCE_COUNTER: AtomicUsize = AtomicUsize::new(0);
            let instance_id = INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
            format!("otlp_grpc_log_exporter/{instance_id}")
        };

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
            component_name,
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
                                    super::handle_interceptor_error!("TonicLogsClient", e)
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
                super::handle_tonic_export_error!("TonicLogsClient", tonic_status)
            }
        }
    }

    fn shutdown_with_timeout(&self, _timeout: time::Duration) -> OTelSdkResult {
        let shutdown_start = Instant::now();
        let was_first_shutdown = match self
            .inner
            .lock()
            .map_err(|e| OTelSdkError::InternalFailure(format!("Failed to acquire lock: {e}")))
        {
            Ok(mut guard) => guard.take().is_some(),
            Err(e) => {
                let duration_secs = shutdown_start.elapsed().as_secs_f64();
                self.emit_shutdown_event("failed", duration_secs);
                return Err(e);
            }
        };

        let duration_secs = shutdown_start.elapsed().as_secs_f64();
        if was_first_shutdown {
            self.emit_shutdown_event("success", duration_secs);
        }
        // Idempotent re-invocation: emit no event per spec.
        Ok(())
    }

    fn set_resource(&mut self, resource: &opentelemetry_sdk::Resource) {
        self.resource = resource.into();
    }
}

impl TonicLogsClient {
    // POC: emit otel.sdk.component.shutdown for this exporter. See
    // BatchLogProcessor::emit_shutdown_event for the rationale on using
    // opentelemetry::_private (tracing) directly instead of the
    // otel_info!/otel_warn! macros.
    fn emit_shutdown_event(&self, result_str: &'static str, duration_secs: f64) {
        // Exporter has no internal queue; shutdown.dropped is always 0.
        let shutdown_dropped: u64 = 0;
        // No lifetime drop counter on this exporter (no queue).
        let lifetime_dropped: u64 = 0;

        if result_str == "success" {
            #[cfg(feature = "internal-logs")]
            opentelemetry::_private::info!(
                name: "otel.sdk.component.shutdown",
                target: env!("CARGO_PKG_NAME"),
                name = "otel.sdk.component.shutdown",
                "otel.component.type" = "otlp_grpc_log_exporter",
                "otel.component.name" = self.component_name.as_str(),
                "otel.component.shutdown.result" = result_str,
                "otel.component.dropped" = lifetime_dropped,
                "otel.component.shutdown.dropped" = shutdown_dropped,
                "otel.component.shutdown.duration" = duration_secs,
            );
        } else {
            #[cfg(feature = "internal-logs")]
            opentelemetry::_private::warn!(
                name: "otel.sdk.component.shutdown",
                target: env!("CARGO_PKG_NAME"),
                name = "otel.sdk.component.shutdown",
                "otel.component.type" = "otlp_grpc_log_exporter",
                "otel.component.name" = self.component_name.as_str(),
                "otel.component.shutdown.result" = result_str,
                "otel.component.dropped" = lifetime_dropped,
                "otel.component.shutdown.dropped" = shutdown_dropped,
                "otel.component.shutdown.duration" = duration_secs,
            );
        }

        #[cfg(not(feature = "internal-logs"))]
        {
            let _ = (
                result_str,
                duration_secs,
                shutdown_dropped,
                lifetime_dropped,
            );
        }
    }
}
