//! # OpenTelemetry Futures Compatibility
//!
//! This module provides utilities for instrumenting asynchronous code written
//! using [`futures`] and async/await.
//!
//! This main trait is [`Instrument`], which allows a [`Tracer`], and a [`Span`]
//! to be attached to a future, sink, stream, or executor.
//!
//! [`futures`]: https://doc.rust-lang.org/std/future/trait.Future.html
//! [`Instrument`]: trait.Instrument.html
//! [`Tracer`]: ../tracer/trait.Tracer.html
//! [`Span`]: ../span/trait.Span.html

use crate::api;
use pin_project::pin_project;
use std::{pin::Pin, task::Context};

/// A future, stream, sink, or executor that has been instrumented with a tracer and span.
#[pin_project]
#[derive(Debug, Clone)]
pub struct Instrumented<F, S: api::Span> {
    #[pin]
    inner: F,
    span: S,
}

impl<F: Sized> Instrument for F {}

impl<F: std::future::Future, S: api::Span> std::future::Future for Instrumented<F, S> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> std::task::Poll<Self::Output> {
        let this = self.project();
        this.span.mark_as_active();
        let res = this.inner.poll(cx);
        this.span.mark_as_inactive();
        res
    }
}

/// Extension trait allowing futures, streams, sinks, and executors to be traced with a span.
pub trait Instrument: Sized {
    /// Traces this type with the provided `Span`, returning a `Instrumented` wrapper.
    fn instrument<S: api::Span>(self, span: S) -> Instrumented<Self, S> {
        Instrumented { inner: self, span }
    }

    /// Traces this type with the provided `Tracer`'s active span, returning a `Instrumented` wrapper.
    fn in_active_span<T: api::Tracer>(self, tracer: T) -> Instrumented<Self, T::Span> {
        let span = tracer.get_active_span();
        self.instrument(span)
    }
}
