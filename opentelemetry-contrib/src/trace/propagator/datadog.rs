use opentelemetry::{
    propagation::{text_map_propagator::FieldIter, Extractor, Injector, TextMapPropagator},
    trace::{
        SpanContext, SpanId, TraceContextExt, TraceId, TraceState, TRACE_FLAG_DEFERRED,
        TRACE_FLAG_NOT_SAMPLED, TRACE_FLAG_SAMPLED,
    },
    Context,
};

const DATADOG_TRACE_ID_HEADER: &str = "x-datadog-trace-id";
const DATADOG_PARENT_ID_HEADER: &str = "x-datadog-parent-id";
const DATADOG_SAMPLING_PRIORITY_HEADER: &str = "x-datadog-sampling-priority";

lazy_static::lazy_static! {
    static ref DATADOG_HEADER_FIELDS: [String; 3] = [
        DATADOG_TRACE_ID_HEADER.to_string(),
        DATADOG_PARENT_ID_HEADER.to_string(),
        DATADOG_SAMPLING_PRIORITY_HEADER.to_string(),
    ];
}

enum SamplingPriority {
    UserReject = -1,
    AutoReject = 0,
    AutoKeep = 1,
    UserKeep = 2,
}

#[derive(Debug)]
enum ExtractError {
    TraceId,
    SpanId,
    SamplingPriority,
}

/// Extracts and injects `SpanContext`s into `Extractor`s or `Injector`s using Datadog's header format.
///
/// The Datadog header format does not have an explicit spec, but can be divined from the client libraries,
/// such as [dd-trace-go]
///
/// ## Example
///
/// ```
/// use opentelemetry::global;
/// use opentelemetry_contrib::trace::propagator::DatadogPropagator;
///
/// global::set_text_map_propagator(DatadogPropagator::default());
/// ```
///
/// [dd-trace-go]: https://github.com/DataDog/dd-trace-go/blob/v1.28.0/ddtrace/tracer/textmap.go#L293
#[derive(Clone, Debug, Default)]
pub struct DatadogPropagator {
    _private: (),
}

impl DatadogPropagator {
    /// Creates a new `DatadogPropagator`.
    pub fn new() -> Self {
        DatadogPropagator::default()
    }

    fn extract_trace_id(&self, trace_id: &str) -> Result<TraceId, ExtractError> {
        u64::from_str_radix(trace_id, 10)
            .map(|id| TraceId::from_u128(id as u128))
            .map_err(|_| ExtractError::TraceId)
    }

    fn extract_span_id(&self, span_id: &str) -> Result<SpanId, ExtractError> {
        u64::from_str_radix(span_id, 10)
            .map(SpanId::from_u64)
            .map_err(|_| ExtractError::SpanId)
    }

    fn extract_sampling_priority(
        &self,
        sampling_priority: &str,
    ) -> Result<SamplingPriority, ExtractError> {
        let i = i32::from_str_radix(sampling_priority, 10)
            .map_err(|_| ExtractError::SamplingPriority)?;

        match i {
            -1 => Ok(SamplingPriority::UserReject),
            0 => Ok(SamplingPriority::AutoReject),
            1 => Ok(SamplingPriority::AutoKeep),
            2 => Ok(SamplingPriority::UserKeep),
            _ => Err(ExtractError::SamplingPriority),
        }
    }

    fn extract_span_context(&self, extractor: &dyn Extractor) -> Result<SpanContext, ExtractError> {
        let trace_id =
            self.extract_trace_id(extractor.get(DATADOG_TRACE_ID_HEADER).unwrap_or(""))?;
        // If we have a trace_id but can't get the parent span, we default it to invalid instead of completely erroring
        // out so that the rest of the spans aren't completely lost
        let span_id = self
            .extract_span_id(extractor.get(DATADOG_PARENT_ID_HEADER).unwrap_or(""))
            .unwrap_or_else(|_| SpanId::invalid());
        let sampling_priority = self.extract_sampling_priority(
            extractor
                .get(DATADOG_SAMPLING_PRIORITY_HEADER)
                .unwrap_or(""),
        );
        let sampled = match sampling_priority {
            Ok(SamplingPriority::UserReject) | Ok(SamplingPriority::AutoReject) => {
                TRACE_FLAG_NOT_SAMPLED
            }
            Ok(SamplingPriority::UserKeep) | Ok(SamplingPriority::AutoKeep) => TRACE_FLAG_SAMPLED,
            // Treat the sampling as DEFERRED instead of erroring on extracting the span context
            Err(_) => TRACE_FLAG_DEFERRED,
        };

        let trace_state = TraceState::default();

        Ok(SpanContext::new(
            trace_id,
            span_id,
            sampled,
            true,
            trace_state,
        ))
    }
}

impl TextMapPropagator for DatadogPropagator {
    fn inject_context(&self, cx: &Context, injector: &mut dyn Injector) {
        let span_context = cx.span().span_context();
        if span_context.is_valid() {
            injector.set(
                DATADOG_TRACE_ID_HEADER,
                (span_context.trace_id().to_u128() as u64).to_string(),
            );
            injector.set(
                DATADOG_PARENT_ID_HEADER,
                span_context.span_id().to_u64().to_string(),
            );

            if !span_context.is_deferred() {
                let sampling_priority = if span_context.is_sampled() {
                    SamplingPriority::AutoKeep
                } else {
                    SamplingPriority::AutoReject
                };

                injector.set(
                    DATADOG_SAMPLING_PRIORITY_HEADER,
                    (sampling_priority as i32).to_string(),
                );
            }
        }
    }

    fn extract_with_context(&self, cx: &Context, extractor: &dyn Extractor) -> Context {
        let extracted = self
            .extract_span_context(extractor)
            .unwrap_or_else(|_| SpanContext::empty_context());

        cx.with_remote_span_context(extracted)
    }

    fn fields(&self) -> FieldIter<'_> {
        FieldIter::new(DATADOG_HEADER_FIELDS.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::testing::trace::TestSpan;
    use opentelemetry::trace::TraceState;
    use std::collections::HashMap;

    #[rustfmt::skip]
    fn extract_test_data() -> Vec<(Vec<(&'static str, &'static str)>, SpanContext)> {
        vec![
            (vec![], SpanContext::empty_context()),
            (vec![(DATADOG_SAMPLING_PRIORITY_HEADER, "0")], SpanContext::empty_context()),
            (vec![(DATADOG_TRACE_ID_HEADER, "garbage")], SpanContext::empty_context()),
            (vec![(DATADOG_TRACE_ID_HEADER, "1234"), (DATADOG_PARENT_ID_HEADER, "garbage")], SpanContext::new(TraceId::from_u128(1234), SpanId::invalid(), TRACE_FLAG_DEFERRED, true, TraceState::default())),
            (vec![(DATADOG_TRACE_ID_HEADER, "1234"), (DATADOG_PARENT_ID_HEADER, "12")], SpanContext::new(TraceId::from_u128(1234), SpanId::from_u64(12), TRACE_FLAG_DEFERRED, true, TraceState::default())),
            (vec![(DATADOG_TRACE_ID_HEADER, "1234"), (DATADOG_PARENT_ID_HEADER, "12"), (DATADOG_SAMPLING_PRIORITY_HEADER, "0")], SpanContext::new(TraceId::from_u128(1234), SpanId::from_u64(12), TRACE_FLAG_NOT_SAMPLED, true, TraceState::default())),
            (vec![(DATADOG_TRACE_ID_HEADER, "1234"), (DATADOG_PARENT_ID_HEADER, "12"), (DATADOG_SAMPLING_PRIORITY_HEADER, "1")], SpanContext::new(TraceId::from_u128(1234), SpanId::from_u64(12), TRACE_FLAG_SAMPLED, true, TraceState::default())),
        ]
    }

    #[rustfmt::skip]
    fn inject_test_data() -> Vec<(Vec<(&'static str, &'static str)>, SpanContext)> {
        vec![
            (vec![], SpanContext::empty_context()),
            (vec![], SpanContext::new(TraceId::from_hex("garbage"), SpanId::invalid(), TRACE_FLAG_DEFERRED, true, TraceState::default())),
            (vec![], SpanContext::new(TraceId::from_hex("1234"), SpanId::invalid(), TRACE_FLAG_DEFERRED, true, TraceState::default())),
            (vec![], SpanContext::new(TraceId::from_hex("1234"), SpanId::invalid(), TRACE_FLAG_SAMPLED, true, TraceState::default())),
            (vec![(DATADOG_TRACE_ID_HEADER, "1234"), (DATADOG_PARENT_ID_HEADER, "12")], SpanContext::new(TraceId::from_u128(1234), SpanId::from_u64(12), TRACE_FLAG_DEFERRED, true, TraceState::default())),
            (vec![(DATADOG_TRACE_ID_HEADER, "1234"), (DATADOG_PARENT_ID_HEADER, "12"), (DATADOG_SAMPLING_PRIORITY_HEADER, "0")], SpanContext::new(TraceId::from_u128(1234), SpanId::from_u64(12), TRACE_FLAG_NOT_SAMPLED, true, TraceState::default())),
            (vec![(DATADOG_TRACE_ID_HEADER, "1234"), (DATADOG_PARENT_ID_HEADER, "12"), (DATADOG_SAMPLING_PRIORITY_HEADER, "1")], SpanContext::new(TraceId::from_u128(1234), SpanId::from_u64(12), TRACE_FLAG_SAMPLED, true, TraceState::default())),
        ]
    }

    #[test]
    fn test_extract() {
        for (header_list, expected) in extract_test_data() {
            let map: HashMap<String, String> = header_list
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            let propagator = DatadogPropagator::default();
            let context = propagator.extract(&map);
            assert_eq!(context.remote_span_context(), Some(&expected));
        }
    }

    #[test]
    fn test_extract_empty() {
        let map: HashMap<String, String> = HashMap::new();
        let propagator = DatadogPropagator::default();
        let context = propagator.extract(&map);
        assert_eq!(
            context.remote_span_context(),
            Some(&SpanContext::empty_context())
        )
    }

    #[test]
    fn test_inject() {
        let propagator = DatadogPropagator::default();
        for (header_values, span_context) in inject_test_data() {
            let mut injector: HashMap<String, String> = HashMap::new();
            propagator.inject_context(
                &Context::current_with_span(TestSpan(span_context)),
                &mut injector,
            );

            if header_values.is_empty() {
                assert!(injector.is_empty());
            } else {
                for (k, v) in header_values.into_iter() {
                    let injected_value: Option<&String> = injector.get(k);
                    assert_eq!(injected_value, Some(&v.to_string()));
                    injector.remove(k);
                }
                assert!(injector.is_empty());
            }
        }
    }
}
