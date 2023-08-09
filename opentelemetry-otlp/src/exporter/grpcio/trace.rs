use core::fmt;
use std::{collections::HashMap, time::Duration};

use futures_core::future::BoxFuture;
use grpcio::CallOption;
use opentelemetry_api::trace::TraceError;
use opentelemetry_proto::grpcio::{
    trace_service::ExportTraceServiceRequest, trace_service_grpc::TraceServiceClient,
};
use opentelemetry_sdk::export::trace::{ExportResult, SpanData, SpanExporter};

use grpcio::MetadataBuilder;

pub(crate) struct GrpcioTraceClient {
    client: Option<TraceServiceClient>,
    timeout: Duration,
    headers: HashMap<String, String>,
}

impl fmt::Debug for GrpcioTraceClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GrpcioTracesClient")
    }
}

impl GrpcioTraceClient {
    /// Create a new trace client
    pub(crate) fn new(
        client: TraceServiceClient,
        timeout: Duration,
        headers: HashMap<String, String>,
    ) -> Self {
        GrpcioTraceClient {
            client: Some(client),
            timeout,
            headers,
        }
    }
}

impl SpanExporter for GrpcioTraceClient {
    fn export(&mut self, batch: Vec<SpanData>) -> BoxFuture<'static, ExportResult> {
        let client = match &self.client {
            Some(client) => client.clone(),
            None => {
                return Box::pin(std::future::ready(Err(TraceError::Other(
                    "exporter is already shut down".into(),
                ))))
            }
        };

        let request = ExportTraceServiceRequest {
            resource_spans: protobuf::RepeatedField::from_vec(
                batch.into_iter().map(Into::into).collect(),
            ),
            ..Default::default()
        };

        let mut call_options = CallOption::default().timeout(self.timeout);

        if !self.headers.is_empty() {
            let mut metadata_builder: MetadataBuilder = MetadataBuilder::new();

            for (key, value) in self.headers.iter() {
                let _ = metadata_builder.add_str(key.as_str(), value.as_str());
            }

            call_options = call_options.headers(metadata_builder.build());
        }

        Box::pin(async move {
            let receiver = client
                .export_async_opt(&request, call_options)
                .map_err(crate::Error::from)?;

            receiver.await.map_err(crate::Error::from)?;

            Ok(())
        })
    }

    fn shutdown(&mut self) {
        let _ = self.client.take();
    }
}
