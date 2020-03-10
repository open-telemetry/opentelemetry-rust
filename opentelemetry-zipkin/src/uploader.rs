//! # Zipkin Span Exporter
use crate::model::span::ListOfSpans;
use opentelemetry::exporter::trace;

/// Default v2 HTTP Zipkin API route for recording spans
static API_V2_COLLECTOR_ROUTE: &str = "/api/v2/spans";

#[derive(Clone, Debug)]
pub enum UploaderFormat {
    HTTP,
}

#[derive(Debug)]
pub(crate) struct Uploader {
    client: reqwest::blocking::Client,
    collector_endpoint: String,
    format: UploaderFormat,
}

impl Uploader {
    pub(crate) fn new(collector_endpoint: String, format: UploaderFormat) -> Self {
        Uploader {
            format,
            client: reqwest::blocking::Client::new(),
            collector_endpoint: format!("http://{}{}", collector_endpoint, API_V2_COLLECTOR_ROUTE),
        }
    }

    /// Upload a `ListOfSpans` to the designated Zipkin collector
    pub(crate) fn upload(&self, spans: ListOfSpans) -> trace::ExportResult {
        match self.format {
            UploaderFormat::HTTP => self.upload_http(spans),
        }
    }

    fn upload_http(&self, spans: ListOfSpans) -> trace::ExportResult {
        let zipkin_span_json = match serde_json::to_string(&spans) {
            Ok(json) => json,
            Err(_) => return trace::ExportResult::FailedNotRetryable,
        };

        let resp = self
            .client
            .post(&self.collector_endpoint)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(zipkin_span_json)
            .send();

        if let Ok(response) = resp {
            if response.status().is_success() {
                return trace::ExportResult::Success;
            }
        }

        trace::ExportResult::FailedRetryable
    }
}
