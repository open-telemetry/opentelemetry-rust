//! # Base64 Format
//!
//! `Base64Format` is a formatter to serialize and deserialize a
//! value into a base64 format.
//!
//! `Base64Format` MUST expose the APIs that serializes values into base64 strings,
//! and deserializes values from base64 strings. There is a blanket implementation
//! for any implementors of `BinaryFormat`
#[cfg(feature = "binary_propagator")]
use crate::trace::propagator::binary::binary_propagator::BinaryFormat;

use base64::{decode, encode};
use opentelemetry::trace::SpanContext;

/// Used to serialize and deserialize `SpanContext`s to and from a base64
/// representation.
pub trait Base64Format {
    /// Serializes span context into a base64 encoded string
    fn serialize_into_base64(&self, context: &SpanContext) -> String;

    /// Deserialize a span context from a base64 encoded string
    fn deserialize_from_base64(&self, base64: &str) -> SpanContext;
}

impl<Format> Base64Format for Format
where
    Format: BinaryFormat,
{
    fn serialize_into_base64(&self, context: &SpanContext) -> String {
        encode(&self.serialize_into_bytes(context))
    }

    fn deserialize_from_base64(&self, base64: &str) -> SpanContext {
        if let Ok(bytes) = decode(base64.as_bytes()) {
            self.deserialize_from_bytes(bytes)
        } else {
            SpanContext::empty_context()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::propagator::binary::binary_propagator::BinaryPropagator;
    use opentelemetry::trace::{SpanId, TraceFlags, TraceId, TraceState};

    #[rustfmt::skip]
    fn to_base64_data() -> Vec<(SpanContext, String)> {
        vec![
            (SpanContext::new(
               TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736),
               SpanId::from_u64(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED, true, TraceState::default()),
                "AABL+S81d7NNpqPOkp0ODkc2AQDwZ6oLqQK3AgE=".to_string()
            ),
            (SpanContext::new(
               TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736),
               SpanId::from_u64(0x00f0_67aa_0ba9_02b7), TraceFlags::default(), true, TraceState::default()),
                "AABL+S81d7NNpqPOkp0ODkc2AQDwZ6oLqQK3AgA=".to_string()
            ),
        ]
    }

    #[rustfmt::skip]
    fn from_base64_data() -> Vec<(SpanContext, String)> {
        vec![
            (SpanContext::empty_context(), "invalid base64 string".to_string())
        ]
    }

    #[test]
    fn serialize_into_base64_conversion() {
        let propagator = BinaryPropagator::new();

        for (context, data) in to_base64_data() {
            assert_eq!(propagator.serialize_into_base64(&context), data)
        }
    }

    #[test]
    fn deserialize_from_base64_conversion() {
        let propagator = BinaryPropagator::new();

        for (context, data) in from_base64_data() {
            assert_eq!(propagator.deserialize_from_base64(&data), context)
        }
        for (context, data) in to_base64_data() {
            assert_eq!(propagator.deserialize_from_base64(&data), context)
        }
    }
}
