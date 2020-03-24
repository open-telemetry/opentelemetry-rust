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
pub mod core;
pub mod metrics;
pub mod propagation;
pub mod trace;

pub use self::core::{Key, KeyValue, Unit, Value};
pub use metrics::{
    counter::{Counter, CounterHandle},
    gauge::{Gauge, GaugeHandle},
    measure::{Measure, MeasureHandle},
    noop::NoopMeter,
    value::MeasurementValue,
    Instrument, InstrumentHandle, LabelSet, Measurement, Meter, MetricOptions,
};
#[cfg(feature = "base64_format")]
pub use propagation::base64_format::Base64Format;
pub use propagation::{binary_propagator::BinaryFormat, text_propagator::HttpTextFormat, Carrier};
pub use trace::{
    b3_propagator::B3Propagator,
    event::Event,
    link::Link,
    noop::{NoopProvider, NoopSpan, NoopTracer},
    provider::Provider,
    sampler::{Sampler, SamplingDecision, SamplingResult},
    span::{Span, SpanKind, SpanStatus},
    span_context::{SpanContext, SpanId, TraceId, TRACE_FLAGS_UNUSED, TRACE_FLAG_SAMPLED},
    span_processor::SpanProcessor,
    trace_context_propagator::TraceContextPropagator,
    tracer::{SpanBuilder, Tracer, TracerGenerics},
};
