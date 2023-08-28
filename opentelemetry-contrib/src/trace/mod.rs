//! # Opentelemetry trace contrib
//!

#[cfg(feature = "api")]
mod context;
#[cfg(feature = "api")]
pub use context::{new_span_if_parent_sampled, new_span_if_recording, Contextualized};

pub mod exporter;
pub mod propagator;

#[cfg(feature = "api")]
mod tracer_source;
#[cfg(feature = "api")]
pub use tracer_source::TracerSource;
