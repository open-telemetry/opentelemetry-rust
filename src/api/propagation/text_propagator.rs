//! # Text Propagator
//!
//! `HttpTextFormat` is a formatter to serialize and deserialize a
//! value into a text format.
use crate::api;

///is used to inject and extract a value as text into carriers that travel
/// in-band across process boundaries.
pub trait HttpTextFormat {
    /// Properly encodes the values of the `SpanContext` and injects them
    /// into the `Carrier`.
    fn inject(&self, context: api::SpanContext, carrier: &mut dyn api::Carrier);

    /// Retrieves encoded `SpanContext`s using the `Carrier`. It decodes
    /// the `SpanContext` and returns it. If no `SpanContext` was retrieved
    /// OR if the retrieved SpanContext is invalid then an empty `SpanContext`
    /// is returned.
    fn extract(&self, carrier: &dyn api::Carrier) -> api::SpanContext;
}
