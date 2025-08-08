use core::fmt;
use tokio::sync::Mutex;

use opentelemetry::otel_debug;
use opentelemetry_proto::tonic::collector::metrics::v1::{
    metrics_service_client::MetricsServiceClient, ExportMetricsServiceRequest,
};
use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
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

        otel_debug!(name: "TonicsMetricsClientBuilt");

        TonicMetricsClient {
            inner: Mutex::new(Some(ClientInner {
                client,
                interceptor,
            })),
        }
    }
}

impl MetricsClient for TonicMetricsClient {
    async fn export(&self, metrics: &ResourceMetrics) -> OTelSdkResult {
        let (mut client, metadata, extensions) = match self.inner.lock().await.as_mut() {
            Some(inner) => {
                let (m, e, _) = inner
                    .interceptor
                    .call(Request::new(()))
                    .map_err(|e| {
                        OTelSdkError::InternalFailure(format!(
                            "unexpected status while exporting {e:?}"
                        ))
                    })?
                    .into_parts();
                (inner.client.clone(), m, e)
            }
            None => {
                return Err(OTelSdkError::InternalFailure(
                    "exporter is already shut down".into(),
                ))
            }
        };

        otel_debug!(name: "TonicMetricsClient.ExportStarted");

        let result = client
            .export(Request::from_parts(
                metadata,
                extensions,
                ExportMetricsServiceRequest::from(metrics),
            ))
            .await;

        match result {
            Ok(_) => {
                otel_debug!(name: "TonicMetricsClient.ExportSucceeded");
                Ok(())
            }
            Err(e) => {
                let error = format!("{e:?}");
                otel_debug!(name: "TonicMetricsClient.ExportFailed", error = &error);
                Err(OTelSdkError::InternalFailure(error))
            }
        }
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
}
