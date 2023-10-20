//! # Span Links

use opentelemetry::trace::Link;
/// Stores span links along with dropped count.
#[derive(Clone, Debug, PartialEq)]
pub struct SpanLinks {
    /// The links stored as a vector. Could be empty is there are no links.
    pub links: Vec<Link>,
    /// The number of links dropped from the span.
    pub dropped_count: u32,
}

impl Default for SpanLinks {
    fn default() -> Self {
        SpanLinks {
            links: Vec::new(),
            dropped_count: 0,
        }
    }
}
