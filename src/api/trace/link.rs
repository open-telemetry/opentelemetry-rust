//! # OpenTelemetry Trace Link Interface
use crate::api;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

/// During the `Span` creation user MUST have the ability to record links to other `Span`s. Linked
/// `Span`s can be from the same or a different trace.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Link {
    span_context: api::trace::SpanContext,
    attributes: Vec<api::KeyValue>,
}

impl Link {
    /// Create a new link
    pub fn new(span_context: api::trace::SpanContext, attributes: Vec<api::KeyValue>) -> Self {
        Link {
            span_context,
            attributes,
        }
    }

    /// The span context of the linked span
    pub fn span_context(&self) -> &api::trace::SpanContext {
        &self.span_context
    }

    /// Attributes of the span link
    pub fn attributes(&self) -> &Vec<api::KeyValue> {
        &self.attributes
    }
}
