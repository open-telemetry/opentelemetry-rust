use crate::{trace, trace::TracerProvider, Context, KeyValue};
use std::fmt;
use std::mem;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

/// Wraps the [`BoxedTracer`]'s [`Span`] so it can be used generically by
/// applications without knowing the underlying type.
///
/// [`BoxedTracer`]: struct.BoxedTracer.html
/// [`Span`]: ../api/trace/span/trait.Span.html
#[derive(Debug)]
pub struct BoxedSpan(Box<DynSpan>);

type DynSpan = dyn trace::Span + Send + Sync;

impl trace::Span for BoxedSpan {
    /// Records events at a specific time in the context of a given `Span`.
    ///
    /// Note that the OpenTelemetry project documents certain ["standard event names and
    /// keys"](https://github.com/open-telemetry/opentelemetry-specification/tree/v0.5.0/specification/trace/semantic_conventions/README.md)
    /// which have prescribed semantic meanings.
    fn add_event_with_timestamp(
        &self,
        name: String,
        timestamp: SystemTime,
        attributes: Vec<KeyValue>,
    ) {
        self.0.add_event_with_timestamp(name, timestamp, attributes)
    }

    /// Returns the `SpanContext` for the given `Span`.
    fn span_context(&self) -> trace::SpanContext {
        self.0.span_context()
    }

    /// Returns true if this `Span` is recording information like events with the `add_event`
    /// operation, attributes using `set_attributes`, status with `set_status`, etc.
    fn is_recording(&self) -> bool {
        self.0.is_recording()
    }

    /// Sets a single `Attribute` where the attribute properties are passed as arguments.
    ///
    /// Note that the OpenTelemetry project documents certain ["standard
    /// attributes"](https://github.com/open-telemetry/opentelemetry-specification/tree/v0.5.0/specification/trace/semantic_conventions/README.md)
    /// that have prescribed semantic meanings.
    fn set_attribute(&self, attribute: KeyValue) {
        self.0.set_attribute(attribute)
    }

    /// Sets the status of the `Span`. If used, this will override the default `Span`
    /// status, which is `OK`.
    fn set_status(&self, code: trace::StatusCode, message: String) {
        self.0.set_status(code, message)
    }

    /// Updates the `Span`'s name.
    fn update_name(&self, new_name: String) {
        self.0.update_name(new_name)
    }

    /// Finishes the span with given timestamp.
    fn end_with_timestamp(&self, timestamp: Option<SystemTime>) {
        self.0.end_with_timestamp(timestamp);
    }
}

/// Wraps the [`GlobalProvider`]'s [`Tracer`] so it can be used generically by
/// applications without knowing the underlying type.
///
/// [`GlobalProvider`]: struct.GlobalProvider.html
/// [`Tracer`]: ../api/trace/tracer/trait.Tracer.html
#[derive(Debug)]
pub struct BoxedTracer(Box<dyn GenericTracer + Send + Sync>);

impl trace::Tracer for BoxedTracer {
    /// Global tracer uses `BoxedSpan`s so that it can be a global singleton,
    /// which is not possible if it takes generic type parameters.
    type Span = BoxedSpan;

    /// Returns a span with an inactive `SpanContext`. Used by functions that
    /// need to return a default span like `get_active_span` if no span is present.
    fn invalid(&self) -> Self::Span {
        BoxedSpan(self.0.invalid_boxed())
    }

    /// Starts a new `Span`.
    ///
    /// Each span has zero or one parent spans and zero or more child spans, which
    /// represent causally related operations. A tree of related spans comprises a
    /// trace. A span is said to be a _root span_ if it does not have a parent. Each
    /// trace includes a single root span, which is the shared ancestor of all other
    /// spans in the trace.
    fn start_from_context(&self, name: &str, cx: &Context) -> Self::Span {
        BoxedSpan(self.0.start_with_context_boxed(name, cx))
    }

    /// Creates a span builder
    ///
    /// An ergonomic way for attributes to be configured before the `Span` is started.
    fn span_builder(&self, name: &str) -> trace::SpanBuilder {
        trace::SpanBuilder::from_name(name.to_string())
    }

    /// Create a span from a `SpanBuilder`
    fn build_with_context(&self, builder: trace::SpanBuilder, cx: &Context) -> Self::Span {
        BoxedSpan(self.0.build_with_context_boxed(builder, cx))
    }
}

/// Allows a specific [`Tracer`] to be used generically by [`BoxedTracer`]
/// instances by mirroring the interface and boxing the return types.
///
/// [`Tracer`]: ../api/trace/tracer/trait.Tracer.html
/// [`BoxedTracer`]: struct.BoxedTracer.html
pub trait GenericTracer: fmt::Debug + 'static {
    /// Create a new invalid span for use in cases where there are no active spans.
    fn invalid_boxed(&self) -> Box<DynSpan>;

    /// Returns a trait object so the underlying implementation can be swapped
    /// out at runtime.
    fn start_with_context_boxed(&self, name: &str, cx: &Context) -> Box<DynSpan>;

    /// Returns a trait object so the underlying implementation can be swapped
    /// out at runtime.
    fn build_with_context_boxed(&self, builder: trace::SpanBuilder, cx: &Context) -> Box<DynSpan>;
}

impl<S, T> GenericTracer for T
where
    S: trace::Span + Send + Sync,
    T: trace::Tracer<Span = S>,
{
    /// Create a new invalid span for use in cases where there are no active spans.
    fn invalid_boxed(&self) -> Box<DynSpan> {
        Box::new(self.invalid())
    }

    /// Returns a trait object so the underlying implementation can be swapped
    /// out at runtime.
    fn start_with_context_boxed(&self, name: &str, cx: &Context) -> Box<DynSpan> {
        Box::new(self.start_from_context(name, cx))
    }

    /// Returns a trait object so the underlying implementation can be swapped
    /// out at runtime.
    fn build_with_context_boxed(&self, builder: trace::SpanBuilder, cx: &Context) -> Box<DynSpan> {
        Box::new(self.build_with_context(builder, cx))
    }
}

/// Allows a specific [`TracerProvider`] to be used generically by the
/// [`GlobalProvider`] by mirroring the interface and boxing the return types.
///
/// [`TracerProvider`]: ../api/trace/provider/trait.TracerProvider.html
/// [`GlobalProvider`]: struct.GlobalProvider.html
pub trait GenericProvider: fmt::Debug + 'static {
    /// Creates a named tracer instance that is a trait object through the underlying `TracerProvider`.
    fn get_tracer_boxed(
        &self,
        name: &'static str,
        version: Option<&'static str>,
    ) -> Box<dyn GenericTracer + Send + Sync>;
}

impl<S, T, P> GenericProvider for P
where
    S: trace::Span + Send + Sync,
    T: trace::Tracer<Span = S> + Send + Sync,
    P: trace::TracerProvider<Tracer = T>,
{
    /// Return a boxed generic tracer
    fn get_tracer_boxed(
        &self,
        name: &'static str,
        version: Option<&'static str>,
    ) -> Box<dyn GenericTracer + Send + Sync> {
        Box::new(self.get_tracer(name, version))
    }
}

/// Represents the globally configured [`TracerProvider`] instance for this
/// application. This allows generic tracing through the returned
/// [`BoxedTracer`] instances.
///
/// [`TracerProvider`]: ../api/trace/provider/trait.TracerProvider.html
/// [`BoxedTracer`]: struct.BoxedTracer.html
#[derive(Clone, Debug)]
pub struct GlobalTracerProvider {
    provider: Arc<dyn GenericProvider + Send + Sync>,
}

impl GlobalTracerProvider {
    /// Create a new GlobalProvider instance from a struct that implements `TracerProvider`.
    fn new<P, T, S>(provider: P) -> Self
    where
        S: trace::Span + Send + Sync,
        T: trace::Tracer<Span = S> + Send + Sync,
        P: trace::TracerProvider<Tracer = T> + Send + Sync,
    {
        GlobalTracerProvider {
            provider: Arc::new(provider),
        }
    }
}

impl trace::TracerProvider for GlobalTracerProvider {
    type Tracer = BoxedTracer;

    /// Find or create a named tracer using the global provider.
    fn get_tracer(&self, name: &'static str, version: Option<&'static str>) -> Self::Tracer {
        BoxedTracer(self.provider.get_tracer_boxed(name, version))
    }
}

lazy_static::lazy_static! {
    /// The global `Tracer` provider singleton.
    static ref GLOBAL_TRACER_PROVIDER: RwLock<GlobalTracerProvider> = RwLock::new(GlobalTracerProvider::new(trace::NoopTracerProvider::new()));
}

/// Returns an instance of the currently configured global [`TracerProvider`] through
/// [`GlobalProvider`].
///
/// [`TracerProvider`]: ../api/trace/provider/trait.TracerProvider.html
/// [`GlobalProvider`]: struct.GlobalProvider.html
pub fn tracer_provider() -> GlobalTracerProvider {
    GLOBAL_TRACER_PROVIDER
        .read()
        .expect("GLOBAL_TRACER_PROVIDER RwLock poisoned")
        .clone()
}

/// Creates a named instance of [`Tracer`] via the configured [`GlobalProvider`].
///
/// If the name is an empty string, the provider will use a default name.
///
/// This is a more convenient way of expressing `global::tracer_provider().get_tracer(name, None)`.
///
/// [`Tracer`]: ../api/trace/tracer/trait.Tracer.html
/// [`GlobalProvider`]: struct.GlobalProvider.html
pub fn tracer(name: &'static str) -> BoxedTracer {
    tracer_provider().get_tracer(name, None)
}

/// Creates a named instance of [`Tracer`] with version info via the configured [`GlobalProvider`]
///
/// If the name is an empty string, the provider will use a default name.
/// If the version is an empty string, it will be used as part of instrumentation library information.
///
/// [`Tracer`]: ../api/trace/tracer/trait.Tracer.html
/// [`GlobalProvider`]: struct.GlobalProvider.html
pub fn tracer_with_version(name: &'static str, version: &'static str) -> BoxedTracer {
    tracer_provider().get_tracer(name, Some(version))
}

/// Restores the previous tracer provider on drop.
///
/// This is commonly used to uninstall pipelines. As you can only have one active tracer provider,
/// the previous provider is usually the default no-op provider.
#[derive(Debug)]
pub struct TracerProviderGuard(Option<GlobalTracerProvider>);

impl Drop for TracerProviderGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.0.take() {
            let mut global_provider = GLOBAL_TRACER_PROVIDER
                .write()
                .expect("GLOBAL_TRACER_PROVIDER RwLock poisoned");
            *global_provider = previous;
        }
    }
}

/// Sets the given [`TracerProvider`] instance as the current global provider.
///
/// [`TracerProvider`]: ../api/trace/provider/trait.TracerProvider.html
#[must_use]
pub fn set_tracer_provider<P, T, S>(new_provider: P) -> TracerProviderGuard
where
    S: trace::Span + Send + Sync,
    T: trace::Tracer<Span = S> + Send + Sync,
    P: trace::TracerProvider<Tracer = T> + Send + Sync,
{
    let mut tracer_provider = GLOBAL_TRACER_PROVIDER
        .write()
        .expect("GLOBAL_TRACER_PROVIDER RwLock poisoned");
    let previous = mem::replace(
        &mut *tracer_provider,
        GlobalTracerProvider::new(new_provider),
    );
    TracerProviderGuard(Some(previous))
}
