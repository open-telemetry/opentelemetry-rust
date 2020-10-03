//! # OpenTelemetry Contrib
//!
//! This is a library for extensions that are not part of the core API, but still may be useful for
//! some users.
//!
//! Typically, those include vendor specific propagators.

mod id_generator;
mod trace_propagator;

#[cfg(feature = "datadog")]
pub mod datadog;

pub use id_generator::aws_xray_id_generator::XrayIdGenerator;

pub use trace_propagator::{
    aws_xray_propagator::XrayTraceContextPropagator,
    b3_propagator::{B3Encoding, B3Propagator},
    jaeger_propagator::JaegerPropagator,
};
