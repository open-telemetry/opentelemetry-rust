//! # Trace Tracer SDK
//!
//! The OpenTelemetry library achieves in-process context propagation of
//! `Span`s by way of the `Tracer`.
//!
//! The `Tracer` is responsible for tracking the currently active `Span`,
//! and exposes methods for creating and activating new `Spans`.
//!
//! Docs: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/api-tracing.md#tracer
use crate::api;
use crate::exporter::trace::jaeger;
use crate::sdk;
use std::cell::RefCell;
use std::collections::HashSet;

/// `Tracer` implementation to create and manage spans
#[derive(Clone, Debug)]
pub struct Tracer {
    name: &'static str,
    inner: jaeger::Tracer,
}

impl Tracer {
    /// Create a new tracer (used internally by `Provider`s.
    pub(crate) fn new(name: &'static str, inner: jaeger::Tracer) -> Self {
        Tracer { name, inner }
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
        sdk::Span::new(0, jaeger::Span::inactive())
    }

    /// Starts a new `Span`.
    ///
    /// Each span has zero or one parent spans and zero or more child spans, which
    /// represent causally related operations. A tree of related spans comprises a
    /// trace. A span is said to be a _root span_ if it does not have a parent. Each
    /// trace includes a single root span, which is the shared ancestor of all other
    /// spans in the trace.
    fn start(&self, name: &str, parent_span: Option<api::SpanContext>) -> Self::Span {
        let start_options = self.inner.span(format!("{}/{}", self.name, name));
        let started = match parent_span
            .filter(|ctx| ctx.is_valid())
            .map(jaeger::SpanContext::from)
        {
            Some(span_context) => start_options.child_of(&span_context).start(),
            None => start_options.start(),
        };
        let span_id = started
            .context()
            .map(|ctx| ctx.state().span_id())
            .unwrap_or(0);

        sdk::Span::new(span_id, started)
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
