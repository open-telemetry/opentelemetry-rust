//! # OpenTelemetry Trace Link Interface
use crate::{trace::SpanContext, KeyValue};
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

/// During the `Span` creation user MUST have the ability to record links to other `Span`s. Linked
/// `Span`s can be from the same or a different trace.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Link {
    span_context: SpanContext,
    attributes: Vec<KeyValue>,
}

impl Link {
    /// Create a new link
    pub fn new(span_context: SpanContext, attributes: Vec<KeyValue>) -> Self {
        Link {
            span_context,
            attributes,
        }
    }

    /// The span context of the linked span
    pub fn span_context(&self) -> &SpanContext {
        &self.span_context
    }

    /// Attributes of the span link
    pub fn attributes(&self) -> &Vec<KeyValue> {
        &self.attributes
    }
}
