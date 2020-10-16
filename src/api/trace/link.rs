//! # OpenTelemetry Trace Link Interface
use crate::api;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

/// During the `Span` creation user MUST have the ability to record links to other `Span`s. Linked
/// `Span`s can be from the same or a different trace.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Link {
    span_reference: api::trace::SpanReference,
    attributes: Vec<api::KeyValue>,
}

impl Link {
    /// Create a new link
    pub fn new(span_reference: api::trace::SpanReference, attributes: Vec<api::KeyValue>) -> Self {
        Link {
            span_reference,
            attributes,
        }
    }

    /// The span context of the linked span
    pub fn span_reference(&self) -> &api::trace::SpanReference {
        &self.span_reference
    }

    /// Attributes of the span link
    pub fn attributes(&self) -> &Vec<api::KeyValue> {
        &self.attributes
    }
}
