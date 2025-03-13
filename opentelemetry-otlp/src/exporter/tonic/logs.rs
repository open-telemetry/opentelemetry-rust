use std::sync::Arc;
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

use opentelemetry_sdk::retry::{retry_with_exponential_backoff, RetryPolicy};

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
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 1600,
            jitter_ms: 100,
        };

        let batch = Arc::new(batch); // Wrap batch in Arc<Mutex<LogBatch>>

        retry_with_exponential_backoff::<_, _, _, _, tokio::time::Sleep>(policy, "TonicLogsClient.Export", {
            let batch = Arc::clone(&batch);
            move || {
                let batch = Arc::clone(&batch); // Clone the Arc inside the closure
                Box::pin(async move {
                    let (mut client, metadata, extensions) = match &self.inner {
                        Some(inner) => {
                            let (m, e, _) = inner
                                .interceptor
                                .lock()
                                .await // tokio::sync::Mutex doesn't return a poisoned error, so we can safely use the interceptor here
                                .call(Request::new(()))
                                .map_err(|e| OTelSdkError::InternalFailure(format!("error: {:?}", e)))?
                                .into_parts();
                            (inner.client.clone(), m, e)
                        }
                        None => return Err(OTelSdkError::AlreadyShutdown),
                    };

                    let resource_logs = group_logs_by_resource_and_scope(&*batch, &self.resource);

                    otel_debug!(name: "TonicsLogsClient.CallingExport");

                    client
                        .export(Request::from_parts(
                            metadata,
                            extensions,
                            ExportLogsServiceRequest { resource_logs },
                        ))
                        .await
                        .map(|_| ()) // Map the successful result to Ok(())
                        .map_err(|e| OTelSdkError::InternalFailure(format!("export error: {:?}", e)))
                })
            }
        }).await
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
