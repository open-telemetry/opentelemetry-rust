//! # OpenTelemetry Tracer interface
//!
//! The OpenTelemetry library achieves in-process context propagation of
//! `Span`s by way of the `Tracer`.
//!
//! The `Tracer` is responsible for tracking the currently active `Span`,
//! and exposes methods for creating and activating new `Spans`. The
//! `Tracer` is configured with `Propagators` which support transferring
//! span context across process boundaries.
//!
//! `Tracer`s are generally expected to be used as singletons.
//! Implementations SHOULD provide a single global default Tracer.
//!
//! Some applications may require multiple `Tracer` instances, e.g. to
//! create `Span`s on behalf of other applications. Implementations MAY
//! provide a global registry of Tracers for such applications.
//!
//! The `Tracer` SHOULD allow end users to configure other tracing components that
//! control how `Span`s are passed across process boundaries, including the binary
//! and text format `Propagator`s used to serialize `Span`s created by the
//! `Tracer`.
//!
//! Docs: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/api-tracing.md#tracer
use crate::api::{self, Span};

/// Interface for constructing `Span`s.
pub trait Tracer: Send + Sync {
    /// The `Span` type used by this `Tracer`.
    type Span: api::Span;

    /// Returns a span with an invalid `SpanContext`. Used by functions that
    /// need to return a default span like `get_active_span` if no span is present.
    fn invalid(&self) -> Self::Span;

    /// Starts a new `Span`.
    ///
    /// By default the currently active `Span` is set as the new `Span`'s
    /// parent. The `Tracer` MAY provide other default options for newly
    /// created `Span`s.
    ///
    /// `Span` creation MUST NOT set the newly created `Span` as the currently
    /// active `Span` by default, but this functionality MAY be offered additionally
    /// as a separate operation.
    ///
    /// Each span has zero or one parent spans and zero or more child spans, which
    /// represent causally related operations. A tree of related spans comprises a
    /// trace. A span is said to be a _root span_ if it does not have a parent. Each
    /// trace includes a single root span, which is the shared ancestor of all other
    /// spans in the trace. Implementations MUST provide an option to create a `Span` as
    /// a root span, and MUST generate a new `TraceId` for each root span created.
    /// For a Span with a parent, the `TraceId` MUST be the same as the parent.
    /// Also, the child span MUST inherit all `TraceState` values of its parent by default.
    ///
    /// A `Span` is said to have a _remote parent_ if it is the child of a `Span`
    /// created in another process. Each propagators' deserialization must set
    /// `is_remote` to true on a parent `SpanContext` so `Span` creation knows if the
    /// parent is remote.
    fn start(&self, name: &'static str, parent_span: Option<api::SpanContext>) -> Self::Span;

    /// Returns the current active span.
    ///
    /// When getting the current `Span`, the `Tracer` MUST return a placeholder
    /// `Span` with an invalid `SpanContext` if there is no currently active `Span`.
    fn get_active_span(&self) -> Self::Span;

    /// Mark a given `Span` as active.
    ///
    /// The `Tracer` MUST provide a way to update its active `Span`, and MAY provide convenience
    /// methods to manage a `Span`'s lifetime and the scope in which a `Span` is active. When an
    /// active `Span` is made inactive, the previously-active `Span` SHOULD be made active. A `Span`
    /// maybe finished (i.e. have a non-null end time) but still be active. A `Span` may be active
    /// on one thread after it has been made inactive on another.
    ///
    /// NOTE: The `mark_span_as_active`/`mark_span_as_inactive` functions MUST be used
    /// together or you can end up retaining references to the currently active `Span`.
    /// If you do not want to manage active state of `Span`s manually, use the `with_span`
    /// API defined for all `Tracer`s via `TracerGenerics`
    fn mark_span_as_active(&self, span: &Self::Span);

    /// Remove span from active span
    ///
    /// When an active `Span` is made inactive, the previously-active `Span` SHOULD be
    /// made active. A `Span` maybe finished (i.e. have a non-null end time) but still
    /// be active. A `Span` may be active on one thread after it has been made inactive
    /// on another.
    ///
    /// NOTE: The `mark_span_as_active`/`mark_span_as_inactive` functions MUST be used
    /// together or you can end up retaining references to the currently active `Span`.
    /// If you do not want to manage active state of `Span`s manually, use the `with_span`
    /// API defined for all `Tracer`s via `TracerGenerics`
    fn mark_span_as_inactive(&self, span_id: u64);
}

/// TracerGenerics are functions that have generic type parameters. They are a separate
/// trait so that `Tracer` can be used as a trait object in `GlobalTracer`.
pub trait TracerGenerics: Tracer {
    /// Wraps the execution of the function body with a span.
    /// It starts a new span and sets it as the active span for the given function.
    /// It then executes the body. It closes the span before returning the execution result.
    fn with_span<T, F>(&self, name: &'static str, f: F) -> T
    where
        F: FnOnce(&mut Self::Span) -> T;
}

// These functions can be implemented for all tracers to allow for convenient `with_span` syntax.
impl<S: Tracer> TracerGenerics for S {
    /// Wraps the execution of the function body with a span.
    /// It starts a new span and sets it as the active span for the given function.
    /// It then executes the body. It closes the span before returning the execution result.
    fn with_span<T, F>(&self, name: &'static str, f: F) -> T
    where
        F: FnOnce(&mut Self::Span) -> T,
    {
        let active_context = self.get_active_span().get_context();
        let parent = if active_context.is_valid() {
            Some(active_context)
        } else {
            None
        };

        let mut span = self.start(name, parent);
        self.mark_span_as_active(&span);

        let result = f(&mut span);
        span.end();
        self.mark_span_as_inactive(span.get_context().span_id());

        result
    }
}
