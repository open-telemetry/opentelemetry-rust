//! # Tracer
//!
//! The OpenTelemetry library achieves in-process context propagation of
//! `Span`s by way of the `Tracer`.
//!
//! The `Tracer` is responsible for tracking the currently active `Span`,
//! and exposes methods for creating and activating new `Spans`.
//!
//! Docs: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/api-tracing.md#tracer
use crate::sdk;
use crate::trace::{
    Link, SpanBuilder, SpanContext, SpanId, SpanKind, StatusCode, TraceContextExt, TraceId,
    TraceState, TRACE_FLAG_SAMPLED,
};
use crate::{exporter, Context, KeyValue};
use std::fmt;
use std::sync::Weak;
use std::time::SystemTime;

/// `Tracer` implementation to create and manage spans
#[derive(Clone)]
pub struct Tracer {
    instrumentation_lib: sdk::InstrumentationLibrary,
    provider: Weak<sdk::trace::provider::TracerProviderInner>,
}

impl fmt::Debug for Tracer {
    /// Formats the `Tracer` using the given formatter.
    /// Omitting `provider` here is necessary to avoid cycles.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tracer")
            .field("name", &self.instrumentation_lib.name)
            .field("version", &self.instrumentation_lib.version)
            .finish()
    }
}

impl Tracer {
    /// Create a new tracer (used internally by `TracerProvider`s).
    pub(crate) fn new(
        instrumentation_lib: sdk::InstrumentationLibrary,
        provider: Weak<sdk::trace::provider::TracerProviderInner>,
    ) -> Self {
        Tracer {
            instrumentation_lib,
            provider,
        }
    }

    /// TracerProvider associated with this tracer.
    pub fn provider(&self) -> Option<sdk::trace::TracerProvider> {
        self.provider.upgrade().map(sdk::trace::TracerProvider::new)
    }

    /// instrumentation library information of this tracer.
    pub fn instrumentation_library(&self) -> &sdk::InstrumentationLibrary {
        &self.instrumentation_lib
    }

    /// Make a sampling decision using the provided sampler for the span and context.
    #[allow(clippy::too_many_arguments)]
    fn make_sampling_decision(
        &self,
        parent_context: Option<&SpanContext>,
        trace_id: TraceId,
        name: &str,
        span_kind: &SpanKind,
        attributes: &[KeyValue],
        links: &[Link],
    ) -> Option<(u8, Vec<KeyValue>, TraceState)> {
        let provider = self.provider()?;
        let sampler = &provider.config().default_sampler;
        let sampling_result =
            sampler.should_sample(parent_context, trace_id, name, span_kind, attributes, links);

        self.process_sampling_result(sampling_result, parent_context)
    }

    fn process_sampling_result(
        &self,
        sampling_result: sdk::trace::SamplingResult,
        parent_context: Option<&SpanContext>,
    ) -> Option<(u8, Vec<KeyValue>, TraceState)> {
        match sampling_result {
            sdk::trace::SamplingResult {
                decision: sdk::trace::SamplingDecision::Drop,
                ..
            } => None,
            sdk::trace::SamplingResult {
                decision: sdk::trace::SamplingDecision::RecordOnly,
                attributes,
                trace_state,
            } => {
                let trace_flags = parent_context.map(|ctx| ctx.trace_flags()).unwrap_or(0);
                Some((trace_flags & !TRACE_FLAG_SAMPLED, attributes, trace_state))
            }
            sdk::trace::SamplingResult {
                decision: sdk::trace::SamplingDecision::RecordAndSample,
                attributes,
                trace_state,
            } => {
                let trace_flags = parent_context.map(|ctx| ctx.trace_flags()).unwrap_or(0);
                Some((trace_flags | TRACE_FLAG_SAMPLED, attributes, trace_state))
            }
        }
    }
}

impl crate::trace::Tracer for Tracer {
    /// This implementation of `Tracer` produces `sdk::Span` instances.
    type Span = sdk::trace::Span;

    /// Returns a span with an inactive `SpanContext`. Used by functions that
    /// need to return a default span like `get_active_span` if no span is present.
    fn invalid(&self) -> Self::Span {
        sdk::trace::Span::new(SpanId::invalid(), None, self.clone())
    }

    /// Starts a new `Span` in a given context.
    ///
    /// Each span has zero or one parent spans and zero or more child spans, which
    /// represent causally related operations. A tree of related spans comprises a
    /// trace. A span is said to be a _root span_ if it does not have a parent. Each
    /// trace includes a single root span, which is the shared ancestor of all other
    /// spans in the trace.
    fn start_from_context(&self, name: &str, cx: &Context) -> Self::Span {
        let builder = self.span_builder(name);

        self.build_with_context(builder, cx)
    }

    /// Creates a span builder
    ///
    /// An ergonomic way for attributes to be configured before the `Span` is started.
    fn span_builder(&self, name: &str) -> SpanBuilder {
        SpanBuilder::from_name(name.to_string())
    }

    /// Starts a span from a `SpanBuilder`.
    ///
    /// Each span has zero or one parent spans and zero or more child spans, which
    /// represent causally related operations. A tree of related spans comprises a
    /// trace. A span is said to be a _root span_ if it does not have a parent. Each
    /// trace includes a single root span, which is the shared ancestor of all other
    /// spans in the trace.
    fn build_with_context(&self, mut builder: SpanBuilder, cx: &Context) -> Self::Span {
        let provider = self.provider();
        if provider.is_none() {
            return sdk::trace::Span::new(SpanId::invalid(), None, self.clone());
        }

        let provider = provider.unwrap();
        let config = provider.config();
        let span_id = builder
            .span_id
            .take()
            .unwrap_or_else(|| config.id_generator.new_span_id());

        let span_kind = builder.span_kind.take().unwrap_or(SpanKind::Internal);
        let mut attribute_options = builder.attributes.take().unwrap_or_else(Vec::new);
        let mut link_options = builder.links.take().unwrap_or_else(Vec::new);

        let parent_span_context = builder
            .parent_context
            .take()
            .or_else(|| Some(cx.span().span_context()).filter(|cx| cx.is_valid()))
            .or_else(|| cx.remote_span_context().cloned())
            .filter(|cx| cx.is_valid());
        // Build context for sampling decision
        let (no_parent, trace_id, parent_span_id, remote_parent, parent_trace_flags) =
            parent_span_context
                .as_ref()
                .map(|ctx| {
                    (
                        false,
                        ctx.trace_id(),
                        ctx.span_id(),
                        ctx.is_remote(),
                        ctx.trace_flags(),
                    )
                })
                .unwrap_or((
                    true,
                    builder
                        .trace_id
                        .unwrap_or_else(|| config.id_generator.new_trace_id()),
                    SpanId::invalid(),
                    false,
                    0,
                ));

        // There are 3 paths for sampling.
        //
        // * Sampling has occurred elsewhere and is already stored in the builder
        // * There is no parent or a remote parent, in which case make decision now
        // * There is a local parent, in which case defer to the parent's decision
        let sampling_decision = if let Some(sampling_result) = builder.sampling_result.take() {
            self.process_sampling_result(sampling_result, parent_span_context.as_ref())
        } else if no_parent || remote_parent {
            self.make_sampling_decision(
                parent_span_context.as_ref(),
                trace_id,
                &builder.name,
                &span_kind,
                &attribute_options,
                &link_options,
            )
        } else {
            // has parent that is local: use parent if sampled, or don't record.
            parent_span_context
                .filter(|span_context| span_context.is_sampled())
                .map(|span_context| {
                    (
                        parent_trace_flags,
                        Vec::new(),
                        span_context.trace_state().clone(),
                    )
                })
        };

        // Build optional inner context, `None` if not recording.
        let inner = sampling_decision.map(move |(trace_flags, mut extra_attrs, trace_state)| {
            attribute_options.append(&mut extra_attrs);
            let mut attributes = sdk::trace::EvictedHashMap::new(config.max_attributes_per_span);
            for attribute in attribute_options {
                attributes.insert(attribute);
            }
            let mut links = sdk::trace::EvictedQueue::new(config.max_links_per_span);
            links.append_vec(&mut link_options);
            let start_time = builder.start_time.unwrap_or_else(SystemTime::now);
            let end_time = builder.end_time.unwrap_or(start_time);
            let mut message_events = sdk::trace::EvictedQueue::new(config.max_events_per_span);
            if let Some(mut events) = builder.message_events {
                message_events.append_vec(&mut events);
            }
            let status_code = builder.status_code.unwrap_or(StatusCode::Unset);
            let status_message = builder.status_message.unwrap_or_else(String::new);
            let resource = config.resource.clone();

            exporter::trace::SpanData {
                span_context: SpanContext::new(trace_id, span_id, trace_flags, false, trace_state),
                parent_span_id,
                span_kind,
                name: builder.name,
                start_time,
                end_time,
                attributes,
                message_events,
                links,
                status_code,
                status_message,
                resource,
                instrumentation_lib: self.instrumentation_lib,
            }
        });

        // Call `on_start` for all processors
        if let Some(inner) = inner.as_ref().cloned() {
            for processor in provider.span_processors() {
                processor.on_start(&inner)
            }
        }

        sdk::trace::Span::new(span_id, inner, self.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        sdk::{
            self,
            trace::{Config, SamplingDecision, SamplingResult, ShouldSample},
        },
        trace::{
            Link, Span, SpanBuilder, SpanContext, SpanId, SpanKind, TraceId, TraceState, Tracer,
            TracerProvider, TRACE_FLAG_SAMPLED,
        },
        Context, KeyValue,
    };

    #[derive(Debug)]
    struct TestSampler {}

    impl ShouldSample for TestSampler {
        fn should_sample(
            &self,
            parent_context: Option<&SpanContext>,
            _trace_id: TraceId,
            _name: &str,
            _span_kind: &SpanKind,
            _attributes: &[KeyValue],
            _links: &[Link],
        ) -> SamplingResult {
            let trace_state = parent_context.unwrap().trace_state().clone();
            SamplingResult {
                decision: SamplingDecision::RecordAndSample,
                attributes: Vec::new(),
                trace_state: trace_state.insert("foo".into(), "notbar".into()).unwrap(),
            }
        }
    }

    #[test]
    fn allow_sampler_to_change_trace_state() {
        // Setup
        let sampler = TestSampler {};
        let config = Config::default().with_default_sampler(sampler);
        let tracer_provider = sdk::trace::TracerProvider::builder()
            .with_config(config)
            .build();
        let tracer = tracer_provider.get_tracer("test", None);
        let context = Context::default();
        let trace_state = TraceState::from_key_value(vec![("foo", "bar")]).unwrap();
        let mut span_builder = SpanBuilder::default();
        span_builder.parent_context = Some(SpanContext::new(
            TraceId::from_u128(128),
            SpanId::from_u64(64),
            TRACE_FLAG_SAMPLED,
            true,
            trace_state,
        ));

        // Test sampler should change trace state
        let span = tracer.build_with_context(span_builder, &context);
        let span_context = span.span_context();
        let expected = span_context.trace_state();
        assert_eq!(expected.get("foo"), Some("notbar"))
    }
}
