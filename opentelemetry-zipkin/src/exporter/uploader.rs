//! # Zipkin Span Exporter
use crate::exporter::model::span::Span;
use crate::exporter::Error;
use http::{header::CONTENT_TYPE, Method, Request, Uri};
use opentelemetry::sdk::export::trace::ExportResult;
use opentelemetry_http::{HttpClient, ResponseExt};
use std::fmt::Debug;

#[derive(Debug)]
pub(crate) enum Uploader {
    Http(JsonV2Client),
}

impl Uploader {
    /// Create a new http uploader
    pub(crate) fn new(client: Box<dyn HttpClient>, collector_endpoint: Uri) -> Self {
        Uploader::Http(JsonV2Client {
            client,
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
    client: Box<dyn HttpClient>,
    collector_endpoint: Uri,
}

impl JsonV2Client {
    async fn upload(&self, spans: Vec<Span>) -> ExportResult {
        let req = Request::builder()
            .method(Method::POST)
            .uri(self.collector_endpoint.clone())
            .header(CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&spans).unwrap_or_default())
            .map_err::<Error, _>(Into::into)?;
        let _ = self.client.send(req).await?.error_for_status()?;
        Ok(())
    }
}
