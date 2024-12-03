use async_trait::async_trait;
use core::fmt;
use opentelemetry_proto::tonic::collector::logs::v1::{
    logs_service_client::LogsServiceClient, ExportLogsServiceRequest,
};
use opentelemetry_sdk::export::logs::{LogBatch, LogExporter};
use opentelemetry_sdk::logs::{LogError, LogResult};
use tonic::{codegen::CompressionEncoding, service::Interceptor, transport::Channel, Request};

use opentelemetry_proto::transform::logs::tonic::group_logs_by_resource_and_scope;

use super::BoxInterceptor;
use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) struct TonicLogsClient {
    inner: ClientInner,
    is_shutdown: AtomicBool,
    #[allow(dead_code)]
    // <allow dead> would be removed once we support set_resource for metrics.
    resource: opentelemetry_proto::transform::common::tonic::ResourceAttributesWithSchema,
}

struct ClientInner {
    client: Mutex<LogsServiceClient<Channel>>,
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

        TonicLogsClient {
            inner: ClientInner {
                client,
                interceptor,
            },
            is_shutdown: AtomicBool::new(false),
            resource: Default::default(),
        }
    }
}

#[async_trait]
impl LogExporter for TonicLogsClient {
    async fn export(&self, batch: LogBatch<'_>) -> LogResult<()> {
        if self.is_shutdown.load(Ordering::SeqCst) {
            return Err(LogError::Other("exporter is already shut down".into()));
        }

        let (metadata, extensions, _) = self
            .inner
            .interceptor
            .call(Request::new(()))
            .map_err(|e| LogError::Other(Box::new(e)))?
            .into_parts();

        let resource_logs = group_logs_by_resource_and_scope(batch, &self.resource);

        self.inner
            .client
            .export(Request::from_parts(
                metadata,
                extensions,
                ExportLogsServiceRequest { resource_logs },
            ))
            .await
            .map_err(crate::Error::from)?;

        Ok(())
    }

    fn shutdown(&mut self) {}

    fn set_resource(&mut self, resource: &opentelemetry_sdk::Resource) {
        self.resource = resource.into();
    }
}
