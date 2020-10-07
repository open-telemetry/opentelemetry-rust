//! # Zipkin Span Exporter
use crate::model::span::Span;
use isahc::http::Uri;
use opentelemetry::exporter::trace::ExportResult;

#[derive(Debug)]
pub(crate) enum Uploader {
    Http(JsonV2Client),
}

impl Uploader {
    /// Create a new http uploader
    pub(crate) fn with_http_endpoint(collector_endpoint: Uri) -> Self {
        Uploader::Http(JsonV2Client {
            client: isahc::HttpClient::new().expect("isahc default client should always build without error"),
            collector_endpoint,
        })
    }

    /// Upload spans to Zipkin
    pub(crate) async fn upload(&self, spans: Vec<Span>) -> ExportResult {
        match self {
            Uploader::Http(client) => client
                .upload(spans)
                .await
                .unwrap_or(ExportResult::FailedNotRetryable),
        }
    }
}

#[derive(Debug)]
pub(crate) struct JsonV2Client {
    client: isahc::HttpClient,
    collector_endpoint: isahc::http::Uri,
}

impl JsonV2Client {
    async fn upload(&self, spans: Vec<Span>) -> Result<ExportResult, Box<dyn std::error::Error>> {
        let resp = self
            .client
            .send_async(
                isahc::http::Request::post(self.collector_endpoint.clone())
                    .header("content-type", "application/json")
                    .body(serde_json::to_vec(&spans)?)?,
            )
            .await;

        Ok(match resp {
            Ok(response) if response.status().is_success() => ExportResult::Success,
            _ => ExportResult::FailedRetryable,
        })
    }
}
