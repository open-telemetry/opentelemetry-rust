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
use std::fmt;
use std::time::SystemTime;

/// Interface for constructing `Span`s.
pub trait Tracer: fmt::Debug + 'static {
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
    fn start(&self, name: &str, parent_span: Option<api::SpanContext>) -> Self::Span;

    /// Creates a span builder
    ///
    /// An ergonomic way for attributes to be configured before the `Span` is started.
    fn span_builder(&self, name: &str) -> SpanBuilder;

    /// Create a span from a `SpanBuilder`
    fn build(&self, builder: SpanBuilder) -> Self::Span;

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
    fn mark_span_as_inactive(&self, span_id: api::SpanId);

    /// Clone a span created by this tracer.
    fn clone_span(&self, span: &Self::Span) -> Self::Span;
}

/// TracerGenerics are functions that have generic type parameters. They are a separate
/// trait so that `Tracer` can be used as a trait object in `GlobalTracer`.
pub trait TracerGenerics: Tracer {
    /// Wraps the execution of the function body with a span.
    /// It starts a new span and sets it as the active span for the given function.
    /// It then executes the body. It closes the span before returning the execution result.
    fn with_span<T, F>(&self, name: &'static str, f: F) -> T
    where
        F: FnOnce(&Self::Span) -> T;
}

// These functions can be implemented for all tracers to allow for convenient `with_span` syntax.
impl<S: Tracer> TracerGenerics for S {
    /// Wraps the execution of the function body with a span.
    /// It starts a new span and sets it as the active span for the given function.
    /// It then executes the body. It closes the span before returning the execution result.
    fn with_span<T, F>(&self, name: &'static str, f: F) -> T
    where
        F: FnOnce(&Self::Span) -> T,
    {
        let span = self.start(name, None);
        self.mark_span_as_active(&span);

        let result = f(&span);
        span.end();
        self.mark_span_as_inactive(span.get_context().span_id());

        result
    }
}

/// `SpanBuilder` allows span attributes to be configured before the span
/// has started.
///
/// ```rust
/// use opentelemetry::{
///     api::{Provider, SpanBuilder, SpanKind, Tracer},
///     global,
/// };
///
/// let tracer = global::tracer("example-tracer");
///
/// // The builder can be used to create a span directly with the tracer
/// let _span = tracer.build(SpanBuilder {
///     name: "example-span-name".to_string(),
///     span_kind: Some(SpanKind::Server),
///     ..Default::default()
/// });
///
/// // Or used with builder pattern
/// let _span = tracer
///     .span_builder("example-span-name")
///     .with_kind(SpanKind::Server)
///     .start(&tracer);
/// ```
#[derive(Debug, Default)]
pub struct SpanBuilder {
    /// Parent `SpanContext`
    pub parent_context: Option<api::SpanContext>,
    /// Trace id, useful for integrations with external tracing systems.
    pub trace_id: Option<api::TraceId>,
    /// Span id, useful for integrations with external tracing systems.
    pub span_id: Option<api::SpanId>,
    /// Span kind
    pub span_kind: Option<api::SpanKind>,
    /// Span name
    pub name: String,
    /// Span start time
    pub start_time: Option<SystemTime>,
    /// Span end time
    pub end_time: Option<SystemTime>,
    /// Span attributes
    pub attributes: Option<Vec<api::KeyValue>>,
    /// Span Message events
    pub message_events: Option<Vec<api::Event>>,
    /// Span Links
    pub links: Option<Vec<api::Link>>,
    /// Span status code
    pub status_code: Option<api::StatusCode>,
    /// Span status message
    pub status_message: Option<String>,
}

/// SpanBuilder methods
impl SpanBuilder {
    /// Create a new span builder from a span name
    pub fn from_name(name: String) -> Self {
        SpanBuilder {
            parent_context: None,
            trace_id: None,
            span_id: None,
            span_kind: None,
            name,
            start_time: None,
            end_time: None,
            attributes: None,
            message_events: None,
            links: None,
            status_code: None,
            status_message: None,
        }
    }

    /// Assign parent context
    pub fn with_parent(self, parent_context: api::SpanContext) -> Self {
        SpanBuilder {
            parent_context: Some(parent_context),
            ..self
        }
    }

    /// Specify trace id to use if no parent context exists
    pub fn with_trace_id(self, trace_id: api::TraceId) -> Self {
        SpanBuilder {
            trace_id: Some(trace_id),
            ..self
        }
    }

    /// Assign span id
    pub fn with_span_id(self, span_id: api::SpanId) -> Self {
        SpanBuilder {
            span_id: Some(span_id),
            ..self
        }
    }

    /// Assign span kind
    pub fn with_kind(self, span_kind: api::SpanKind) -> Self {
        SpanBuilder {
            span_kind: Some(span_kind),
            ..self
        }
    }

    /// Assign span start time
    pub fn with_start_time<T: Into<SystemTime>>(self, start_time: T) -> Self {
        SpanBuilder {
            start_time: Some(start_time.into()),
            ..self
        }
    }

    /// Assign span end time
    pub fn with_end_time<T: Into<SystemTime>>(self, end_time: T) -> Self {
        SpanBuilder {
            end_time: Some(end_time.into()),
            ..self
        }
    }

    /// Assign span attributes
    pub fn with_attributes(self, attributes: Vec<api::KeyValue>) -> Self {
        SpanBuilder {
            attributes: Some(attributes),
            ..self
        }
    }

    /// Assign message events
    pub fn with_message_events(self, message_events: Vec<api::Event>) -> Self {
        SpanBuilder {
            message_events: Some(message_events),
            ..self
        }
    }

    /// Assign links
    pub fn with_links(self, links: Vec<api::Link>) -> Self {
        SpanBuilder {
            links: Some(links),
            ..self
        }
    }

    /// Assign status code
    pub fn with_status_code(self, code: api::StatusCode) -> Self {
        SpanBuilder {
            status_code: Some(code),
            ..self
        }
    }

    /// Assign status message
    pub fn with_status_message(self, message: String) -> Self {
        SpanBuilder {
            status_message: Some(message),
            ..self
        }
    }

    /// Builds a span with the given tracer from this configuration.
    pub fn start<T: api::Tracer>(self, tracer: &T) -> T::Span {
        tracer.build(self)
    }
}
