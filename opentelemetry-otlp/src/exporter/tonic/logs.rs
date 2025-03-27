use core::fmt;
use opentelemetry::otel_debug;
use opentelemetry_proto::tonic::collector::logs::v1::{
    logs_service_client::LogsServiceClient, ExportLogsServiceRequest,
};
use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use opentelemetry_sdk::logs::{LogBatch, LogExporter};
use tokio::sync::Mutex;
use tonic::{codegen::CompressionEncoding, service::Interceptor, transport::Channel, Request};

use opentelemetry_proto::transform::logs::tonic::group_logs_by_resource_and_scope;

use super::BoxInterceptor;

use super::retry::{retry_with_exponential_backoff, RetryPolicy};

pub(crate) struct TonicLogsClient {
    inner: Mutex<Option<ClientInner>>,
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
            resource: Default::default(),
        }
    }
}

impl LogExporter for TonicLogsClient {
    async fn export(&self, batch: LogBatch<'_>) -> OTelSdkResult {
        let (mut client, metadata, extensions) = match self.inner.lock().await.as_mut() {
            Some(inner) => {
                let (m, e, _) = inner
                    .interceptor
                    .call(Request::new(()))
                    .map_err(|e| OTelSdkError::InternalFailure(format!("error: {:?}", e)))?
                    .into_parts();
                (inner.client.clone(), m, e)
            }
            None => return Err(OTelSdkError::AlreadyShutdown),
        };

        let resource_logs = group_logs_by_resource_and_scope(&batch, &self.resource);

        otel_debug!(name: "TonicsLogsClient.CallingExport");

        // First attempt without retry
        let result = client
            .export(Request::from_parts(
                metadata.clone(),
                extensions.clone(),
                ExportLogsServiceRequest { 
                    resource_logs: resource_logs.clone() 
                },
            ))
            .await;

        // If the first attempt succeeds, return success
        if result.is_ok() {
            return Ok(());
        }

        // If the first attempt fails, try with retry
        otel_debug!(name: "TonicsLogsClient.FirstAttemptFailed.Retrying");

        let policy = RetryPolicy {
            max_retries: 10,
            initial_delay_ms: 100,
            max_delay_ms: 1600,
            jitter_ms: 100,
        };
        
        // Now use retry_with_exponential_backoff for subsequent attempts
        retry_with_exponential_backoff(
            policy,
            "TonicsLogsClient.export",
            || async {
                client
                    .clone()
                    .export(Request::from_parts(
                        metadata.clone(),
                        extensions.clone(),
                        ExportLogsServiceRequest { 
                            resource_logs: resource_logs.clone() 
                        },
                    ))
                    .await
                    .map_err(|e| OTelSdkError::InternalFailure(format!("export error: {:?}", e)))
            }
        )
        .await
        .map(|_| ()) // Convert successful response to () as required by OTelSdkResult
    }

    fn shutdown(&self) -> OTelSdkResult {
        // TODO: Implement actual shutdown
        // Due to the use of tokio::sync::Mutex to guard
        // the inner client, we need to await the call to lock the mutex
        // and that requires async runtime.
        // It is possible to fix this by using
        // a dedicated thread just to handle shutdown.
        // But for now, we just return Ok.
        Ok(())
    }

    fn set_resource(&mut self, resource: &opentelemetry_sdk::Resource) {
        self.resource = resource.into();
    }
}
