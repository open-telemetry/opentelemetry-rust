use core::fmt;
use opentelemetry::otel_debug;
use opentelemetry_proto::tonic::collector::logs::v1::{
    logs_service_client::LogsServiceClient, ExportLogsServiceRequest,
};
use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use opentelemetry_sdk::logs::{LogBatch, LogExporter};
use tonic::{codegen::CompressionEncoding, service::Interceptor, transport::Channel, Request};

use opentelemetry_proto::transform::logs::tonic::group_logs_by_resource_and_scope;

use super::BoxInterceptor;
use tokio::sync::Mutex;

pub(crate) struct TonicLogsClient {
    inner: Option<ClientInner>,
    #[allow(dead_code)]
    // <allow dead> would be removed once we support set_resource for metrics.
    resource: opentelemetry_proto::transform::common::tonic::ResourceAttributesWithSchema,
}

struct ClientInner {
    client: LogsServiceClient<Channel>,
    interceptor: Mutex<BoxInterceptor>,
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
            inner: Some(ClientInner {
                client,
                interceptor: Mutex::new(interceptor),
            }),
            resource: Default::default(),
        }
    }
}

impl LogExporter for TonicLogsClient {
    #[allow(clippy::manual_async_fn)]
    fn export(
        &self,
        batch: LogBatch<'_>,
    ) -> impl std::future::Future<Output = OTelSdkResult> + Send {
        async move {
            let (mut client, metadata, extensions) = match &self.inner {
                Some(inner) => {
                    let (m, e, _) = inner
                        .interceptor
                        .lock()
                        .await // tokio::sync::Mutex doesn't return a poisoned error, so we can safely use the interceptor here
                        .call(Request::new(()))
                        .map_err(|e| OTelSdkError::InternalFailure(e.to_string()))?
                        .into_parts();
                    (inner.client.clone(), m, e)
                }
                None => return Err(OTelSdkError::AlreadyShutdown),
            };

            let resource_logs = group_logs_by_resource_and_scope(batch, &self.resource);

            otel_debug!(name: "TonicsLogsClient.CallingExport");

            client
                .export(Request::from_parts(
                    metadata,
                    extensions,
                    ExportLogsServiceRequest { resource_logs },
                ))
                .await
                .map_err(|e| OTelSdkError::InternalFailure(crate::Error::from(e).to_string()))?;
            Ok(())
        }
    }

    fn shutdown(&mut self) -> OTelSdkResult {
        let _ = self.inner.take();
        Ok(())
    }

    fn set_resource(&mut self, resource: &opentelemetry_sdk::Resource) {
        self.resource = resource.into();
    }
}
