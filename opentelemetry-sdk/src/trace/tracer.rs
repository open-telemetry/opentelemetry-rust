//! # Tracer
//!
//! The OpenTelemetry library achieves in-process context propagation of
//! `Span`s by way of the `Tracer`.
//!
//! The `Tracer` is responsible for tracking the currently active `Span`,
//! and exposes methods for creating and activating new `Spans`.
//!
//! Docs: <https://github.com/open-telemetry/opentelemetry-specification/blob/v1.3.0/specification/trace/api.md#tracer>
use crate::trace::SpanLimits;
use crate::{
    trace::{
        provider::{TracerProvider, TracerProviderInner},
        span::{Span, SpanData},
        Config, EvictedHashMap, EvictedQueue,
    },
    InstrumentationLibrary,
};
use opentelemetry_api::trace::{
    Link, SamplingDecision, SamplingResult, SpanBuilder, SpanContext, SpanId, SpanKind,
    TraceContextExt, TraceFlags, TraceId, TraceState,
};
use opentelemetry_api::{Context, Key, KeyValue, Value};
use indexmap::IndexMap;
use std::fmt;
use std::sync::Weak;

/// `Tracer` implementation to create and manage spans
#[derive(Clone)]
pub struct Tracer {
    instrumentation_lib: InstrumentationLibrary,
    provider: Weak<TracerProviderInner>,
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
        instrumentation_lib: InstrumentationLibrary,
        provider: Weak<TracerProviderInner>,
    ) -> Self {
        Tracer {
            instrumentation_lib,
            provider,
        }
    }

    /// TracerProvider associated with this tracer.
    pub fn provider(&self) -> Option<TracerProvider> {
        self.provider.upgrade().map(TracerProvider::new)
    }

    /// Instrumentation library information of this tracer.
    pub fn instrumentation_library(&self) -> &InstrumentationLibrary {
        &self.instrumentation_lib
    }

    /// Make a sampling decision using the provided sampler for the span and context.
    #[allow(clippy::too_many_arguments)]
    fn make_sampling_decision(
        &self,
        parent_cx: &Context,
        trace_id: TraceId,
        name: &str,
        span_kind: &SpanKind,
        attributes: &IndexMap<Key, Value>,
        links: &[Link],
        config: &Config,
        instrumentation_library: &InstrumentationLibrary,
    ) -> Option<(TraceFlags, Vec<KeyValue>, TraceState)> {
        let sampling_result = config.sampler.should_sample(
            Some(parent_cx),
            trace_id,
            name,
            span_kind,
            attributes,
            links,
            instrumentation_library,
        );

        self.process_sampling_result(sampling_result, parent_cx)
    }

    fn process_sampling_result(
        &self,
        sampling_result: SamplingResult,
        parent_cx: &Context,
    ) -> Option<(TraceFlags, Vec<KeyValue>, TraceState)> {
        match sampling_result {
            SamplingResult {
                decision: SamplingDecision::Drop,
                ..
            } => None,
            SamplingResult {
                decision: SamplingDecision::RecordOnly,
                attributes,
                trace_state,
            } => {
                let trace_flags = parent_cx.span().span_context().trace_flags();
                Some((trace_flags.with_sampled(false), attributes, trace_state))
            }
            SamplingResult {
                decision: SamplingDecision::RecordAndSample,
                attributes,
                trace_state,
            } => {
                let trace_flags = parent_cx.span().span_context().trace_flags();
                Some((trace_flags.with_sampled(true), attributes, trace_state))
            }
        }
    }
}

impl opentelemetry_api::trace::Tracer for Tracer {
    /// This implementation of `Tracer` produces `sdk::Span` instances.
    type Span = Span;

    /// Starts a span from a `SpanBuilder`.
    ///
    /// Each span has zero or one parent spans and zero or more child spans, which
    /// represent causally related operations. A tree of related spans comprises a
    /// trace. A span is said to be a _root span_ if it does not have a parent. Each
    /// trace includes a single root span, which is the shared ancestor of all other
    /// spans in the trace.
    fn build_with_context(&self, mut builder: SpanBuilder, parent_cx: &Context) -> Self::Span {
        let provider = self.provider();
        if provider.is_none() {
            return Span::new(
                SpanContext::empty_context(),
                None,
                self.clone(),
                SpanLimits::default(),
            );
        }

        let provider = provider.unwrap();
        let config = provider.config();
        let span_limits = config.span_limits;
        let span_id = builder
            .span_id
            .take()
            .unwrap_or_else(|| config.id_generator.new_span_id());
        let span_kind = builder.span_kind.take().unwrap_or(SpanKind::Internal);
        let mut attribute_options = builder.attributes.take().unwrap_or_default();
        let mut link_options = builder.links.take();
        let mut no_parent = true;
        let mut remote_parent = false;
        let mut parent_span_id = SpanId::INVALID;
        let mut parent_trace_flags = TraceFlags::default();
        let trace_id;

        let parent_span = if parent_cx.has_active_span() {
            Some(parent_cx.span())
        } else {
            None
        };

        // Build context for sampling decision
        if let Some(sc) = parent_span.as_ref().map(|parent| parent.span_context()) {
            no_parent = false;
            remote_parent = sc.is_remote();
            parent_span_id = sc.span_id();
            parent_trace_flags = sc.trace_flags();
            trace_id = sc.trace_id();
        } else {
            trace_id = builder
                .trace_id
                .unwrap_or_else(|| config.id_generator.new_trace_id());
        };

        // There are 3 paths for sampling.
        //
        // * Sampling has occurred elsewhere and is already stored in the builder
        // * There is no parent or a remote parent, in which case make decision now
        // * There is a local parent, in which case defer to the parent's decision
        let sampling_decision = if let Some(sampling_result) = builder.sampling_result.take() {
            self.process_sampling_result(sampling_result, parent_cx)
        } else if no_parent || remote_parent {
            self.make_sampling_decision(
                parent_cx,
                trace_id,
                &builder.name,
                &span_kind,
                &attribute_options,
                link_options.as_deref().unwrap_or(&[]),
                provider.config(),
                &self.instrumentation_lib,
            )
        } else {
            // has parent that is local: use parent if sampled, or don't record.
            parent_span
                .filter(|span| span.span_context().is_sampled())
                .map(|span| {
                    (
                        parent_trace_flags,
                        Vec::new(),
                        span.span_context().trace_state().clone(),
                    )
                })
        };

        let SpanBuilder {
            name,
            start_time,
            end_time,
            events,
            status,
            ..
        } = builder;

        // Build optional inner context, `None` if not recording.
        let mut span = if let Some((flags, extra_attrs, trace_state)) = sampling_decision {
            for extra_attr in extra_attrs {
                attribute_options.insert(extra_attr.key, extra_attr.value);
            }
            let mut attributes =
                EvictedHashMap::new(span_limits.max_attributes_per_span, attribute_options.len());
            for (key, value) in attribute_options {
                attributes.insert(KeyValue::new(key, value));
            }
            let mut links = EvictedQueue::new(span_limits.max_links_per_span);
            if let Some(link_options) = &mut link_options {
                let link_attributes_limit = span_limits.max_attributes_per_link as usize;
                for link in link_options.iter_mut() {
                    let dropped_attributes_count =
                        link.attributes.len().saturating_sub(link_attributes_limit);
                    link.attributes.truncate(link_attributes_limit);
                    link.dropped_attributes_count = dropped_attributes_count as u32;
                }
                links.append_vec(link_options);
            }
            let start_time = start_time.unwrap_or_else(opentelemetry_api::time::now);
            let end_time = end_time.unwrap_or(start_time);
            let mut events_queue = EvictedQueue::new(span_limits.max_events_per_span);
            if let Some(mut events) = events {
                let event_attributes_limit = span_limits.max_attributes_per_event as usize;
                for event in events.iter_mut() {
                    let dropped_attributes_count = event
                        .attributes
                        .len()
                        .saturating_sub(event_attributes_limit);
                    event.attributes.truncate(event_attributes_limit);
                    event.dropped_attributes_count = dropped_attributes_count as u32;
                }
                events_queue.append_vec(&mut events);
            }

            let span_context = SpanContext::new(trace_id, span_id, flags, false, trace_state);
            Span::new(
                span_context,
                Some(SpanData {
                    parent_span_id,
                    span_kind,
                    name,
                    start_time,
                    end_time,
                    attributes,
                    events: events_queue,
                    links,
                    status,
                }),
                self.clone(),
                span_limits,
            )
        } else {
            let span_context = SpanContext::new(
                trace_id,
                span_id,
                TraceFlags::default(),
                false,
                Default::default(),
            );
            Span::new(span_context, None, self.clone(), span_limits)
        };

        // Call `on_start` for all processors
        for processor in provider.span_processors() {
            processor.on_start(&mut span, parent_cx)
        }

        span
    }
}

#[cfg(all(test, feature = "testing", feature = "trace"))]
mod tests {
    use indexmap::IndexMap;
    use crate::{
        testing::trace::TestSpan,
        trace::{Config, Sampler, ShouldSample},
        InstrumentationLibrary,
    };
    use opentelemetry_api::{trace::{
        Link, SamplingDecision, SamplingResult, Span, SpanContext, SpanId, SpanKind,
        TraceContextExt, TraceFlags, TraceId, TraceState, Tracer, TracerProvider,
    }, Context, Value, Key};

    #[derive(Debug)]
    struct TestSampler {}

    impl ShouldSample for TestSampler {
        fn should_sample(
            &self,
            parent_context: Option<&Context>,
            _trace_id: TraceId,
            _name: &str,
            _span_kind: &SpanKind,
            _attributes: &IndexMap<Key, Value>,
            _links: &[Link],
            _instrumentation_library: &InstrumentationLibrary,
        ) -> SamplingResult {
            let trace_state = parent_context
                .unwrap()
                .span()
                .span_context()
                .trace_state()
                .clone();
            SamplingResult {
                decision: SamplingDecision::RecordAndSample,
                attributes: Vec::new(),
                trace_state: trace_state.insert("foo", "notbar").unwrap(),
            }
        }
    }

    #[test]
    fn allow_sampler_to_change_trace_state() {
        // Setup
        let sampler = TestSampler {};
        let config = Config::default().with_sampler(sampler);
        let tracer_provider = crate::trace::TracerProvider::builder()
            .with_config(config)
            .build();
        let tracer = tracer_provider.tracer("test");
        let trace_state = TraceState::from_key_value(vec![("foo", "bar")]).unwrap();

        let parent_context = Context::new().with_span(TestSpan(SpanContext::new(
            TraceId::from_u128(128),
            SpanId::from_u64(64),
            TraceFlags::SAMPLED,
            true,
            trace_state,
        )));

        // Test sampler should change trace state
        let span = tracer.start_with_context("foo", &parent_context);
        let span_context = span.span_context();
        let expected = span_context.trace_state();
        assert_eq!(expected.get("foo"), Some("notbar"))
    }

    #[test]
    fn drop_parent_based_children() {
        let sampler = Sampler::ParentBased(Box::new(Sampler::AlwaysOn));
        let config = Config::default().with_sampler(sampler);
        let tracer_provider = crate::trace::TracerProvider::builder()
            .with_config(config)
            .build();

        let context = Context::current_with_span(TestSpan(SpanContext::empty_context()));
        let tracer = tracer_provider.tracer("test");
        let span = tracer.start_with_context("must_not_be_sampled", &context);

        assert!(!span.span_context().is_sampled());
    }

    #[test]
    fn uses_current_context_for_builders_if_unset() {
        let sampler = Sampler::ParentBased(Box::new(Sampler::AlwaysOn));
        let config = Config::default().with_sampler(sampler);
        let tracer_provider = crate::trace::TracerProvider::builder()
            .with_config(config)
            .build();
        let tracer = tracer_provider.tracer("test");

        let _attached = Context::current_with_span(TestSpan(SpanContext::empty_context())).attach();
        let span = tracer.span_builder("must_not_be_sampled").start(&tracer);
        assert!(!span.span_context().is_sampled());

        let _attached = Context::current()
            .with_remote_span_context(SpanContext::new(
                TraceId::from_u128(1),
                SpanId::from_u64(1),
                TraceFlags::default(),
                true,
                Default::default(),
            ))
            .attach();
        let span = tracer.span_builder("must_not_be_sampled").start(&tracer);

        assert!(!span.span_context().is_sampled());
    }
}
