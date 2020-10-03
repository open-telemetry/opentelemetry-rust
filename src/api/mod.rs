//! ## OpenTelemetry API: What applications use and SDKs implement.
//!
//! OpenTelemetry Language Libraries are composed of 2 packages: `api` and `sdk`.
//!
//! Third-party libraries and frameworks that want to be instrumented in OpenTelemetry-compatible
//! way will have a dependency on the `api` package. The developers of these third-party libraries
//! will add calls to telemetry API to produce telemetry data.
//!
//! Applications that use third-party libraries that are instrumented with OpenTelemetry API will
//! have a choice to enable or not enable the actual delivery of telemetry data. The application
//! can also call telemetry API directly to produce additional telemetry data.
//!
//! In order to enable telemetry the application must take a dependency on the OpenTelemetry SDK,
//! which implements the delivery of the telemetry. The application must also configure exporters
//! so that the SDK knows where and how to deliver the telemetry.
#[cfg(feature = "trace")]
pub mod baggage;
pub mod context;
pub mod core;
pub mod labels;
#[cfg(feature = "metrics")]
pub mod metrics;
#[cfg(feature = "trace")]
pub mod trace;

pub use self::core::{Key, KeyValue, Unit, Value};
#[cfg(feature = "trace")]
pub use baggage::{Baggage, BaggageExt, BaggagePropagator};
#[cfg(feature = "trace")]
pub use context::propagation::{
    composite_propagator::TextMapCompositePropagator, text_propagator::FieldIter,
    text_propagator::TextMapFormat, Extractor, Injector,
};
pub use context::Context;

#[cfg(feature = "trace")]
pub use trace::{
    context::TraceContextExt,
    event::Event,
    futures::FutureExt,
    id_generator::IdGenerator,
    link::Link,
    noop::{NoopSpan, NoopSpanExporter, NoopTracer, NoopTracerProvider},
    provider::TracerProvider,
    span::{Span, SpanKind, StatusCode},
    span_context::{
        SpanContext, SpanId, TraceId, TraceState, TRACE_FLAG_DEBUG, TRACE_FLAG_DEFERRED,
        TRACE_FLAG_NOT_SAMPLED, TRACE_FLAG_SAMPLED,
    },
    span_processor::SpanProcessor,
    trace_context_propagator::TraceContextPropagator,
    tracer::{SpanBuilder, Tracer},
};
