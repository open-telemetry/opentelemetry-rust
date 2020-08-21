//! # Jaeger Propagator
//!
//! See [`Jaeger documentation`] for detail of Jaeger propagation format
//!
//! [`Jaeger documentation`]: https://www.jaegertracing.io/docs/1.18/client-libraries/#propagation-format

use crate::api::{Context, FieldIter, Injector, Extractor, HttpTextFormat, SpanContext,
                 TraceContextExt, TraceId, SpanId, TRACE_FLAG_SAMPLED, TRACE_FLAG_NOT_SAMPLED,
                 TRACE_FLAG_DEBUG};
use std::str::FromStr;

const JAEGER_HEADER: &'static str = "uber-trace-id";
const DEPRECATED_PARENT_SPAN: &'static str = "0";

lazy_static::lazy_static! {
    static ref JAEGER_HEADER_FIELD: [String; 1] = [JAEGER_HEADER.to_string()];
}

/// Jaeger propagator propagate span context in jaeger propagation format.
///
/// See [`Jaeger documentation`] for format details.
///
///  [`Jaeger documentation`]: https://www.jaegertracing.io/docs/1.18/client-libraries/#propagation-format
#[derive(Clone, Debug)]
pub struct JaegerPropagator {
    _private: (),
}

impl Default for JaegerPropagator {
    fn default() -> Self {
        JaegerPropagator {
            _private: (),
        }
    }
}

impl JaegerPropagator {
    /// Create a Jaeger propagator
    pub fn new() -> Self {
        JaegerPropagator::default()
    }

    /// Extract span context from header value
    fn extract_span_context(&self, header_value: &str) -> Result<SpanContext, ()> {
        let parts = header_value.split_terminator(":").collect::<Vec<&str>>();
        if parts.len() != 4 {
            return Err(());
        }

        // extract trace id
        let trace_id = self.extract_trace_id(parts[0])?;
        let span_id = self.extract_span_id(parts[1])?;
        // Ignore parent span id since it's deprecated.
        let flag = self.extract_flag(parts[3])?;

        Ok(SpanContext::new(trace_id, span_id, flag, true))
    }

    /// Extract trace id from the header.
    fn extract_trace_id(&self, trace_id: &str) -> Result<TraceId, ()> {
        if trace_id.len() > 32 {
            return Err(())
        }

        // allow variable length, padding 0 when length is less than 32
        let padded_trace_id = format!("{:0>32}", trace_id);

        u128::from_str_radix(padded_trace_id.as_str(), 16)
            .map(TraceId::from_u128)
            .map_err(|_| ())
    }

    /// Extract span id from the header.
    fn extract_span_id(&self, span_id: &str) -> Result<SpanId, ()> {
        if span_id.len() != 16 {
            return Err(())
        }

        u64::from_str_radix(span_id, 16)
            .map(SpanId::from_u64)
            .map_err(|_| ())
    }

    /// Extract flag from the header
    ///
    /// First bit control whether to sample
    /// Second bit control whether it's a debug trace
    /// Third bit is not used.
    /// Forth bit is firehose flag, which is not supported in OT now.
    fn extract_flag(&self, flag: &str) -> Result<u8, ()> {
        let flag = u8::from_str(flag).map_err(|_| ())?;
        return if flag & 0x01 == 0x01 {
            if flag & 0x02 == 0x02 {
                Ok(TRACE_FLAG_SAMPLED | TRACE_FLAG_DEBUG)
            } else {
                Ok(TRACE_FLAG_SAMPLED)
            }
        } else {
            // Debug flag should only be set when sampled flag is set.
            // So if debug flag is set alone. We will just use not sampled flag
            Ok(TRACE_FLAG_NOT_SAMPLED)
        };
    }
}

impl HttpTextFormat for JaegerPropagator {
    fn inject_context(&self, cx: &Context, injector: &mut dyn Injector) {
        let span_context = cx.span().span_context();
        if span_context.is_valid() {
            let flag: u8 = if span_context.is_sampled() {
                if span_context.is_debug() {
                    0x03
                } else {
                    0x01
                }
            } else {
                0x00
            };
            let header_value = format!(
                "{:032x}:{:016x}:{:01}:{:01}",
                span_context.trace_id().to_u128(),
                span_context.span_id().to_u64(),
                DEPRECATED_PARENT_SPAN,
                flag,
            );
            injector.set(JAEGER_HEADER, header_value);
        }
    }

    fn extract_with_context(&self, cx: &Context, extractor: &dyn Extractor) -> Context {
        let header_value = extractor.get(JAEGER_HEADER).unwrap_or("");
        cx.with_remote_span_context(
            self.extract_span_context(header_value).unwrap_or(SpanContext::empty_context()))
    }

    fn fields(&self) -> FieldIter {
        FieldIter::new(JAEGER_HEADER_FIELD.as_ref())
    }
}

#[cfg!(test)]
mod tests{

}