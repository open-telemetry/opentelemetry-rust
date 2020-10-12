//! # Zipkin Span Exporter
use crate::model::span::Span;
use async_trait::async_trait;
use http::{header::CONTENT_TYPE, Request, Uri, Method};
use opentelemetry::exporter::trace::ExportResult;
use std::fmt::Debug;
use std::error::Error;
#[cfg(feature = "reqwest")]
use std::convert::TryInto;

#[derive(Debug)]
pub(crate) enum Uploader {
    Http(JsonV2Client),
}

impl Uploader {
    /// Create a new http uploader
    pub(crate) fn new(
        client: Box<dyn HttpClient>,
        collector_endpoint: Uri,
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
    client: Box<dyn HttpClient>,
    collector_endpoint: Uri,
}

impl JsonV2Client {
    async fn upload(&self, spans: Vec<Span>) -> Result<ExportResult, Box<dyn std::error::Error>> {
        let req = Request::builder()
            .method(Method::POST)
            .uri(self.collector_endpoint.clone())
            .header(CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&spans).unwrap_or_default())?;
        self.client.send(req).await
    }
}

/// A minimal interface necessary for uploading Zipkin spans over HTTP.
#[async_trait]
pub trait HttpClient: Debug + Send + Sync {
    /// Send a batch of spans to Zipkin
    async fn send(&self, request: Request<Vec<u8>>) -> Result<ExportResult, Box<dyn Error>>;
}

#[cfg(feature = "reqwest")]
#[async_trait]
impl HttpClient for reqwest::Client {
    async fn send(&self, request: Request<Vec<u8>>) -> Result<ExportResult, Box<dyn Error>> {
        let result = self.execute(request.try_into()?).await?;

        if result.status().is_success() {
            Ok(ExportResult::Success)
        } else {
            Ok(ExportResult::FailedNotRetryable)
        }
    }
}

#[cfg(feature = "reqwest")]
#[async_trait]
impl HttpClient for reqwest::blocking::Client {
    async fn send(&self, request: Request<Vec<u8>>) -> Result<ExportResult, Box<dyn Error>> {
        let result = self.execute(request.try_into()?)?;

        if result.status().is_success() {
            Ok(ExportResult::Success)
        } else {
            Ok(ExportResult::FailedNotRetryable)
        }
    }
}

#[cfg(feature = "surf")]
#[async_trait]
impl HttpClient for surf::Client {
    async fn send(&self, request: Request<Vec<u8>>) -> Result<ExportResult, Box<dyn Error>> {
        let (parts, body) = request.into_parts();
        let uri = parts.uri.to_string().parse()?;

        let req = surf::Request::builder(surf::http::Method::Post, uri)
            .content_type("application/json")
            .body(body);
        let result = self.send(req).await?;

        if result.status().is_success() {
            Ok(ExportResult::Success)
        } else {
            Ok(ExportResult::FailedNotRetryable)
        }
    }
}
