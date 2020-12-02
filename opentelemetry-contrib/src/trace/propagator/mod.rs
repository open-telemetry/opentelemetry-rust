//! # Opentelemetry propagator contrib
//!
//! This module provides propagators for third party vendor format or experimental propagators that
//! aren't part of Opentelemetry.
//!
//! Currently, the following propagators are supported:
//!
//! * `binary_propagator`, propagating trace context in the binary format.
//! * `XrayPropagator`, propagating via AWS XRay protocol.
//! * `B3Propagator`, propagating via B3 protocol and headers.
//! * `JaegerPropagator`, propagating via Jaeger protocol and headers.
//!
//! This module also provides relative types for those propagators.
#[cfg(feature = "aws-xray")]
mod aws;
#[cfg(feature = "zipkin")]
mod b3;
pub mod binary;
#[cfg(feature = "jaeger")]
mod jaeger;

#[cfg(feature = "aws-xray")]
pub use aws::XrayPropagator;
#[cfg(feature = "zipkin")]
pub use b3::{B3Encoding, B3Propagator};
#[cfg(feature = "jaeger")]
pub use jaeger::JaegerPropagator;
