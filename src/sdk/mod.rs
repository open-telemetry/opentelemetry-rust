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

#[cfg(feature = "trace")]
pub use crate::exporter::trace::jaeger::AllSampler as AlwaysSample;
#[cfg(feature = "metrics")]
pub use metrics::{LabelSet, Meter};
#[cfg(feature = "trace")]
pub use trace::{config::Config, provider::Provider, span::Span, tracer::Tracer};
