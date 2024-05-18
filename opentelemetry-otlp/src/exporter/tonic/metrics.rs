use core::fmt;
use std::sync::Mutex;

use async_trait::async_trait;
use opentelemetry::metrics::{MetricsError, Result};
use opentelemetry_proto::tonic::collector::metrics::v1::{
    metrics_service_client::MetricsServiceClient, ExportMetricsServiceRequest,
};
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use tonic::{codegen::CompressionEncoding, service::Interceptor, transport::Channel, Request};

use super::BoxInterceptor;
use crate::metric::MetricsClient;

pub(crate) struct TonicMetricsClient {
    inner: Mutex<Option<ClientInner>>,
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
    ) -> Self {
        let mut client = MetricsServiceClient::new(channel);
        if let Some(compression) = compression {
            client = client
                .send_compressed(compression)
                .accept_compressed(compression);
        }

        TonicMetricsClient {
            inner: Mutex::new(Some(ClientInner {
                client,
                interceptor,
            })),
        }
    }
}

#[async_trait]
impl MetricsClient for TonicMetricsClient {
    async fn export(&self, metrics: &mut ResourceMetrics) -> Result<()> {
        let (mut client, metadata, extensions) =
            self.inner
                .lock()
                .map_err(Into::into)
                .and_then(|mut inner| match &mut *inner {
                    Some(inner) => {
                        let (m, e, _) = inner
                            .interceptor
                            .call(Request::new(()))
                            .map_err(|e| {
                                MetricsError::Other(format!(
                                    "unexpected status while exporting {e:?}"
                                ))
                            })?
                            .into_parts();
                        Ok((inner.client.clone(), m, e))
                    }
                    None => Err(MetricsError::Other("exporter is already shut down".into())),
                })?;

        client
            .export(Request::from_parts(
                metadata,
                extensions,
                ExportMetricsServiceRequest::from(&*metrics),
            ))
            .await
            .map_err(crate::Error::from)?;

        Ok(())
    }

    fn shutdown(&self) -> Result<()> {
        let _ = self.inner.lock()?.take();

        Ok(())
    }
}
