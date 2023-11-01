//! # Span Links

use std::ops::Deref;

use opentelemetry::trace::Link;
/// Stores span links along with dropped count.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct SpanLinks {
    /// The links stored as a vector. Could be empty if there are no links.
    pub links: Vec<Link>,
    /// The number of links dropped from the span.
    pub dropped_count: u32,
}

impl Deref for SpanLinks {
    type Target = [Link];

    fn deref(&self) -> &Self::Target {
        &self.links
    }
}
