//! # Opentelemetry propagator contrib
//!
//! This module provides propagators for third party vendor format or experimental propagators that
//! aren't part of Opentelemetry.
//!
//! Currently, the following propagators are supported:
//!
//! * `binary_propagator`, propagating trace context in the binary format.
//!
//! This module also provides relative types for those propagators.
pub mod binary;
pub mod trace_context_response;
