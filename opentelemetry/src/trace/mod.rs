//! API for tracing applications and libraries.
//!
//! The `trace` module includes types for tracking the progression of a single
//! request while it is handled by services that make up an application. A trace
//! is a tree of [`Span`]s which are objects that represent the work being done
//! by individual services or components involved in a request as it flows
//! through a system. This module implements the OpenTelemetry [trace
//! specification].
//!
//! [trace specification]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/api.md
//!
//! ## Getting Started
//!
//! In application code:
//!
//! ```
//! use opentelemetry::trace::{Tracer, noop::NoopTracerProvider};
//! use opentelemetry::global;
//!
//! fn init_tracer() {
//!     // Swap this no-op provider for your tracing service of choice (jaeger, zipkin, etc)
//!     let provider = NoopTracerProvider::new();
//!
//!     // Configure the global `TracerProvider` singleton when your app starts
//!     // (there is a no-op default if this is not set by your application)
//!     let _ = global::set_tracer_provider(provider);
//! }
//!
//! fn do_something_tracked() {
//!     // Then you can get a named tracer instance anywhere in your codebase.
//!     let tracer = global::tracer("my-component");
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//! }
//!
//! // in main or other app start
//! init_tracer();
//! do_something_tracked();
//! ```
//!
//! In library code:
//!
//! ```
//! use opentelemetry::{global, trace::{Span, Tracer, TracerProvider}};
//! use opentelemetry::InstrumentationScope;
//! use std::sync::Arc;
//!
//! fn my_library_function() {
//!     // Use the global tracer provider to get access to the user-specified
//!     // tracer configuration
//!     let tracer_provider = global::tracer_provider();
//!
//!     // Get a tracer for this library
//!     let scope = InstrumentationScope::builder("my_name")
//!         .with_version(env!("CARGO_PKG_VERSION"))
//!         .with_schema_url("https://opentelemetry.io/schemas/1.17.0")
//!         .build();
//!
//!     let tracer = tracer_provider.tracer_with_scope(scope);
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
//! whichever runtime you are already using such as [Tokio].
//! When using an async runtime it's best to use the batch span processor
//! where the spans will be sent in batches as opposed to being sent once ended,
//! which often ends up being more efficient.
//!
//! [Tokio]: https://tokio.rs
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
//! use opentelemetry::{global, trace::{self, Span, Status, Tracer, TracerProvider}};
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
//! ```
//!
//! ### Spans and contexts in async code
//!
//! A [`Tracer`] creates spans, a [`Span`] records an operation, and a [`Context`]
//! carries the span and other values, such as baggage, to the code that needs
//! them. Creating a span and making it current are separate steps:
//!
//! * [`Tracer::start`] creates a span using the current context as its parent.
//!   It does not make the new span current.
//! * [`TraceContextExt::with_span`] returns a new context containing a span.
//!   [`Context::current_with_span`] does the same using the current context.
//!   Neither changes the current context.
//! * [`FutureExt::with_context`] makes a context current while a future is
//!   being polled. Code running inside that future can use [`Context::current`]
//!   and [`Tracer::start`] to access the context and create child spans.
//!
//! Use a span directly to record an operation's events and attributes. Use a
//! context to pass that span to other code, either explicitly as a function
//! argument or by making the context current while the code runs.
//!
//! The current context is stored per thread. An async task can yield at an
//! `.await`, allowing another task to run on that thread, and may later resume
//! on a different thread. Do not keep a [`Context::attach`] guard or a
//! [`mark_span_as_active`] guard across an `.await`. The context could remain
//! current while unrelated code runs. Instead, wrap the future with
//! `.with_context(cx)`. Each time the runtime polls the future to make progress,
//! the wrapper makes `cx` current. It restores the previous context when that
//! poll returns, including when the future yields.
//!
//! #### Creating child spans
//!
//! This example keeps a request span current while creating a child span for
//! loading data. Both spans cover the work performed by their futures.
//! Configure an SDK tracer provider in your application to record and export
//! the spans.
//!
//! ```
//! use opentelemetry::{
//!     global,
//!     trace::{FutureExt, TraceContextExt, Tracer},
//!     Context,
//! };
//!
//! async fn load_data() {
//!     // Load data here.
//! }
//!
//! async fn handle_request() {
//!     let tracer = global::tracer("my-component");
//!     let request_cx = Context::current_with_span(tracer.start("request"));
//!
//!     async {
//!         // The request span is current when this code runs.
//!         let span = tracer.start("load_data");
//!         let child_cx = Context::current_with_span(span);
//!         load_data().with_context(child_cx.clone()).await;
//!         child_cx.span().end();
//!     }
//!     .with_context(request_cx.clone())
//!     .await;
//!
//!     request_cx.span().end();
//! }
//! ```
//!
//! The parent is chosen when the span is created. Wrapping a future in a
//! context later does not change the parent of a span that already exists.
//! If a function accepts a parent context explicitly, use
//! [`Tracer::start_with_context`] to create a child of that context. Wrap its
//! work in a context containing the child span if calls within that work need
//! to find the child through [`Context::current`].
//!
//! #### Span lifetime
//!
//! `.with_context` controls which context is current; it does not call
//! [`Span::end`] when the future finishes. In the Rust SDK, a span also ends
//! when it is dropped. Moving a span into a context transfers ownership, and
//! cloned contexts share that span. The span is dropped when the last context
//! holding it is dropped. Keeping a context clone for later use can therefore
//! keep its span open longer than the operation it represents. The example
//! above calls `.span().end()` explicitly after each operation completes.
//!
//! [`Tracer::in_span`] makes a span current for its synchronous closure. An
//! `async` block returned by that closure runs later, when the future is
//! polled, after the context guard has been dropped. If no context keeps the
//! span alive, the SDK ends it before the async work begins. Keeping a context
//! clone alive does not make it current inside the future; wrap the future
//! with `.with_context` to do that.
//!
//! #### Spawning tasks
//!
//! A newly spawned task does not automatically inherit the current
//! OpenTelemetry context. Capture the context at the spawn site and wrap the
//! task's future before passing it to the runtime. `.with_current_context()`
//! is shorthand for `.with_context(Context::current())`; it captures the
//! context when the wrapper is created, not when the task first runs.
//!
//! ```no_run
//! use opentelemetry::{
//!     global,
//!     trace::{FutureExt, TraceContextExt, Tracer},
//!     Context,
//! };
//! async fn work() {
//!     // Do work here.
//! }
//!
//! // Await this function while the request context is current.
//! async fn spawn_work() -> Result<(), tokio::task::JoinError> {
//!     let task = tokio::spawn(
//!         async {
//!             let tracer = global::tracer("worker");
//!             let cx = Context::current_with_span(tracer.start("worker"));
//!             work().with_context(cx.clone()).await;
//!             cx.span().end();
//!         }
//!         .with_current_context(),
//!     );
//!
//!     task.await
//! }
//! ```
//!
//! Capture and wrap the context for each spawned task, including when using
//! a different async runtime. If the parent operation should include the
//! tasks' work, wait for those tasks before ending the parent span. Ending a
//! parent span does not wait for or end its children.
//!
//! [`Context::current_with_span`]: TraceContextExt::current_with_span
//! [`Context::current`]: crate::Context::current
//! [`Context::attach`]: crate::Context::attach

use std::borrow::Cow;
use std::time;

pub(crate) mod context;
pub mod noop;
mod span;
mod span_context;
mod tracer;
mod tracer_provider;

pub use self::{
    context::{
        get_active_span, mark_span_as_active, FutureExt, SpanRef, TraceContextExt, WithContext,
    },
    span::{Span, SpanKind, Status},
    span_context::{SpanContext, TraceState},
    tracer::{SpanBuilder, Tracer},
    tracer_provider::TracerProvider,
};
use crate::KeyValue;
pub use crate::{SpanId, TraceFlags, TraceId};

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
    /// Create new `Link`
    pub fn new(
        span_context: SpanContext,
        attributes: Vec<KeyValue>,
        dropped_attributes_count: u32,
    ) -> Self {
        Link {
            span_context,
            attributes,
            dropped_attributes_count,
        }
    }

    /// Create new `Link` with given context
    pub fn with_context(span_context: SpanContext) -> Self {
        Link {
            span_context,
            attributes: Vec::new(),
            dropped_attributes_count: 0,
        }
    }
}
