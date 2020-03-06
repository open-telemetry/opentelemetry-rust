//! # Binary Propagator
//!
//! `BinaryFormat` is a formatter to serialize and deserialize a
//! value into a binary format.
//!
//! `BinaryFormat` MUST expose the APIs that serializes values into bytes,
//! and deserializes values from bytes.
use crate::api;
use std::convert::TryInto;

/// Used to serialize and deserialize `SpanContext`s to and from  a binary
/// representation.
pub trait BinaryFormat {
    /// Serializes span context into a byte array and returns the array.
    fn to_bytes(&self, context: &api::SpanContext) -> [u8; 29];

    /// Deserializes a span context from a byte array.
    fn from_bytes(&self, bytes: Vec<u8>) -> api::SpanContext;
}

/// Extracts and injects `SpanContext`s from byte arrays.
#[derive(Debug, Default)]
pub struct BinaryPropagator {}

impl BinaryPropagator {
    /// Create a new binary propagator.
    pub fn new() -> Self {
        BinaryPropagator {}
    }
}

impl BinaryFormat for BinaryPropagator {
    /// Serializes span context into a byte array and returns the array.
    fn to_bytes(&self, context: &api::SpanContext) -> [u8; 29] {
        let mut res = [0u8; 29];
        if !context.is_valid() {
            return res;
        }
        res[2..18].copy_from_slice(&context.trace_id().to_u128().to_be_bytes());
        res[18] = 1;
        res[19..27].copy_from_slice(&context.span_id().to_u64().to_be_bytes());
        res[27] = 2;
        res[28] = context.trace_flags();

        res
    }

    /// Deserializes a span context from a byte array.
    fn from_bytes(&self, bytes: Vec<u8>) -> api::SpanContext {
        if bytes.is_empty() {
            return api::SpanContext::empty_context();
        }
        let trace_id: u128;
        let mut span_id = 0;
        let mut trace_flags = 0;
        let mut b = &bytes[1..];
        if b.len() >= 17 && b[0] == 0 {
            trace_id = u128::from_be_bytes(b[1..17].try_into().unwrap());
            b = &b[17..];
        } else {
            return api::SpanContext::empty_context();
        }
        if b.len() >= 9 && b[0] == 1 {
            span_id = u64::from_be_bytes(b[1..9].try_into().unwrap());
            b = &b[9..];
        }
        if b.len() >= 2 && b[0] == 2 {
            trace_flags = b[1]
        }

        let span_context = api::SpanContext::new(
            api::TraceId::from_u128(trace_id),
            api::SpanId::from_u64(span_id),
            trace_flags,
            true,
        );

        if span_context.is_valid() {
            span_context
        } else {
            api::SpanContext::empty_context()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::api::BinaryFormat;

    #[rustfmt::skip]
    fn to_bytes_data() -> Vec<(api::SpanContext, [u8; 29])> {
        vec![
            // Context with sampled
            (api::SpanContext::new(
                api::TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736),
                api::SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 1, true), [
                0x00, 0x00, 0x4b, 0xf9, 0x2f, 0x35, 0x77, 0xb3, 0x4d, 0xa6, 0xa3, 0xce, 0x92, 0x9d, 0x0e, 0x0e, 0x47, 0x36,
                0x01, 0x00, 0xf0, 0x67, 0xaa, 0x0b, 0xa9, 0x02, 0xb7,
                0x02, 0x01,
            ]),
            // Context without sampled
            (api::SpanContext::new(
                api::TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736),
                api::SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 0, true), [
                0x00, 0x00, 0x4b, 0xf9, 0x2f, 0x35, 0x77, 0xb3, 0x4d, 0xa6, 0xa3, 0xce, 0x92, 0x9d, 0x0e, 0x0e, 0x47, 0x36,
                0x01, 0x00, 0xf0, 0x67, 0xaa, 0x0b, 0xa9, 0x02, 0xb7,
                0x02, 0x00,
            ]),
            // Invalid context
            (api::SpanContext::empty_context(), [0u8; 29]),
        ]
    }

    #[rustfmt::skip]
    fn from_bytes_data() -> Vec<(api::SpanContext, Vec<u8>)> {
        vec![
            // Future version of the proto
            (api::SpanContext::new(api::TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), api::SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 1, true), vec![
                0x02, 0x00, 0x4b, 0xf9, 0x2f, 0x35, 0x77, 0xb3, 0x4d, 0xa6, 0xa3, 0xce, 0x92, 0x9d, 0x0e, 0x0e, 0x47, 0x36,
                0x01, 0x00, 0xf0, 0x67, 0xaa, 0x0b, 0xa9, 0x02, 0xb7,
                0x02, 0x01,
            ]),
            // current version with sampled
            (api::SpanContext::new(api::TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), api::SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 1, true), vec![
                0x02, 0x00, 0x4b, 0xf9, 0x2f, 0x35, 0x77, 0xb3, 0x4d, 0xa6, 0xa3, 0xce, 0x92, 0x9d, 0x0e, 0x0e, 0x47, 0x36,
                0x01, 0x00, 0xf0, 0x67, 0xaa, 0x0b, 0xa9, 0x02, 0xb7,
                0x02, 0x01,
            ]),
            // valid context without option
            (api::SpanContext::new(api::TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), api::SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 0, true), vec![
                0x00, 0x00, 0x4b, 0xf9, 0x2f, 0x35, 0x77, 0xb3, 0x4d, 0xa6, 0xa3, 0xce, 0x92, 0x9d, 0x0e, 0x0e, 0x47, 0x36,
                0x01, 0x00, 0xf0, 0x67, 0xaa, 0x0b, 0xa9, 0x02, 0xb7,
            ]),
            // zero trace id
            (api::SpanContext::empty_context(), vec![
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x02, 0x01,
            ]),
            // zero span id
            (api::SpanContext::empty_context(), vec![
                0x00, 0x00, 0x4b, 0xf9, 0x2f, 0x35, 0x77, 0xb3, 0x4d, 0xa6, 0xa3, 0xce, 0x92, 0x9d, 0x0e, 0x0e, 0x47, 0x36,
                0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x02, 0x01,
            ]),
            // wrong trace id field number
            (api::SpanContext::empty_context(), vec![
                0x00, 0x01, 0x4b, 0xf9, 0x2f, 0x35, 0x77, 0xb3, 0x4d, 0xa6, 0xa3, 0xce, 0x92, 0x9d, 0x0e, 0x0e, 0x47, 0x36,
                0x01, 0x00, 0xf0, 0x67, 0xaa, 0x0b, 0xa9, 0x02, 0xb7,
            ]),
            // short byte array
            (api::SpanContext::empty_context(), vec![
                0x00, 0x00, 0x4b, 0xf9, 0x2f, 0x35, 0x77, 0xb3, 0x4d,
            ]),
        ]
    }

    #[test]
    fn to_bytes_conversion() {
        let propagator = BinaryPropagator::new();

        for (context, data) in to_bytes_data() {
            assert_eq!(propagator.to_bytes(&context), data)
        }
    }

    #[test]
    fn from_bytes_conversion() {
        let propagator = BinaryPropagator::new();

        for (context, data) in from_bytes_data() {
            assert_eq!(propagator.from_bytes(data), context)
        }
    }
}
