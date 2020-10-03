//! Trace exporters
use crate::{api, sdk};
use async_trait::async_trait;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::SystemTime;

pub mod stdout;

/// Describes the result of an export.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportResult {
    /// Batch is successfully exported.
    Success,
    /// Batch export failed. Caller must not retry.
    FailedNotRetryable,
    /// Batch export failed transiently. Caller should record error and may retry.
    FailedRetryable,
}

/// `SpanExporter` defines the interface that protocol-specific exporters must
/// implement so that they can be plugged into OpenTelemetry SDK and support
/// sending of telemetry data.
///
/// The goals of the interface are:
///
/// - Minimize burden of implementation for protocol-dependent telemetry
///  exporters. The protocol exporter is expected to be primarily a simple
/// telemetry data encoder and transmitter.
/// - Allow implementing helpers as composable components that use the same
/// chainable Exporter interface. SDK authors are encouraged to implement common
/// functionality such as queuing, batching, tagging, etc. as helpers. This
/// functionality will be applicable regardless of what protocol exporter is used.
#[async_trait]
pub trait SpanExporter: Send + Sync + std::fmt::Debug {
    /// Exports a batch of telemetry data. Protocol exporters that will implement
    /// this function are typically expected to serialize and transmit the data
    /// to the destination.
    ///
    /// This function will never be called concurrently for the same exporter
    /// instance. It can be called again only after the current call returns.
    ///
    /// This function must not block indefinitely, there must be a reasonable
    /// upper limit after which the call must time out with an error result.
    async fn export(&self, batch: &[Arc<SpanData>]) -> ExportResult;

    /// Shuts down the exporter. Called when SDK is shut down. This is an
    /// opportunity for exporter to do any cleanup required.
    ///
    /// `shutdown` should be called only once for each Exporter instance. After
    /// the call to `shutdown`, subsequent calls to `SpanExport` are not allowed
    /// and should return an error.
    ///
    /// Shutdown should not block indefinitely (e.g. if it attempts to flush the
    /// data and the destination is unavailable). SDK authors can
    /// decide if they want to make the shutdown timeout to be configurable.
    fn shutdown(&mut self) {}
}

/// `SpanData` contains all the information collected by a `Span` and can be used
/// by exporters as a standard input.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SpanData {
    /// Exportable `SpanContext`
    pub span_context: api::SpanContext,
    /// Span parent id
    pub parent_span_id: api::SpanId,
    /// Span kind
    pub span_kind: api::SpanKind,
    /// Span name
    pub name: String,
    /// Span start time
    pub start_time: SystemTime,
    /// Span end time
    pub end_time: SystemTime,
    /// Span attributes
    pub attributes: sdk::EvictedHashMap,
    /// Span Message events
    pub message_events: sdk::EvictedQueue<api::Event>,
    /// Span Links
    pub links: sdk::EvictedQueue<api::Link>,
    /// Span status code
    pub status_code: api::StatusCode,
    /// Span status message
    pub status_message: String,
    /// Resource contains attributes representing an entity that produced this span.
    pub resource: Arc<sdk::Resource>,
    /// Instrumentation library that produced this span
    #[cfg_attr(feature = "serialize", serde(skip))]
    pub instrumentation_lib: sdk::InstrumentationLibrary,
}

#[cfg(feature = "serialize")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::trace::span_context::TraceState;

    #[test]
    fn test_serialise() {
        let trace_id = 7;
        let span_id = 99;

        let trace_flags = 0;
        let remote = false;
        let span_context = api::SpanContext::new(
            api::TraceId::from_u128(trace_id),
            api::SpanId::from_u64(span_id),
            trace_flags,
            remote,
            TraceState::default(),
        );

        let parent_span_id = 1;
        let span_kind = api::SpanKind::Client;
        let name = "foo/bar baz 人?!".to_string();
        let start_time = SystemTime::now();
        let end_time = SystemTime::now();

        let capacity = 3;
        let attributes = sdk::EvictedHashMap::new(capacity);
        let message_events = sdk::EvictedQueue::new(capacity);
        let links = sdk::EvictedQueue::new(capacity);

        let status_code = api::StatusCode::OK;
        let status_message = String::new();
        let resource = Arc::new(sdk::Resource::default());

        let span_data = SpanData {
            span_context,
            parent_span_id: api::SpanId::from_u64(parent_span_id),
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
