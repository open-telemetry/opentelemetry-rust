//! # Evicted Queue

use opentelemetry::sdk::export::trace::SpanData;
use opentelemetry::trace::SpanContext;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// This queue maintains an ordered list of elements, Elements are
/// removed from the queue in a first in first out fashion.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpanQueue {
    // We can't really use the opentelemetry::EvictedQueue here because
    // we need to compare the SpanData based on their span context
    // rather than all fields. Thus, we cannot use SpanData's default
    // equal function as it compares all fields.

    // We may also try to build a more complicate structure to `remove` here
    // as current approach's time complexity is high.
    // We should add and remove all in O(1) time complexity
    pub(crate) queue: VecDeque<SpanData>,
    pub(crate) max_len: usize,
}

impl SpanQueue {
    /// Create a new `EvictedQueue` with a given max length.
    pub(crate) fn new(max_len: usize) -> Self {
        SpanQueue {
            queue: VecDeque::with_capacity(max_len),
            max_len,
        }
    }

    /// Push a new element to the back of the queue, dropping and
    /// recording dropped count if over capacity.
    pub(crate) fn push_back(&mut self, value: SpanData) {
        if self.queue.len() == self.max_len {
            self.queue.pop_front();
        }
        self.queue.push_back(value);
    }

    /// Returns the number of elements in the `EvictedQueue`.
    pub(crate) fn len(&self) -> usize {
        self.queue.len()
    }

    /// Remove one element if exist
    pub(crate) fn remove(&mut self, value: SpanContext) -> Option<SpanData> {
        let mut idx = 0;
        for ele in self.queue.iter() {
            if value == ele.span_context {
                break;
            }
            idx += 1;
        }
        self.queue.remove(idx)
    }

    /// Get an element from the queue.
    pub(crate) fn get(&self, index: usize) -> Option<&SpanData> {
        self.queue.get(index)
    }
}
