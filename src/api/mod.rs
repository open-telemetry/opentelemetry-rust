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
pub mod distributed_context;
pub mod metrics;
pub mod trace;

pub use self::core::{Key, KeyValue, Unit, Value};
pub use distributed_context::http_b3_propagator::HttpB3Propagator;
pub use metrics::{
    counter::{Counter, CounterHandle},
    gauge::{Gauge, GaugeHandle},
    measure::{Measure, MeasureHandle},
    noop::NoopMeter,
    value::MeasurementValue,
    Instrument, InstrumentHandle, LabelSet, Measurement, Meter, MetricOptions,
};
pub use trace::{
    noop::{NoopProvider, NoopSpan, NoopTracer},
    propagator::{BinaryFormat, Carrier, HttpTextFormat},
    provider::Provider,
    span::Span,
    span_context::{SpanContext, TRACE_FLAGS_UNUSED, TRACE_FLAG_SAMPLED},
    tracer::{Tracer, TracerGenerics},
};
