/// The OpenTelemetry library achieves in-process context propagation of `Span`s by way of the
/// `Tracer`.
///
/// The `Tracer` is responsible for tracking the currently active `Span`, and exposes methods for
/// creating and activating new `Spans`. The `Tracer` is configured with `Propagators` which support
/// transferring span context across process boundaries.
///
/// `Tracer`s are generally expected to be used as singletons. Implementations SHOULD provide a
/// single global default Tracer.
///
/// Some applications may require multiple `Tracer` instances, e.g. to create `Span`s on behalf of
/// other applications. Implementations MAY provide a global registry of Tracers for such
/// applications.
///
/// Docs: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/api-tracing.md#tracer
use crate::api;

pub trait Tracer {
    type Span: api::Span;

    fn invalid(&self) -> Self::Span;

    // Starts a new span.
    //
    // When creating a new `Span`, the `Tracer` MUST allow the caller to specify the new `Span`'s
    // parent in the form of a Span or SpanContext. The Tracer SHOULD create each new Span as a
    // child of its active `Span` unless an explicit parent is provided or the option to create a
    // `Span` without a parent is selected, or the current active `Span` is invalid.
    fn start(&self, name: String, parent_span: Option<api::SpanContext>) -> Self::Span;

    fn with_span(&self, name: String, f: Box<dyn FnOnce(Self::Span)>) {
        // TODO: use active as parent, fix lifetimes...
        let context: Option<api::SpanContext> = None;
        let span = self.start(name.into(), context);

        f(span)
    }

    // Returns the current active span.
    //
    // When getting the current `Span`, the `Tracer` MUST return a placeholder `Span` with an invalid
    // `SpanContext` if there is no currently active `Span`.
    fn get_active_span(&self) -> Self::Span;

    // Returns the matching `Span`, or a `Span` with an invalid `SpanContext` if there is no
    // matching `Span`.
    fn get_span_by_id(&self, span_id: u64) -> Self::Span;

    // Mark a given span as active by id.
    //
    // The `Tracer` MUST provide a way to update its active `Span`, and MAY provide convenience
    // methods to manage a `Span`'s lifetime and the scope in which a `Span` is active. When an
    // active `Span` is made inactive, the previously-active `Span` SHOULD be made active. A `Span`
    // maybe finished (i.e. have a non-null end time) but still be active. A `Span` may be active
    // on one thread after it has been made inactive on another.
    fn mark_span_as_active(&self, span_id: u64);
}
