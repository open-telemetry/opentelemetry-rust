//! # Tracer
//!
//! The OpenTelemetry library achieves in-process context propagation of
//! `Span`s by way of the `Tracer`.
//!
//! The `Tracer` is responsible for tracking the currently active `Span`,
//! and exposes methods for creating and activating new `Spans`.
//!
//! Docs: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/api-tracing.md#tracer
use crate::api::Span;
use crate::sdk;
use crate::{api, exporter};
use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::SystemTime;

/// `Tracer` implementation to create and manage spans
#[derive(Clone, Debug)]
pub struct Tracer {
    name: &'static str,
    provider: sdk::Provider,
}

impl Tracer {
    /// Create a new tracer (used internally by `Provider`s.
    pub(crate) fn new(name: &'static str, provider: sdk::Provider) -> Self {
        Tracer { name, provider }
    }

    /// Provider associated with this tracer
    pub fn provider(&self) -> &sdk::Provider {
        &self.provider
    }

    /// Make a sampling decision using the provided sampler for the span and context.
    #[allow(clippy::too_many_arguments)]
    fn make_sampling_decision(
        &self,
        parent_context: Option<&api::SpanContext>,
        trace_id: u128,
        span_id: u64,
        name: &str,
        span_kind: &api::SpanKind,
        attributes: &[api::KeyValue],
        links: &[api::Link],
    ) -> Option<(u8, Vec<api::KeyValue>)> {
        let sampler = &self.provider.config().default_sampler;
        match sampler.should_sample(
            parent_context,
            trace_id,
            span_id,
            name,
            span_kind,
            attributes,
            links,
        ) {
            api::SamplingResult {
                decision: api::SamplingDecision::NotRecord,
                ..
            } => None,
            api::SamplingResult {
                decision: api::SamplingDecision::Record,
                attributes,
            } => {
                let trace_flags = parent_context.map(|ctx| ctx.trace_flags()).unwrap_or(0);
                Some((trace_flags & !api::TRACE_FLAG_SAMPLED, attributes))
            }
            api::SamplingResult {
                decision: api::SamplingDecision::RecordAndSampled,
                attributes,
            } => {
                let trace_flags = parent_context.map(|ctx| ctx.trace_flags()).unwrap_or(0);
                Some((trace_flags | api::TRACE_FLAG_SAMPLED, attributes))
            }
        }
    }
}

thread_local! {
    /// Track currently active `Span` per thread via a `SpanStack`.
    static CURRENT_SPANS: RefCell<SpanStack> = RefCell::new(SpanStack::new());
}

impl api::Tracer for Tracer {
    /// This implementation of `api::Tracer` produces `sdk::Span` instances.
    type Span = sdk::Span;

    /// Returns a span with an inactive `SpanContext`. Used by functions that
    /// need to return a default span like `get_active_span` if no span is present.
    fn invalid(&self) -> Self::Span {
        sdk::Span::new(0, None, self.clone())
    }

    /// Starts a new `Span`.
    ///
    /// Each span has zero or one parent spans and zero or more child spans, which
    /// represent causally related operations. A tree of related spans comprises a
    /// trace. A span is said to be a _root span_ if it does not have a parent. Each
    /// trace includes a single root span, which is the shared ancestor of all other
    /// spans in the trace.
    fn start(&self, name: &str, parent_span: Option<api::SpanContext>) -> Self::Span {
        let name = format!("{}/{}", self.name, name);
        let config = self.provider.config();
        let span_id: u64 = rand::random();

        // TODO allow the following to be set when starting span
        let span_kind = api::SpanKind::Internal;
        let mut attribute_options = Vec::new();
        let mut link_options = Vec::new();

        // Build context for sampling decision
        let (no_parent, trace_id, parent_span_id, remote_parent, parent_trace_flags) = parent_span
            .clone()
            .or_else(|| Some(self.get_active_span().get_context()))
            .filter(|ctx| ctx.is_valid())
            .map(|ctx| {
                (
                    false,
                    ctx.trace_id(),
                    ctx.span_id(),
                    ctx.is_remote(),
                    ctx.trace_flags(),
                )
            })
            .unwrap_or((true, rand::random(), 0, false, 0));

        // Make new sampling decision or use parent sampling decision
        let sampling_decision = if no_parent || remote_parent {
            self.make_sampling_decision(
                parent_span.as_ref(),
                trace_id,
                span_id,
                &name,
                &span_kind,
                &attribute_options,
                &link_options,
            )
        } else {
            Some((parent_trace_flags, Vec::new()))
        };

        // Build optional inner context, `None` if not recording.
        let inner = sampling_decision.map(move |(trace_flags, mut extra_attrs)| {
            attribute_options.append(&mut extra_attrs);
            let mut attributes = sdk::EvictedQueue::new(config.max_attributes_per_span);
            attributes.append_vec(&mut attribute_options);
            let mut links = sdk::EvictedQueue::new(config.max_links_per_span);
            links.append_vec(&mut link_options);

            exporter::trace::SpanData {
                context: api::SpanContext::new(trace_id, span_id, trace_flags, false),
                parent_span_id,
                span_kind,
                name,
                start_time: SystemTime::now(),
                end_time: SystemTime::now(),
                attributes,
                message_events: sdk::EvictedQueue::new(config.max_events_per_span),
                links,
                status: api::SpanStatus::OK,
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

    /// Returns the current active span.
    ///
    /// When getting the current `Span`, the `Tracer` will return a placeholder
    /// `Span` with an invalid `SpanContext` if there is no currently active `Span`.
    fn get_active_span(&self) -> Self::Span {
        CURRENT_SPANS
            .with(|spans| spans.borrow().current())
            .unwrap_or_else(|| self.invalid())
    }

    /// Mark a given `Span` as active.
    fn mark_span_as_active(&self, span: &Self::Span) {
        CURRENT_SPANS.with(|spans| {
            spans.borrow_mut().push(span.clone());
        })
    }

    /// Mark a given `Span` as inactive.
    fn mark_span_as_inactive(&self, span_id: u64) {
        CURRENT_SPANS.with(|spans| {
            spans.borrow_mut().pop(span_id);
        })
    }

    /// Clone span
    fn clone_span(&self, span: &Self::Span) -> Self::Span {
        span.clone()
    }
}

/// Used to track `Span` and its status in the stack
struct ContextId {
    span: sdk::Span,
    duplicate: bool,
}

/// A stack of `Span`s that can be used to track active `Span`s per thread.
pub(crate) struct SpanStack {
    stack: Vec<ContextId>,
    ids: HashSet<u64>,
}

impl SpanStack {
    /// Create a new `SpanStack`
    fn new() -> Self {
        SpanStack {
            stack: vec![],
            ids: HashSet::new(),
        }
    }

    /// Push a `Span` to the stack
    fn push(&mut self, span: sdk::Span) {
        let duplicate = self.ids.contains(&span.id());
        if !duplicate {
            self.ids.insert(span.id());
        }
        self.stack.push(ContextId { span, duplicate })
    }

    /// Pop a `Span` from the stack
    fn pop(&mut self, expected_id: u64) -> Option<sdk::Span> {
        if self.stack.last()?.span.id() == expected_id {
            let ContextId { span, duplicate } = self.stack.pop()?;
            if !duplicate {
                self.ids.remove(&span.id());
            }
            Some(span)
        } else {
            None
        }
    }

    /// Find the latest `Span` that is not doubly marked as active (pushed twice)
    #[inline]
    fn current(&self) -> Option<sdk::Span> {
        self.stack
            .iter()
            .rev()
            .find(|context_id| !context_id.duplicate)
            .map(|context_id| context_id.span.clone())
    }
}
