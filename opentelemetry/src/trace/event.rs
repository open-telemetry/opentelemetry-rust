//! # OpenTelemetry Trace Event Interface

use crate::KeyValue;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::time::SystemTime;

/// A `Span` has the ability to add events. Events have a time associated
/// with the moment when they are added to the `Span`.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    /// Event name
    pub name: Cow<'static, str>,
    /// Event timestamp
    pub timestamp: SystemTime,
    /// Event attributes
    pub attributes: Vec<KeyValue>,
}

impl Event {
    /// Create new `Event`
    pub fn new<T: Into<Cow<'static, str>>>(
        name: T,
        timestamp: SystemTime,
        attributes: Vec<KeyValue>,
    ) -> Self {
        Event {
            name: name.into(),
            timestamp,
            attributes,
        }
    }

    /// Create new `Event` with a given name.
    pub fn with_name<T: Into<Cow<'static, str>>>(name: T) -> Self {
        Event {
            name: name.into(),
            timestamp: crate::time::now(),
            attributes: Vec::new(),
        }
    }
}
