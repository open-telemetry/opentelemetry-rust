//! # OpenTelemetry Id Generator Interface
use crate::api;
use std::fmt;

/// Interface for generating IDs
pub trait IdGenerator: Send + Sync + fmt::Debug {
    /// Generate a new `TraceId`
    fn new_trace_id(&self) -> api::TraceId;

    /// Generate a new `SpanId`
    fn new_span_id(&self) -> api::SpanId;
}
