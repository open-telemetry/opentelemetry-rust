//! # OpenTelemetry Trace Link Interface

/// During the `Span` creation user MUST have the ability to record links to other `Span`s. Linked
/// `Span`s can be from the same or a different trace.
#[derive(Clone, Debug)]
pub struct Link {}
