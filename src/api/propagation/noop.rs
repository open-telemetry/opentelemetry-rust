//! # No-op OpenTelemetry Propagation Implementation
//!
//! This implementation is useful for testing purposes as it is intended
//! to have minimal resource utilization and runtime impact.
use crate::api;

/// A no-op instance of a `HttpTextFormat`.
#[derive(Debug)]
pub struct NoopTextFormat {}

impl api::HttpTextFormat for NoopTextFormat {
    /// Ignores calls to `inject`
    fn inject(&self, _context: api::SpanContext, _carrier: &mut dyn api::Carrier) {
        // Ignored
    }

    /// Always returns invalid span contexts
    fn extract(&self, _carrier: &dyn api::Carrier) -> api::SpanContext {
        api::SpanContext::new(api::TraceId::invalid(), api::SpanId::invalid(), 0, false)
    }
}

/// A no-op instance of `BinaryFormat`
#[derive(Debug)]
pub struct NoopBinaryFormat {}

impl api::BinaryFormat for NoopBinaryFormat {
    fn to_bytes(&self, _context: &api::SpanContext) -> [u8; 29] {
        [0; 29]
    }

    /// Always returns invalid span contexts
    fn from_bytes(&self, _bytes: Vec<u8>) -> api::SpanContext {
        api::SpanContext::new(api::TraceId::invalid(), api::SpanId::invalid(), 0, false)
    }
}
