//! Trace exporters
use crate::{api, sdk};
use std::sync::Arc;
use std::time::SystemTime;

/// Describes the result of an export.
#[derive(Debug)]
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
    fn export(&self, batch: Vec<Arc<SpanData>>) -> ExportResult;

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
    fn shutdown(&self);

    /// Allows exporter to be downcast
    fn as_any(&self) -> &dyn std::any::Any;
}

/// `SpanData` contains all the information collected by a `Span` and can be used
/// by exporters as a standard input.
#[derive(Clone, Debug)]
pub struct SpanData {
    /// Exportable `SpanContext`
    pub context: api::SpanContext,
    /// Span parent id
    pub parent_span_id: u64,
    /// Span kind
    pub span_kind: api::SpanKind,
    /// Span name
    pub name: String,
    /// Span start time
    pub start_time: SystemTime,
    /// Span end time
    pub end_time: SystemTime,
    /// Span attributes
    pub attributes: sdk::EvictedQueue<api::KeyValue>,
    /// Span Message events
    pub message_events: sdk::EvictedQueue<api::Event>,
    /// Span Links
    pub links: sdk::EvictedQueue<api::Link>,
    /// Span status
    pub status: api::SpanStatus,
}
