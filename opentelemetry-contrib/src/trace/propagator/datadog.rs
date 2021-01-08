use opentelemetry::{
    propagation::{text_map_propagator::FieldIter, Extractor, Injector, TextMapPropagator},
    trace::{
        SpanContext, SpanId, TraceContextExt, TraceId, TraceState, TRACE_FLAG_NOT_SAMPLED,
        TRACE_FLAG_SAMPLED,
    },
    Context,
};

const DATADOG_TRACE_ID_HEADER: &'static str = "x-datadog-trace-id";
const DATADOG_PARENT_ID_HEADER: &'static str = "x-datadog-parent-id";
const DATADOG_SAMPLING_PRIORITY_HEADER: &'static str = "x-datadog-sampling-priority";
// TODO: Implement origin propagation once the Context API is stable
// const HTTP_HEADER_ORIGIN: &'static str = "x-datadog-origin";
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
enum ExtractError {
    InvalidTraceId,
    InvalidSpanId,
    InvalidSamplingPriority,
    InvalidSpanContext,
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
            .map_err(|_| ExtractError::InvalidTraceId)
    }

    fn extract_span_id(&self, span_id: &str) -> Result<SpanId, ExtractError> {
        u64::from_str_radix(span_id, 10)
            .map(SpanId::from_u64)
            .map_err(|_| ExtractError::InvalidSpanId)
    }

    fn extract_sampling_priority(
        &self,
        sampling_priority: &str,
    ) -> Result<SamplingPriority, ExtractError> {
        let i = i32::from_str_radix(sampling_priority, 10)
            .map_err(|_| ExtractError::InvalidSamplingPriority)?;

        match i {
            -1 => Ok(SamplingPriority::UserReject),
            0 => Ok(SamplingPriority::AutoReject),
            1 => Ok(SamplingPriority::AutoKeep),
            2 => Ok(SamplingPriority::UserKeep),
            _ => Err(ExtractError::InvalidSamplingPriority),
        }
    }

    fn extract_span_context(&self, extractor: &dyn Extractor) -> Result<SpanContext, ExtractError> {
        let trace_id =
            self.extract_trace_id(extractor.get(DATADOG_TRACE_ID_HEADER).unwrap_or(""))?;
        let span_id =
            self.extract_span_id(extractor.get(DATADOG_PARENT_ID_HEADER).unwrap_or(""))?;
        let sampling_priority = self.extract_sampling_priority(
            extractor
                .get(DATADOG_SAMPLING_PRIORITY_HEADER)
                .unwrap_or(""),
        )?;
        let sampled = match sampling_priority {
            SamplingPriority::UserReject | SamplingPriority::AutoReject => TRACE_FLAG_NOT_SAMPLED,
            SamplingPriority::UserKeep | SamplingPriority::AutoKeep => TRACE_FLAG_SAMPLED,
        };

        let trace_state = TraceState::default();

        let span_context = SpanContext::new(trace_id, span_id, sampled, true, trace_state);

        if span_context.is_valid() {
            Ok(span_context)
        } else {
            Err(ExtractError::InvalidSpanContext)
        }
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
