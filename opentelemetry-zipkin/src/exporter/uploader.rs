//! # Zipkin Span Exporter
use crate::exporter::model::span::Span;
use http::{header::CONTENT_TYPE, Method, Request, Uri};
use opentelemetry_http::{HttpClient, ResponseExt};
use opentelemetry_sdk::error::OTelSdkError;
use opentelemetry_sdk::error::OTelSdkResult;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) enum Uploader {
    Http(JsonV2Client),
}

impl Uploader {
    /// Create a new http uploader
    pub(crate) fn new(client: Arc<dyn HttpClient>, collector_endpoint: Uri) -> Self {
        Uploader::Http(JsonV2Client {
            client,
            collector_endpoint,
        })
    }

    /// Upload spans to Zipkin
    pub(crate) async fn upload(&self, spans: Vec<Span>) -> OTelSdkResult {
        match self {
            Uploader::Http(client) => client.upload(spans).await,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct JsonV2Client {
    client: Arc<dyn HttpClient>,
    collector_endpoint: Uri,
}

impl JsonV2Client {
    async fn upload(&self, spans: Vec<Span>) -> OTelSdkResult {
        let body = serde_json::to_vec(&spans).map_err(|e| {
            OTelSdkError::InternalFailure(format!("JSON serialization failed: {}", e))
        })?;
        let req = Request::builder()
            .method(Method::POST)
            .uri(self.collector_endpoint.clone())
            .header(CONTENT_TYPE, "application/json")
            .body(body.into())
            .map_err(|e| {
                OTelSdkError::InternalFailure(format!("Failed to create request: {}", e))
            })?;

        let response =
            self.client.send_bytes(req).await.map_err(|e| {
                OTelSdkError::InternalFailure(format!("HTTP request failed: {}", e))
            })?;

        response
            .error_for_status()
            .map_err(|e| OTelSdkError::InternalFailure(format!("HTTP response error: {}", e)))?;
        Ok(())
    }
}
