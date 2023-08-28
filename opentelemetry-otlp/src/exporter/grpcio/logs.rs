use core::fmt;
use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use grpcio::CallOption;
use opentelemetry::logs::{LogError, LogResult};
use opentelemetry_proto::grpcio::collector::logs::v1::{
    ExportLogsServiceRequest, LogsServiceClient,
};
use opentelemetry_sdk::export::logs::{LogData, LogExporter};

use grpcio::MetadataBuilder;

pub(crate) struct GrpcioLogsClient {
    client: Option<LogsServiceClient>,
    timeout: Duration,
    headers: HashMap<String, String>,
}

impl fmt::Debug for GrpcioLogsClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GrpcioLogsClient")
    }
}

impl GrpcioLogsClient {
    /// Create a new logs client
    pub(crate) fn new(
        client: LogsServiceClient,
        timeout: Duration,
        headers: HashMap<String, String>,
    ) -> Self {
        GrpcioLogsClient {
            client: Some(client),
            timeout,
            headers,
        }
    }
}

#[async_trait]
impl LogExporter for GrpcioLogsClient {
    async fn export(&mut self, batch: Vec<LogData>) -> LogResult<()> {
        let client = match &self.client {
            Some(client) => client,
            None => return Err(LogError::Other("exporter is already shut down".into())),
        };

        let request = ExportLogsServiceRequest {
            resource_logs: batch.into_iter().map(Into::into).collect(),
        };

        let mut call_options = CallOption::default().timeout(self.timeout);

        if !self.headers.is_empty() {
            let mut metadata_builder: MetadataBuilder = MetadataBuilder::new();

            for (key, value) in self.headers.iter() {
                let _ = metadata_builder.add_str(key.as_str(), value.as_str());
            }

            call_options = call_options.headers(metadata_builder.build());
        }

        let receiver = client
            .export_async_opt(&request, call_options)
            .map_err(crate::Error::from)?;
        receiver.await.map_err(crate::Error::from)?;

        Ok(())
    }

    fn shutdown(&mut self) {
        let _ = self.client.take();
    }
}
