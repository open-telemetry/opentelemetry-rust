//! # Zipkin Span Exporter
use crate::model::span::Span;
use opentelemetry::exporter::trace::ExportResult;
use reqwest::Url;

#[derive(Debug)]
pub(crate) enum Uploader {
    Http(JsonV2Client),
}

impl Uploader {
    /// Create a new http uploader
    pub(crate) fn with_http_endpoint(collector_endpoint: Url) -> Self {
        Uploader::Http(JsonV2Client {
            client: reqwest::Client::new(),
            collector_endpoint,
        })
    }

    /// Upload spans to Zipkin
    pub(crate) async fn upload(&self, spans: Vec<Span>) -> ExportResult {
        match self {
            Uploader::Http(client) => client.upload(spans).await,
        }
    }
}

#[derive(Debug)]
pub(crate) struct JsonV2Client {
    client: reqwest::Client,
    collector_endpoint: Url,
}

impl JsonV2Client {
    async fn upload(&self, spans: Vec<Span>) -> ExportResult {
        let resp = self
            .client
            .post(self.collector_endpoint.clone())
            .json(&spans)
            .send()
            .await;

        match resp {
            Ok(response) if response.status().is_success() => ExportResult::Success,
            _ => ExportResult::FailedRetryable,
        }
    }
}
