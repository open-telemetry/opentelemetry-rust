use crate::{api, api::TracerProvider};
use std::fmt;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

/// Wraps the [`BoxedTracer`]'s [`Span`] so it can be used generically by
/// applications without knowing the underlying type.
///
/// [`BoxedTracer`]: struct.BoxedTracer.html
/// [`Span`]: ../api/trace/span/trait.Span.html
#[derive(Debug)]
pub struct BoxedSpan(Box<DynSpan>);
type DynSpan = dyn api::Span + Send + Sync;

impl api::Span for BoxedSpan {
    /// Records events at a specific time in the context of a given `Span`.
    ///
    /// Note that the OpenTelemetry project documents certain ["standard event names and
    /// keys"](https://github.com/open-telemetry/opentelemetry-specification/tree/v0.5.0/specification/trace/semantic_conventions/README.md)
    /// which have prescribed semantic meanings.
    fn add_event_with_timestamp(
        &self,
        name: String,
        timestamp: SystemTime,
        attributes: Vec<api::KeyValue>,
    ) {
        self.0.add_event_with_timestamp(name, timestamp, attributes)
    }

    /// Returns the `SpanContext` for the given `Span`.
    fn span_context(&self) -> api::SpanContext {
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
    fn set_attribute(&self, attribute: api::KeyValue) {
        self.0.set_attribute(attribute)
    }

    /// Sets the status of the `Span`. If used, this will override the default `Span`
    /// status, which is `OK`.
    fn set_status(&self, code: api::StatusCode, message: String) {
        self.0.set_status(code, message)
    }

    /// Updates the `Span`'s name.
    fn update_name(&self, new_name: String) {
        self.0.update_name(new_name)
    }

    /// Finishes the span with given timestamp.
    fn end_with_timestamp(&self, timestamp: SystemTime) {
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

impl api::Tracer for BoxedTracer {
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
    fn start_from_context(&self, name: &str, cx: &api::Context) -> Self::Span {
        BoxedSpan(self.0.start_with_context_boxed(name, cx))
    }

    /// Creates a span builder
    ///
    /// An ergonomic way for attributes to be configured before the `Span` is started.
    fn span_builder(&self, name: &str) -> api::SpanBuilder {
        api::SpanBuilder::from_name(name.to_string())
    }

    /// Create a span from a `SpanBuilder`
    fn build_with_context(&self, builder: api::SpanBuilder, cx: &api::Context) -> Self::Span {
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
    fn start_with_context_boxed(&self, name: &str, cx: &api::Context) -> Box<DynSpan>;

    /// Returns a trait object so the underlying implementation can be swapped
    /// out at runtime.
    fn build_with_context_boxed(
        &self,
        builder: api::SpanBuilder,
        cx: &api::Context,
    ) -> Box<DynSpan>;
}

impl<S, T> GenericTracer for T
where
    S: api::Span + Send + Sync,
    T: api::Tracer<Span = S>,
{
    /// Create a new invalid span for use in cases where there are no active spans.
    fn invalid_boxed(&self) -> Box<DynSpan> {
        Box::new(self.invalid())
    }

    /// Returns a trait object so the underlying implementation can be swapped
    /// out at runtime.
    fn start_with_context_boxed(&self, name: &str, cx: &api::Context) -> Box<DynSpan> {
        Box::new(self.start_from_context(name, cx))
    }

    /// Returns a trait object so the underlying implementation can be swapped
    /// out at runtime.
    fn build_with_context_boxed(
        &self,
        builder: api::SpanBuilder,
        cx: &api::Context,
    ) -> Box<DynSpan> {
        Box::new(self.build_with_context(builder, cx))
    }
}

/// Allows a specific [`Provider`] to be used generically by the
/// [`GlobalProvider`] by mirroring the interface and boxing the return types.
///
/// [`Provider`]: ../api/trace/provider/trait.Provider.html
/// [`GlobalProvider`]: struct.GlobalProvider.html
pub trait GenericProvider: fmt::Debug + 'static {
    /// Creates a named tracer instance that is a trait object through the underlying `Provider`.
    fn get_tracer_boxed(&self, name: &'static str) -> Box<dyn GenericTracer + Send + Sync>;
}

impl<S, T, P> GenericProvider for P
where
    S: api::Span + Send + Sync,
    T: api::Tracer<Span = S> + Send + Sync,
    P: api::TracerProvider<Tracer = T>,
{
    /// Return a boxed generic tracer
    fn get_tracer_boxed(&self, name: &'static str) -> Box<dyn GenericTracer + Send + Sync> {
        Box::new(self.get_tracer(name))
    }
}

/// Represents the globally configured [`Provider`] instance for this
/// application. This allows generic tracing through the returned
/// [`BoxedTracer`] instances.
///
/// [`Provider`]: ../api/trace/provider/trait.Provider.html
/// [`BoxedTracer`]: struct.BoxedTracer.html
#[derive(Clone, Debug)]
pub struct GlobalProvider {
    provider: Arc<dyn GenericProvider + Send + Sync>,
}

impl GlobalProvider {
    /// Create a new GlobalProvider instance from a struct that implements `Provider`.
    fn new<P, T, S>(provider: P) -> Self
    where
        S: api::Span + Send + Sync,
        T: api::Tracer<Span = S> + Send + Sync,
        P: api::TracerProvider<Tracer = T> + Send + Sync,
    {
        GlobalProvider {
            provider: Arc::new(provider),
        }
    }
}

impl api::TracerProvider for GlobalProvider {
    type Tracer = BoxedTracer;

    /// Find or create a named tracer using the global provider.
    fn get_tracer(&self, name: &'static str) -> Self::Tracer {
        BoxedTracer(self.provider.get_tracer_boxed(name))
    }
}

lazy_static::lazy_static! {
    /// The global `Tracer` provider singleton.
    static ref GLOBAL_TRACER_PROVIDER: RwLock<GlobalProvider> = RwLock::new(GlobalProvider::new(api::NoopProvider {}));
}

/// Returns an instance of the currently configured global [`Provider`] through
/// [`GlobalProvider`].
///
/// [`Provider`]: ../api/trace/provider/trait.Provider.html
/// [`GlobalProvider`]: struct.GlobalProvider.html
pub fn trace_provider() -> GlobalProvider {
    GLOBAL_TRACER_PROVIDER
        .read()
        .expect("GLOBAL_TRACER_PROVIDER RwLock poisoned")
        .clone()
}

/// Creates a named instance of [`Tracer`] via the configured [`GlobalProvider`].
///
/// If the name is an empty string, the provider will use a default name.
///
/// This is a more convenient way of expressing `global::trace_provider().get_tracer(name)`.
///
/// [`Tracer`]: ../api/trace/tracer/trait.Tracer.html
/// [`GlobalProvider`]: struct.GlobalProvider.html
pub fn tracer(name: &'static str) -> BoxedTracer {
    trace_provider().get_tracer(name)
}

/// Sets the given [`Provider`] instance as the current global provider.
///
/// [`Provider`]: ../api/trace/provider/trait.Provider.html
pub fn set_provider<P, T, S>(new_provider: P)
where
    S: api::Span + Send + Sync,
    T: api::Tracer<Span = S> + Send + Sync,
    P: api::TracerProvider<Tracer = T> + Send + Sync,
{
    let mut global_provider = GLOBAL_TRACER_PROVIDER
        .write()
        .expect("GLOBAL_TRACER_PROVIDER RwLock poisoned");
    *global_provider = GlobalProvider::new(new_provider);
}
