//! # Opentelemetry propagator contrib
//!
//! This module provides propagators for third party vendor format or experimental propagators that
//! aren't part of Opentelemetry.
//!
//! Currently, the following propagators are supported:
//!
//! * `binary_propagator`, propagating trace context in the binary format.
//! * `XrayPropagator`, propagating via AWS XRay protocol.
//!
//! This module also provides relative types for those propagators.
#[cfg(feature = "aws-xray")]
mod aws;
pub mod binary;

#[cfg(feature = "aws-xray")]
pub use aws::XrayPropagator;
