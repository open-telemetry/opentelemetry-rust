//! # OpenTelemetry SDK
//!
//! This SDK provides an opinionated reference implementation of
//! the OpenTelemetry API. The SDK implements the specifics of
//! deciding which data to collect through `Sampler`s, and
//! facilitates the delivery of telemetry data to storage systems
//! through `Exporter`s. These can be configured on `Tracer` and
//! `Meter` creation.
pub mod export;
#[cfg(feature = "metrics")]
pub mod metrics;
pub mod resource;
#[cfg(feature = "trace")]
pub mod trace;

pub use resource::Resource;
#[cfg(feature = "trace")]
pub use trace::{
    config::Config,
    evicted_hash_map::EvictedHashMap,
    evicted_queue::EvictedQueue,
    id_generator::IdGenerator,
    provider::Provider,
    sampler::{Sampler, SamplingDecision, SamplingResult, ShouldSample},
    span::Span,
    span_processor::{BatchSpanProcessor, SimpleSpanProcessor},
    tracer::Tracer,
};
