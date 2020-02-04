//! # OpenTelemetry Trace Event Interface

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// A `Span` has the ability to add events. Events have a time associated
/// with the moment when they are added to the `Span`.
#[cfg_attr(feature = "serialize", derive(Deserialize, PartialEq, Serialize))]
#[derive(Clone, Debug)]
pub struct Event {
    /// Event message
    pub message: String,
    /// Event timestamp
    pub timestamp: SystemTime,
}
