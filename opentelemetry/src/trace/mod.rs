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
//! In application code:
//!
//! ```no_run
//! # #[cfg(feature = "trace")]
//! # {
//! use opentelemetry::{global, sdk::export::trace::stdout, trace::Tracer};
//!
//! fn main() {
//!     // Create a new trace pipeline that prints to stdout
//!     let tracer = stdout::new_pipeline().install_simple();
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     // Shutdown trace pipeline
//!     global::shutdown_tracer_provider();
//! }
//! # }
//! ```
//!
//! In library code:
//!
//! ```
//! # #[cfg(feature = "trace")]
//! # {
//! use opentelemetry::{global, trace::{Span, Tracer, TracerProvider}};
//!
//! fn my_library_function() {
//!     // Use the global tracer provider to get access to the user-specified
//!     // tracer configuration
//!     let tracer_provider = global::tracer_provider();
//!
//!     // Get a tracer for this library
//!     let tracer = tracer_provider.get_tracer("my_name", Some(env!("CARGO_PKG_VERSION")));
//!
//!     // Create spans
//!     let mut span = tracer.start("doing_work");
//!
//!     // Do work...
//!
//!     // End the span
//!     span.end();
//! }
//! # }
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
//! When using an async runtime it's best to use the [`BatchSpanProcessor`]
//! where the spans will be sent in batches as opposed to being sent once ended,
//! which often ends up being more efficient.
//!
//! [`BatchSpanProcessor`]: crate::sdk::trace::BatchSpanProcessor
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
//! # #[cfg(feature = "trace")]
//! # {
//! use opentelemetry::{global, trace::{self, Span, StatusCode, Tracer, TracerProvider}};
//!
//! fn may_error(rand: f32) {
//!     if rand < 0.5 {
//!         // Get the currently active span to record additional attributes,
//!         // status, etc.
//!         trace::get_active_span(|span| {
//!             span.set_status(StatusCode::Error, "value too small".into());
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
//! # }
//! ```
//!
//! Additionally [`Tracer::with_span`] and [`Tracer::in_span`] can be used as shorthand to
//! simplify managing the parent context.
//!
//! ```
//! # #[cfg(feature = "trace")]
//! # {
//! use opentelemetry::{global, trace::Tracer};
//!
//! // Get a tracer
//! let tracer = global::tracer("my_tracer");
//!
//! // Use `in_span` to create a new span and mark it as the parent, dropping it
//! // at the end of the block.
//! tracer.in_span("parent_span", |cx| {
//!     // spans created here will be children of `parent_span`
//! });
//!
//! // Use `with_span` to mark a span as active for a given period.
//! let span = tracer.start("parent_span");
//! tracer.with_span(span, |cx| {
//!     // spans created here will be children of `parent_span`
//! });
//! # }
//! ```
//!
//! #### Async active spans
//!
//! Async spans can be propagated with [`TraceContextExt`] and [`FutureExt`].
//!
//! ```
//! # #[cfg(feature = "trace")]
//! # {
//! use opentelemetry::{Context, global, trace::{FutureExt, TraceContextExt, Tracer}};
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
//! # }
//! ```

use ::futures::channel::{mpsc::TrySendError, oneshot::Canceled};
use thiserror::Error;

mod context;
mod event;
mod futures;
mod id_generator;
mod link;
mod noop;
mod provider;
mod span;
mod span_context;
mod tracer;

pub use self::{
    context::{get_active_span, mark_span_as_active, TraceContextExt},
    event::Event,
    futures::FutureExt,
    id_generator::IdGenerator,
    link::Link,
    noop::{NoopSpan, NoopSpanExporter, NoopTracer, NoopTracerProvider},
    provider::TracerProvider,
    span::{Span, SpanKind, StatusCode},
    span_context::{SpanContext, SpanId, TraceFlags, TraceId, TraceState, TraceStateError},
    tracer::{SpanBuilder, Tracer},
};
use crate::sdk::export::ExportError;
use std::time;

/// Describe the result of operations in tracing API.
pub type TraceResult<T> = Result<T, TraceError>;

/// Errors returned by the trace API.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum TraceError {
    /// Export failed with the error returned by the exporter
    #[error("Exporter {} failed with {0}", .0.exporter_name())]
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
