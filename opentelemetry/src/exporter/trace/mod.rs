//! Trace exporters
use crate::{
    sdk,
    trace::{Event, Link, SpanContext, SpanId, SpanKind, StatusCode},
};
use async_trait::async_trait;
#[cfg(feature = "http")]
use http::Request;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
#[cfg(all(feature = "http", feature = "reqwest"))]
use std::convert::TryInto;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::sync::Arc;
use std::time::SystemTime;

pub mod stdout;

/// Describes the result of an export.
pub type ExportResult = Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;

/// Timed out when exporting spans to remote
#[derive(Debug, Default)]
pub struct ExportTimedOutError {}

impl Display for ExportTimedOutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("export timed out")
    }
}

impl Error for ExportTimedOutError {}

/// `SpanExporter` defines the interface that protocol-specific exporters must
/// implement so that they can be plugged into OpenTelemetry SDK and support
/// sending of telemetry data.
///
/// The goal of the interface is to minimize burden of implementation for
/// protocol-dependent telemetry exporters. The protocol exporter is expected to
/// be primarily a simple telemetry data encoder and transmitter.
#[async_trait]
pub trait SpanExporter: Send + Debug {
    /// Exports a batch of readable spans. Protocol exporters that will
    /// implement this function are typically expected to serialize and transmit
    /// the data to the destination.
    ///
    /// This function will never be called concurrently for the same exporter
    /// instance. It can be called again only after the current call returns.
    ///
    /// This function must not block indefinitely, there must be a reasonable
    /// upper limit after which the call must time out with an error result.
    ///
    /// Any retry logic that is required by the exporter is the responsibility
    /// of the exporter.
    async fn export(&mut self, batch: Vec<SpanData>) -> ExportResult;

    /// Shuts down the exporter. Called when SDK is shut down. This is an
    /// opportunity for exporter to do any cleanup required.
    ///
    /// This function should be called only once for each `SpanExporter`
    /// instance. After the call to `shutdown`, subsequent calls to `export` are
    /// not allowed and should return an error.
    ///
    /// This function should not block indefinitely (e.g. if it attempts to
    /// flush the data and the destination is unavailable). SDK authors
    /// can decide if they want to make the shutdown timeout
    /// configurable.
    fn shutdown(&mut self) {}
}

/// A minimal interface necessary for export spans over HTTP.
///
/// Users sometime choose http clients that relay on certain runtime. This trait
/// allows users to bring their choice of http clients.
#[cfg(feature = "http")]
#[cfg_attr(docsrs, doc(cfg(feature = "http")))]
#[async_trait]
pub trait HttpClient: Debug + Send + Sync {
    /// Send a batch of spans to collectors
    async fn send(&self, request: Request<Vec<u8>>) -> ExportResult;
}

/// `SpanData` contains all the information collected by a `Span` and can be used
/// by exporters as a standard input.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SpanData {
    /// Exportable `SpanContext`
    pub span_context: SpanContext,
    /// Span parent id
    pub parent_span_id: SpanId,
    /// Span kind
    pub span_kind: SpanKind,
    /// Span name
    pub name: String,
    /// Span start time
    pub start_time: SystemTime,
    /// Span end time
    pub end_time: SystemTime,
    /// Span attributes
    pub attributes: sdk::trace::EvictedHashMap,
    /// Span Message events
    pub message_events: sdk::trace::EvictedQueue<Event>,
    /// Span Links
    pub links: sdk::trace::EvictedQueue<Link>,
    /// Span status code
    pub status_code: StatusCode,
    /// Span status message
    pub status_message: String,
    /// Resource contains attributes representing an entity that produced this span.
    pub resource: Arc<sdk::Resource>,
    /// Instrumentation library that produced this span
    #[cfg_attr(feature = "serialize", serde(skip))]
    pub instrumentation_lib: sdk::InstrumentationLibrary,
}

#[cfg(all(feature = "reqwest", feature = "http"))]
#[async_trait]
impl HttpClient for reqwest::Client {
    async fn send(&self, request: Request<Vec<u8>>) -> ExportResult {
        let _result = self
            .execute(request.try_into()?)
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[cfg(all(feature = "reqwest", feature = "http"))]
#[async_trait]
impl HttpClient for reqwest::blocking::Client {
    async fn send(&self, request: Request<Vec<u8>>) -> ExportResult {
        let _result = self.execute(request.try_into()?)?.error_for_status()?;
        Ok(())
    }
}

#[cfg(all(feature = "surf", feature = "http"))]
#[async_trait]
impl HttpClient for surf::Client {
    async fn send(&self, request: Request<Vec<u8>>) -> ExportResult {
        let (parts, body) = request.into_parts();
        let uri = parts.uri.to_string().parse()?;

        let req = surf::Request::builder(surf::http::Method::Post, uri)
            .content_type("application/json")
            .body(body);
        let result = self.send(req).await?;

        if result.status().is_success() {
            Ok(())
        } else {
            Err(result.status().canonical_reason().into())
        }
    }
}

#[cfg(feature = "serialize")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::{TraceId, TraceState};

    #[test]
    fn test_serialise() {
        let trace_id = 7;
        let span_id = 99;

        let trace_flags = 0;
        let remote = false;
        let span_context = SpanContext::new(
            TraceId::from_u128(trace_id),
            SpanId::from_u64(span_id),
            trace_flags,
            remote,
            TraceState::default(),
        );

        let parent_span_id = 1;
        let span_kind = SpanKind::Client;
        let name = "foo/bar baz 人?!".to_string();
        let start_time = crate::time::now();
        let end_time = crate::time::now();

        let capacity = 3;
        let attributes = sdk::trace::EvictedHashMap::new(capacity, 0);
        let message_events = sdk::trace::EvictedQueue::new(capacity);
        let links = sdk::trace::EvictedQueue::new(capacity);

        let status_code = StatusCode::Ok;
        let status_message = String::new();
        let resource = Arc::new(sdk::Resource::default());

        let span_data = SpanData {
            span_context,
            parent_span_id: SpanId::from_u64(parent_span_id),
            span_kind,
            name,
            start_time,
            end_time,
            attributes,
            message_events,
            links,
            status_code,
            status_message,
            resource,
            instrumentation_lib: sdk::InstrumentationLibrary::new("", None),
        };

        let encoded: Vec<u8> = bincode::serialize(&span_data).unwrap();

        let decoded: SpanData = bincode::deserialize(&encoded[..]).unwrap();

        assert_eq!(span_data, decoded);
    }
}
