//! # OpenTelemetry Trace Link Interface
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

/// During the `Span` creation user MUST have the ability to record links to other `Span`s. Linked
/// `Span`s can be from the same or a different trace.
#[cfg_attr(feature = "serialize", derive(Deserialize, PartialEq, Serialize))]
#[derive(Clone, Debug)]
pub struct Link {}
