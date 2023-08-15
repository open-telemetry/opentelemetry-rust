use core::fmt;
use std::{collections::HashMap, sync::Mutex, time::Duration};

use async_trait::async_trait;
use grpcio::CallOption;
use opentelemetry_api::metrics::{MetricsError, Result};
use opentelemetry_proto::grpcio::collector::metrics::v1::{
    ExportMetricsServiceRequest, MetricsServiceClient,
};
use opentelemetry_sdk::metrics::data::ResourceMetrics;

use crate::metric::MetricsClient;

use grpcio::MetadataBuilder;

pub(crate) struct GrpcioMetricsClient {
    client: Mutex<Option<MetricsServiceClient>>,
    timeout: Duration,
    headers: HashMap<String, String>,
}

impl fmt::Debug for GrpcioMetricsClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GrpcioMetricsClient")
    }
}

impl GrpcioMetricsClient {
    /// Create a new metrics client
    pub(crate) fn new(
        client: MetricsServiceClient,
        timeout: Duration,
        headers: HashMap<String, String>,
    ) -> Self {
        GrpcioMetricsClient {
            client: Mutex::new(Some(client)),
            timeout,
            headers,
        }
    }
}

#[async_trait]
impl MetricsClient for GrpcioMetricsClient {
    async fn export(&self, metrics: &mut ResourceMetrics) -> Result<()> {
        let client =
            self.client
                .lock()
                .map_err(Into::into)
                .and_then(|mut inner| match &mut *inner {
                    Some(inner) => Ok(inner.clone()),
                    None => Err(MetricsError::Other("exporter is already shut down".into())),
                })?;

        let request = ExportMetricsServiceRequest::from(&*metrics);

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

    fn shutdown(&self) -> Result<()> {
        let _ = self.client.lock()?.take();

        Ok(())
    }
}
