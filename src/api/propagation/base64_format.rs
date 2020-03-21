//! # Base64 Format
//!
//! `Base64Format` is a formatter to serialize and deserialize a
//! value into a base64 format.
//!
//! `Base64Format` MUST expose the APIs that serializes values into base64 strings,
//! and deserializes values from base64 strings. There is a blanket implementation
//! for any implementors of `BinaryFormat`
use base64::{decode, encode};
use crate::api;
use crate::api::propagation::binary_propagator::BinaryFormat;

/// Used to serialize and deserialize `SpanContext`s to and from a base64
/// representation.
pub trait Base64Format {
    /// Serializes span context into a base64 encoded string
    fn to_base64(&self, context: &api::SpanContext) -> String;

    /// Deserialize a span context from a base64 encoded string
    fn from_base64(&self, base64: &str) -> api::SpanContext;
}

impl<Format> Base64Format for Format where Format: BinaryFormat {
    fn to_base64(&self, context: &api::SpanContext) -> String {
        encode(&self.to_bytes(context))
    }

    fn from_base64(&self, base64: &str) -> api::SpanContext {
        if let Ok(bytes) = decode(base64.as_bytes()) {
            self.from_bytes(bytes)
        } else {
            api::SpanContext::empty_context()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::api::propagation::binary_propagator::BinaryPropagator;
    
    #[rustfmt::skip]
    fn to_base64_data() -> Vec<(api::SpanContext, String)> {
        vec![
            (api::SpanContext::new(
                api::TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736),
                api::SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 1, true),
                "AABL+S81d7NNpqPOkp0ODkc2AQDwZ6oLqQK3AgE=".to_string()
            ),
            (api::SpanContext::new(
                api::TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736),
                api::SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 0, true),
                "AABL+S81d7NNpqPOkp0ODkc2AQDwZ6oLqQK3AgA=".to_string()
            ),
        ]
    }

    #[rustfmt::skip]
    fn from_base64_data() -> Vec<(api::SpanContext, String)> {
        vec![
            (api::SpanContext::empty_context(), "invalid base64 string".to_string())
        ]
    }

    #[test]
    fn to_base64_conversion() {
        let propagator = BinaryPropagator::new();

        for (context, data) in to_base64_data() {
            assert_eq!(propagator.to_base64(&context), data)
        }
    }

    #[test]
    fn from_base64_conversion() {
        let propagator = BinaryPropagator::new();

        for (context, data) in from_base64_data() {
            assert_eq!(propagator.from_base64(&data), context)
        }
    }
}
