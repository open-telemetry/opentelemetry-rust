use async_trait::async_trait;
use core::fmt;
use opentelemetry::logs::{LogError, LogResult};
use opentelemetry_proto::tonic::collector::logs::v1::{
    logs_service_client::LogsServiceClient, ExportLogsServiceRequest,
};
use opentelemetry_sdk::export::logs::{LogData, LogExporter};
use tonic::{codegen::CompressionEncoding, service::Interceptor, transport::Channel, Request};

use super::BoxInterceptor;

pub(crate) struct TonicLogsClient {
    inner: Option<ClientInner>,
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

        TonicLogsClient {
            inner: Some(ClientInner {
                client,
                interceptor,
            }),
            resource: Default::default(),
        }
    }
}

#[async_trait]
impl LogExporter for TonicLogsClient {
    async fn export<'a>(&mut self, batch: Vec<std::borrow::Cow<'a, LogData>>) -> LogResult<()> {
        let (mut client, metadata, extensions) = match &mut self.inner {
            Some(inner) => {
                let (m, e, _) = inner
                    .interceptor
                    .call(Request::new(()))
                    .map_err(|e| LogError::Other(Box::new(e)))?
                    .into_parts();
                (inner.client.clone(), m, e)
            }
            None => return Err(LogError::Other("exporter is already shut down".into())),
        };

        // TODO: Avoid cloning here.
        let resource_logs = {
            batch
                .into_iter()
                .map(|log_data_cow| (log_data_cow.into_owned()))
                .map(|log_data| (log_data, &self.resource))
                .map(Into::into)
                .collect()
        };

        client
            .export(Request::from_parts(
                metadata,
                extensions,
                ExportLogsServiceRequest { resource_logs },
            ))
            .await
            .map_err(crate::Error::from)?;

        Ok(())
    }

    fn shutdown(&mut self) {
        let _ = self.inner.take();
    }

    fn set_resource(&mut self, resource: &opentelemetry_sdk::Resource) {
        self.resource = resource.into();
    }
}
