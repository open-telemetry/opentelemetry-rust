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

impl Event {
    /// Create new `Event`
    pub fn new(message: String, timestamp: SystemTime) -> Self {
        Event { message, timestamp }
    }

    /// Create new `Event` for a given message.
    pub fn from_message(message: String) -> Self {
        Event {
            message,
            timestamp: SystemTime::now(),
        }
    }
}
