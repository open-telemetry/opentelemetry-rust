//! OpenTelemetry Propagators
mod baggage;
mod trace_context;

pub use baggage::BaggagePropagator;
use std::fmt::Display;
pub use trace_context::TraceContextPropagator;


