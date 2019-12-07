//! # OpenTelemetry Trace Event Interface

use std::time::SystemTime;

/// A `Span` has the ability to add events. Events have a time associated
/// with the moment when they are added to the `Span`.
#[derive(Clone, Debug)]
pub struct Event {
    /// Event message
    pub message: String,
    /// Event timestamp
    pub timestamp: SystemTime,
}
