//! # Zipkin Span Exporter
use crate::model::span::Span;
use opentelemetry::exporter::trace::ExportResult;
use http_client::http_types::{Method, url, Mime};
use http_client::Body;

#[derive(Debug)]
pub(crate) enum Uploader<'a> {
    Http(JsonV2Client<'a>),
}

impl<'a> Uploader<'a> {
    /// Create a new http uploader
    pub(crate) fn new(client: &'a dyn http_client::HttpClient, collector_endpoint: url::Url) -> Self {
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
pub(crate) struct JsonV2Client<'a> {
    client: &'a dyn http_client::HttpClient,
    collector_endpoint: url::Url,
}

impl<'a> JsonV2Client<'a> {
    async fn upload(&self, spans: Vec<Span>) -> Result<ExportResult, Box<dyn std::error::Error>> {
        let mut req = http_client::Request::
        new(Method::Post, self.collector_endpoint.clone());
        req.set_body(Body::from_json(&spans)?);
        req.set_content_type(Mime::from("application/json"));

        let resp = self
            .client
            .send(req)
            .await;

        Ok(match resp {
            Ok(response) if response.status().is_success() => ExportResult::Success,
            _ => ExportResult::FailedRetryable,
        })
    }
}
