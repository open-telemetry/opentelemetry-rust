//! # Zipkin Span Exporter
use crate::model::span::Span;
use http_client::http_types::{url, Method, Mime};
use http_client::Body;
use opentelemetry::exporter::trace::ExportResult;

#[derive(Debug)]
pub(crate) enum Uploader {
    Http(JsonV2Client),
}

impl Uploader {
    /// Create a new http uploader
    pub(crate) fn new(
        client: Box<dyn http_client::HttpClient + Send + Sync>,
        collector_endpoint: url::Url,
    ) -> Self {
        Uploader::Http(JsonV2Client {
            client,
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
    client: Box<dyn http_client::HttpClient + Send + Sync>,
    collector_endpoint: url::Url,
}

impl JsonV2Client {
    async fn upload(&self, spans: Vec<Span>) -> Result<ExportResult, Box<dyn std::error::Error>> {
        let mut req = http_client::Request::new(Method::Post, self.collector_endpoint.clone());
        req.set_body(Body::from_json(&spans)?);
        req.set_content_type(Mime::from("application/json"));

        let resp = self.client.send(req).await;

        Ok(match resp {
            Ok(response) if response.status().is_success() => ExportResult::Success,
            _ => ExportResult::FailedRetryable,
        })
    }
}
