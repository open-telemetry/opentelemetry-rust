//! # OpenTelemetry SDK
//!
//! This SDK provides an opinionated reference implementation of
//! the OpenTelemetry API. The SDK implements the specifics of
//! deciding which data to collect through `Sampler`s, and
//! facilitates the delivery of telemetry data to storage systems
//! through `Exporter`s. These can be configured on `Tracer` and
//! `Meter` creation.
pub mod env;
pub mod export;
pub mod instrumentation;
#[cfg(feature = "metrics")]
pub mod metrics;
pub mod resource;
#[cfg(feature = "trace")]
pub mod trace;

pub use env::EnvResourceDetector;
pub use instrumentation::InstrumentationLibrary;
pub use resource::Resource;
#[cfg(feature = "trace")]
pub use trace::{
    config::Config,
    evicted_hash_map::EvictedHashMap,
    evicted_queue::EvictedQueue,
    id_generator::IdGenerator,
    provider::{Builder, TracerProvider},
    sampler::{Sampler, SamplingDecision, SamplingResult, ShouldSample},
    span::Span,
    span_processor::{BatchSpanProcessor, SimpleSpanProcessor},
    tracer::Tracer,
};
