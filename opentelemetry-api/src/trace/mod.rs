//! API for tracing applications and libraries.
//!
//! The `trace` module includes types for tracking the progression of a single
//! request while it is handled by services that make up an application. A trace
//! is a tree of [`Span`]s which are objects that represent the work being done
//! by individual services or components involved in a request as it flows
//! through a system. This module implements the OpenTelemetry [trace
//! specification].
//!
//! [trace specification]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.3.0/specification/trace/api.md
//!
//! ## Getting Started
//!
//! ```
//! use opentelemetry_api::{global, trace::{Span, Tracer, TracerProvider}};
//!
//! fn my_library_function() {
//!     // Use the global tracer provider to get access to the user-specified
//!     // tracer configuration
//!     let tracer_provider = global::tracer_provider();
//!
//!     // Get a tracer for this library
//!     let tracer = tracer_provider.versioned_tracer(
//!         "my_name",
//!         Some(env!("CARGO_PKG_VERSION")),
//!         None
//!     );
//!
//!     // Create spans
//!     let mut span = tracer.start("doing_work");
//!
//!     // Do work...
//!
//!     // End the span
//!     span.end();
//! }
//! ```
//!
//! ## Overview
//!
//! The tracing API consists of a three main traits:
//!
//! * [`TracerProvider`]s are the entry point of the API. They provide access to
//!   `Tracer`s.
//! * [`Tracer`]s are types responsible for creating `Span`s.
//! * [`Span`]s provide the API to trace an operation.
//!
//! ## Working with Async Runtimes
//!
//! Exporting spans often involves sending data over a network or performing
//! other I/O tasks. OpenTelemetry allows you to schedule these tasks using
//! whichever runtime you area already using such as [Tokio] or [async-std].
//! When using an async runtime it's best to use the batch span processor
//! where the spans will be sent in batches as opposed to being sent once ended,
//! which often ends up being more efficient.
//!
//! [Tokio]: https://tokio.rs
//! [async-std]: https://async.rs
//!
//! ## Managing Active Spans
//!
//! Spans can be marked as "active" for a given [`Context`], and all newly
//! created spans will automatically be children of the currently active span.
//!
//! The active span for a given thread can be managed via [`get_active_span`]
//! and [`mark_span_as_active`].
//!
//! [`Context`]: crate::Context
//!
//! ```
//! use opentelemetry_api::{global, trace::{self, Span, Status, Tracer, TracerProvider}};
//!
//! fn may_error(rand: f32) {
//!     if rand < 0.5 {
//!         // Get the currently active span to record additional attributes,
//!         // status, etc.
//!         trace::get_active_span(|span| {
//!             span.set_status(Status::error("value too small"));
//!         });
//!     }
//! }
//!
//! // Get a tracer
//! let tracer = global::tracer("my_tracer");
//!
//! // Create a span
//! let span = tracer.start("parent_span");
//!
//! // Mark the span as active
//! let active = trace::mark_span_as_active(span);
//!
//! // Any span created here will be a child of `parent_span`...
//!
//! // Drop the guard and the span will no longer be active
//! drop(active)
//! ```
//!
//! Additionally [`Tracer::in_span`] can be used as shorthand to simplify
//! managing the parent context.
//!
//! ```
//! use opentelemetry_api::{global, trace::Tracer};
//!
//! // Get a tracer
//! let tracer = global::tracer("my_tracer");
//!
//! // Use `in_span` to create a new span and mark it as the parent, dropping it
//! // at the end of the block.
//! tracer.in_span("parent_span", |cx| {
//!     // spans created here will be children of `parent_span`
//! });
//! ```
//!
//! #### Async active spans
//!
//! Async spans can be propagated with [`TraceContextExt`] and [`FutureExt`].
//!
//! ```
//! use opentelemetry_api::{Context, global, trace::{FutureExt, TraceContextExt, Tracer}};
//!
//! async fn some_work() { }
//!
//! // Get a tracer
//! let tracer = global::tracer("my_tracer");
//!
//! // Start a span
//! let span = tracer.start("my_span");
//!
//! // Perform some async work with this span as the currently active parent.
//! some_work().with_context(Context::current_with_span(span));
//! ```

use futures_channel::{mpsc::TrySendError, oneshot::Canceled};
use std::borrow::Cow;
use std::time;
use thiserror::Error;

mod context;
pub mod noop;
mod span;
mod span_context;
mod tracer;
mod tracer_provider;

pub use self::{
    context::{get_active_span, mark_span_as_active, FutureExt, SpanRef, TraceContextExt},
    span::{Span, SpanKind, Status},
    span_context::{SpanContext, SpanId, TraceFlags, TraceId, TraceState},
    tracer::{SamplingDecision, SamplingResult, SpanBuilder, Tracer},
    tracer_provider::TracerProvider,
};
use crate::{ExportError, KeyValue};

/// Describe the result of operations in tracing API.
pub type TraceResult<T> = Result<T, TraceError>;

/// Errors returned by the trace API.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum TraceError {
    /// Export failed with the error returned by the exporter
    #[error("Exporter {} encountered the following error(s): {0}", .0.exporter_name())]
    ExportFailed(Box<dyn ExportError>),

    /// Export failed to finish after certain period and processor stopped the export.
    #[error("Exporting timed out after {} seconds", .0.as_secs())]
    ExportTimedOut(time::Duration),

    /// Other errors propagated from trace SDK that weren't covered above
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl<T> From<T> for TraceError
where
    T: ExportError,
{
    fn from(err: T) -> Self {
        TraceError::ExportFailed(Box::new(err))
    }
}

impl<T> From<TrySendError<T>> for TraceError {
    fn from(err: TrySendError<T>) -> Self {
        TraceError::Other(Box::new(err.into_send_error()))
    }
}

impl From<Canceled> for TraceError {
    fn from(err: Canceled) -> Self {
        TraceError::Other(Box::new(err))
    }
}

impl From<String> for TraceError {
    fn from(err_msg: String) -> Self {
        TraceError::Other(Box::new(Custom(err_msg)))
    }
}

impl From<&'static str> for TraceError {
    fn from(err_msg: &'static str) -> Self {
        TraceError::Other(Box::new(Custom(err_msg.into())))
    }
}

/// Wrap type for string
#[derive(Error, Debug)]
#[error("{0}")]
struct Custom(String);

/// Events record things that happened during a [`Span`]'s lifetime.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    /// The name of this event.
    pub name: Cow<'static, str>,

    /// The time at which this event occurred.
    pub timestamp: time::SystemTime,

    /// Attributes that describe this event.
    pub attributes: Vec<KeyValue>,

    /// The number of attributes that were above the configured limit, and thus
    /// dropped.
    pub dropped_attributes_count: u32,
}

impl Event {
    /// Create new `Event`
    pub fn new<T: Into<Cow<'static, str>>>(
        name: T,
        timestamp: time::SystemTime,
        attributes: Vec<KeyValue>,
        dropped_attributes_count: u32,
    ) -> Self {
        Event {
            name: name.into(),
            timestamp,
            attributes,
            dropped_attributes_count,
        }
    }

    /// Create new `Event` with a given name.
    pub fn with_name<T: Into<Cow<'static, str>>>(name: T) -> Self {
        Event {
            name: name.into(),
            timestamp: crate::time::now(),
            attributes: Vec::new(),
            dropped_attributes_count: 0,
        }
    }
}

/// Link is the relationship between two Spans.
///
/// The relationship can be within the same trace or across different traces.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct Link {
    /// The span context of the linked span.
    pub span_context: SpanContext,

    /// Attributes that describe this link.
    pub attributes: Vec<KeyValue>,

    /// The number of attributes that were above the configured limit, and thus
    /// dropped.
    pub dropped_attributes_count: u32,
}

impl Link {
    /// Create a new link.
    pub fn new(span_context: SpanContext, attributes: Vec<KeyValue>) -> Self {
        Link {
            span_context,
            attributes,
            dropped_attributes_count: 0,
        }
    }
}
