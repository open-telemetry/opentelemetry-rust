use std::sync::Arc;

use async_trait::async_trait;
use http::{header::CONTENT_TYPE, Method};
use opentelemetry::metrics::{MetricsError, Result};
use opentelemetry_sdk::metrics::data::ResourceMetrics;

use crate::{metric::MetricsClient, Error};

use super::OtlpHttpClient;

#[async_trait]
impl MetricsClient for OtlpHttpClient {
    async fn export(&self, metrics: &mut ResourceMetrics) -> Result<()> {
        let client = self
            .client
            .lock()
            .map_err(Into::into)
            .and_then(|g| match &*g {
                Some(client) => Ok(Arc::clone(client)),
                _ => Err(MetricsError::Other("exporter is already shut down".into())),
            })?;

        let (body, content_type) = build_body(metrics)?;
        let mut request = http::Request::builder()
            .method(Method::POST)
            .uri(&self.collector_endpoint)
            .header(CONTENT_TYPE, content_type)
            .body(body)
            .map_err(|e| crate::Error::RequestFailed(Box::new(e)))?;

        for (k, v) in &self.headers {
            request.headers_mut().insert(k.clone(), v.clone());
        }

        client
            .send(request)
            .await
            .map_err(|e| MetricsError::ExportErr(Box::new(Error::RequestFailed(e))))?;

        Ok(())
    }

    fn shutdown(&self) -> Result<()> {
        let _ = self.client.lock()?.take();

        Ok(())
    }
}

#[cfg(any(feature = "http-proto", feature = "http-json"))]
fn build_body(metrics: &mut ResourceMetrics) -> Result<(Vec<u8>, &'static str)> {
    use crate::exporter::default_protocol;
    #[cfg(feature = "http-json")]
    use crate::Protocol;
    use prost::Message;

    let req: opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest =
        (&*metrics).into();

    match default_protocol() {
        #[cfg(feature = "http-json")]
        Protocol::HttpJson => match serde_json::to_string_pretty(&req) {
            Ok(json) => Ok((json.into(), "application/json")),
            Err(e) => Err(MetricsError::Other(e.to_string())),
        },
        _ => Ok((req.encode_to_vec(), "application/x-protobuf")),
    }
}

#[cfg(not(any(feature = "http-proto", feature = "http-json")))]
fn build_body(_metrics: &mut ResourceMetrics) -> Result<(Vec<u8>, &'static str)> {
    Err(MetricsError::Other(
        "No http protocol configured. Enable `http-proto` or `http-json`".into(),
    ))
}
