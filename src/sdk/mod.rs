//! # OpenTelemetry SDK
//!
//! This SDK provides an opinionated reference implementation of
//! the OpenTelemetry API. The SDK implements the specifics of
//! deciding which data to collect through `Sampler`s, and
//! facilitates the delivery of telemetry data to storage systems
//! through `Exporter`s. These can be configured on `Tracer` and
//! `Meter` creation.
#[cfg(feature = "metrics")]
pub mod metrics;
#[cfg(feature = "trace")]
pub mod trace;

#[cfg(feature = "metrics")]
pub use metrics::{LabelSet, Meter};
#[cfg(feature = "trace")]
pub use trace::{
    config::Config,
    evicted_hash_map::EvictedHashMap,
    evicted_queue::EvictedQueue,
    provider::Provider,
    sampler::Sampler,
    span::Span,
    span_processor::{BatchSpanProcessor, SimpleSpanProcessor},
    tracer::Tracer,
};
