use core::fmt;

use futures_core::future::BoxFuture;
use opentelemetry_api::trace::TraceError;
use opentelemetry_proto::tonic::collector::trace::v1::{
    trace_service_client::TraceServiceClient, ExportTraceServiceRequest,
};
use opentelemetry_sdk::export::trace::{ExportResult, SpanData, SpanExporter};
use tonic::{codegen::CompressionEncoding, service::Interceptor, transport::Channel, Request};

use super::BoxInterceptor;

pub(crate) struct TonicTracesClient {
    inner: Option<ClientInner>,
}

struct ClientInner {
    client: TraceServiceClient<Channel>,
    interceptor: BoxInterceptor,
}

impl fmt::Debug for TonicTracesClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TonicTracesClient")
    }
}

impl TonicTracesClient {
    pub(super) fn new(
        channel: Channel,
        interceptor: BoxInterceptor,
        compression: Option<CompressionEncoding>,
    ) -> Self {
        let mut client = TraceServiceClient::new(channel);
        if let Some(compression) = compression {
            client = client.send_compressed(compression);
        }

        TonicTracesClient {
            inner: Some(ClientInner {
                client,
                interceptor,
            }),
        }
    }
}

impl SpanExporter for TonicTracesClient {
    fn export(&mut self, batch: Vec<SpanData>) -> BoxFuture<'static, ExportResult> {
        let (mut client, metadata, extensions) = match &mut self.inner {
            Some(inner) => {
                let (m, e, _) = match inner.interceptor.call(Request::new(())) {
                    Ok(res) => res.into_parts(),
                    Err(e) => {
                        return Box::pin(std::future::ready(Err(TraceError::Other(Box::new(e)))))
                    }
                };
                (inner.client.clone(), m, e)
            }
            None => {
                return Box::pin(std::future::ready(Err(TraceError::Other(
                    "exporter is already shut down".into(),
                ))))
            }
        };

        Box::pin(async move {
            client
                .export(Request::from_parts(
                    metadata,
                    extensions,
                    ExportTraceServiceRequest {
                        resource_spans: batch.into_iter().map(Into::into).collect(),
                    },
                ))
                .await
                .map_err(crate::Error::from)?;

            Ok(())
        })
    }

    fn shutdown(&mut self) {
        let _ = self.inner.take();
    }
}
