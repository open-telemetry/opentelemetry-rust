//! # OpenTelemetry Trace Event Interface

use crate::api;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// A `Span` has the ability to add events. Events have a time associated
/// with the moment when they are added to the `Span`.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    /// Event name
    pub name: String,
    /// Event timestamp
    pub timestamp: SystemTime,
    /// Event attributes
    pub attributes: Vec<api::KeyValue>,
}

impl Event {
    /// Create new `Event`
    pub fn new(name: String, timestamp: SystemTime, attributes: Vec<api::KeyValue>) -> Self {
        Event {
            name,
            timestamp,
            attributes,
        }
    }

    /// Create new `Event` with a given name.
    pub fn with_name(name: String) -> Self {
        Event {
            name,
            timestamp: SystemTime::now(),
            attributes: Vec::new(),
        }
    }
}
