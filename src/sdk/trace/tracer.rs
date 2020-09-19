//! # Tracer
//!
//! The OpenTelemetry library achieves in-process context propagation of
//! `Span`s by way of the `Tracer`.
//!
//! The `Tracer` is responsible for tracking the currently active `Span`,
//! and exposes methods for creating and activating new `Spans`.
//!
//! Docs: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/api-tracing.md#tracer
use crate::api::trace::span_context::TraceState;
use crate::api::TraceContextExt;
use crate::sdk;
use crate::{api, api::context::Context, exporter};
use std::fmt;
use std::sync::Arc;
use std::time::SystemTime;

/// `Tracer` implementation to create and manage spans
#[derive(Clone)]
pub struct Tracer {
    instrumentation_lib: sdk::InstrumentationLibrary,
    provider: sdk::TracerProvider,
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
        provider: sdk::TracerProvider,
    ) -> Self {
        Tracer {
            instrumentation_lib,
            provider,
        }
    }

    /// TracerProvider associated with this tracer.
    pub fn provider(&self) -> &sdk::TracerProvider {
        &self.provider
    }

    /// instrumentation library information of this tracer.
    pub fn instrumentation_library(&self) -> &sdk::InstrumentationLibrary {
        &self.instrumentation_lib
    }

    /// Make a sampling decision using the provided sampler for the span and context.
    #[allow(clippy::too_many_arguments)]
    fn make_sampling_decision(
        &self,
        parent_context: Option<&api::SpanContext>,
        trace_id: api::TraceId,
        name: &str,
        span_kind: &api::SpanKind,
        attributes: &[api::KeyValue],
        links: &[api::Link],
    ) -> Option<(u8, Vec<api::KeyValue>)> {
        let sampler = &self.provider.config().default_sampler;
        let sampling_result =
            sampler.should_sample(parent_context, trace_id, name, span_kind, attributes, links);

        self.process_sampling_result(sampling_result, parent_context)
    }

    fn process_sampling_result(
        &self,
        sampling_result: sdk::SamplingResult,
        parent_context: Option<&api::SpanContext>,
    ) -> Option<(u8, Vec<api::KeyValue>)> {
        match sampling_result {
            sdk::SamplingResult {
                decision: sdk::SamplingDecision::NotRecord,
                ..
            } => None,
            sdk::SamplingResult {
                decision: sdk::SamplingDecision::Record,
                attributes,
            } => {
                let trace_flags = parent_context.map(|ctx| ctx.trace_flags()).unwrap_or(0);
                Some((trace_flags & !api::TRACE_FLAG_SAMPLED, attributes))
            }
            sdk::SamplingResult {
                decision: sdk::SamplingDecision::RecordAndSampled,
                attributes,
            } => {
                let trace_flags = parent_context.map(|ctx| ctx.trace_flags()).unwrap_or(0);
                Some((trace_flags | api::TRACE_FLAG_SAMPLED, attributes))
            }
        }
    }
}

impl api::Tracer for Tracer {
    /// This implementation of `api::Tracer` produces `sdk::Span` instances.
    type Span = sdk::Span;

    /// Returns a span with an inactive `SpanContext`. Used by functions that
    /// need to return a default span like `get_active_span` if no span is present.
    fn invalid(&self) -> Self::Span {
        sdk::Span::new(api::SpanId::invalid(), None, self.clone())
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
    fn span_builder(&self, name: &str) -> api::SpanBuilder {
        api::SpanBuilder::from_name(name.to_string())
    }

    /// Starts a span from a `SpanBuilder`.
    ///
    /// Each span has zero or one parent spans and zero or more child spans, which
    /// represent causally related operations. A tree of related spans comprises a
    /// trace. A span is said to be a _root span_ if it does not have a parent. Each
    /// trace includes a single root span, which is the shared ancestor of all other
    /// spans in the trace.
    fn build_with_context(&self, mut builder: api::SpanBuilder, cx: &Context) -> Self::Span {
        let config = self.provider.config();
        let span_id = builder
            .span_id
            .take()
            .unwrap_or_else(|| self.provider().config().id_generator.new_span_id());

        let span_kind = builder.span_kind.take().unwrap_or(api::SpanKind::Internal);
        let mut attribute_options = builder.attributes.take().unwrap_or_else(Vec::new);
        let mut link_options = builder.links.take().unwrap_or_else(Vec::new);

        let parent_span_context = builder
            .parent_context
            .take()
            .or_else(|| Some(cx.span().span_context()).filter(|cx| cx.is_valid()))
            .or_else(|| cx.remote_span_context().cloned())
            .filter(|cx| cx.is_valid());
        // Build context for sampling decision
        let (no_parent, trace_id, parent_span_id, remote_parent, parent_trace_flags, trace_state) =
            parent_span_context
                .as_ref()
                .map(|ctx| {
                    (
                        false,
                        ctx.trace_id(),
                        ctx.span_id(),
                        ctx.is_remote(),
                        ctx.trace_flags(),
                        ctx.trace_state().clone(),
                    )
                })
                .unwrap_or((
                    true,
                    builder
                        .trace_id
                        .unwrap_or_else(|| self.provider().config().id_generator.new_trace_id()),
                    api::SpanId::invalid(),
                    false,
                    0,
                    TraceState::default(),
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
                .map(|_| (parent_trace_flags, Vec::new()))
        };

        // Build optional inner context, `None` if not recording.
        let inner = sampling_decision.map(move |(trace_flags, mut extra_attrs)| {
            attribute_options.append(&mut extra_attrs);
            let mut attributes = sdk::EvictedHashMap::new(config.max_attributes_per_span);
            for attribute in attribute_options {
                attributes.insert(attribute);
            }
            let mut links = sdk::EvictedQueue::new(config.max_links_per_span);
            links.append_vec(&mut link_options);
            let start_time = builder.start_time.unwrap_or_else(SystemTime::now);
            let end_time = builder.end_time.unwrap_or(start_time);
            let mut message_events = sdk::EvictedQueue::new(config.max_events_per_span);
            if let Some(mut events) = builder.message_events {
                message_events.append_vec(&mut events);
            }
            let status_code = builder.status_code.unwrap_or(api::StatusCode::OK);
            let status_message = builder.status_message.unwrap_or_else(String::new);
            let resource = config.resource.clone();

            exporter::trace::SpanData {
                span_context: api::SpanContext::new(
                    trace_id,
                    span_id,
                    trace_flags,
                    false,
                    trace_state,
                ),
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
            let inner_data = Arc::new(inner);
            for processor in self.provider.span_processors() {
                processor.on_start(inner_data.clone())
            }
        }

        sdk::Span::new(span_id, inner, self.clone())
    }
}
