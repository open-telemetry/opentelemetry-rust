//! # Span Queue

use opentelemetry::sdk::export::trace::SpanData;
use opentelemetry::trace::SpanContext;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// This queue maintains an ordered list of elements, Elements are
/// removed from the queue in a first in first out fashion.
#[derive(Clone, Debug)]
pub(crate) struct SpanQueue {
    // We can't really use the opentelemetry::EvictedQueue here because
    // we need to compare the SpanData based on their span context
    // rather than all fields. Thus, we cannot use SpanData's default
    // equal function as it compares all fields.

    // All operation within SpanQueue should be O(1)
    queue: Vec<SpanData>,
    map: HashMap<SpanContext, usize>,
    next_idx: usize,
}

impl PartialEq for SpanQueue {
    fn eq(&self, other: &Self) -> bool {
        self.queue.eq(&other.queue) && self.next_idx == other.next_idx
    }
}

impl SpanQueue {
    /// Create a new `SpanQueue` with a given max length.
    pub(crate) fn new(max_len: usize) -> Self {
        SpanQueue {
            queue: Vec::with_capacity(max_len),
            next_idx: 0,
            map: HashMap::with_capacity(max_len),
        }
    }

    /// Push a new element to the back of the queue
    pub(crate) fn push_back(&mut self, value: SpanData) {
        self.next_idx = self.next_idx % self.queue.capacity();
        self.map.insert(value.span_context.clone(), self.next_idx);
        match self.queue.get_mut(self.next_idx) {
            Some(ele) => {
                self.map.remove(&ele.span_context);
                *ele = value;
            }
            None => {
                self.queue.push(value);
            }
        }
        self.next_idx += 1;
    }

    /// Returns the number of elements in the `SpanQueue`.
    pub(crate) fn len(&self) -> usize {
        self.queue.len()
    }

    /// Remove one element if exist.
    pub(crate) fn remove(&mut self, span_context: SpanContext) -> Option<SpanData> {
        if !self.map.contains_key(&span_context) {
            None
        } else {
            self.next_idx = self.queue.len() - 1;
            let idx = *(self.map.get(&span_context).unwrap());
            if idx == self.queue.len() - 1 {
                // if it's last element, just remove
                self.map.remove(&span_context);
                Some(self.queue.remove(idx))
            } else {
                let last_span_context = self.queue.last().unwrap().span_context.clone();
                self.map.remove(&span_context);
                self.map.insert(last_span_context, idx);
                Some(self.queue.swap_remove(idx))
            }
        }
    }

    /// Return all spans it currently hold
    pub(crate) fn spans(self) -> Vec<SpanData> {
        self.queue.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, NaiveDateTime, Utc};
    use opentelemetry::testing::trace::new_test_export_span_data;
    use opentelemetry::trace::{TraceFlags, TraceState, TraceId, SpanId};

    enum Action {
        PushBack(u128, u64),
        Remove(u128, u64),
    }

    // If the expected is None, means we skip this check in this test plan.
    #[derive(Default)]
    struct TestPlan {
        max_len: usize,
        actions: Vec<Action>,
        expected_next_idx: Option<usize>,
        expected_queue: Option<Vec<(u128, u64)>>,
        expected_len: Option<usize>,
    }

    #[test]
    fn test_span_queue() {
        let get_span_context = |trace_id: u128, span_id: u64| {
            SpanContext::new(
                TraceId::from_u128(trace_id),
                SpanId::from_u64(span_id),
                TraceFlags::new(0),
                false,
                TraceState::default(),
            )
        };
        let get_span_data = |trace_id: u128, span_id: u64| {
            let mut span_data = new_test_export_span_data();
            span_data.span_context = get_span_context(trace_id, span_id);
            let time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc);
            span_data.start_time = time.clone().into();
            span_data.end_time = time.into();
            span_data
        };
        let plans = vec![
            TestPlan {
                max_len: 3,
                actions: vec![
                    Action::PushBack(1, 1),
                    Action::PushBack(1, 2),
                    Action::PushBack(1, 3),
                    Action::PushBack(1, 4),
                ],
                expected_next_idx: Some(1),
                expected_len: Some(3),
                expected_queue: Some(vec![(1, 4), (1, 2), (1, 3)]),
            },
            TestPlan {
                max_len: 3,
                actions: vec![
                    Action::PushBack(1, 3),
                    Action::PushBack(2, 2),
                    Action::PushBack(1, 4),
                    Action::PushBack(1, 5),
                    Action::Remove(1, 3),
                    Action::Remove(1, 4),
                ],
                expected_queue: Some(vec![(1, 5), (2, 2)]),
                expected_next_idx: Some(2),
                expected_len: Some(2),
            },
            TestPlan {
                max_len: 3,
                actions: vec![
                    Action::PushBack(1, 1),
                    Action::Remove(1, 3),
                    Action::Remove(1, 4),
                    Action::PushBack(1, 3),
                    Action::Remove(1, 1),
                    Action::Remove(1, 3),
                ],
                expected_len: Some(0),
                expected_next_idx: Some(0),
                expected_queue: Some(vec![]),
            },
        ];

        for plan in plans {
            let mut span_queue = SpanQueue::new(plan.max_len);
            for action in plan.actions {
                match action {
                    Action::PushBack(trace_id, span_id) => {
                        span_queue.push_back(get_span_data(trace_id, span_id));
                    }
                    Action::Remove(trace_id, span_id) => {
                        span_queue.remove(get_span_context(trace_id, span_id));
                    }
                }
            }
            if let Some(next_id) = plan.expected_next_idx {
                assert_eq!(span_queue.next_idx, next_id);
            }
            if let Some(len) = plan.expected_len {
                assert_eq!(span_queue.len(), len);
            }
            if let Some(queue) = plan.expected_queue {
                assert_eq!(
                    span_queue.queue,
                    queue
                        .iter()
                        .cloned()
                        .map(|(trace_id, span_id)| get_span_data(trace_id, span_id))
                        .collect::<Vec<SpanData>>()
                );
                assert_eq!(span_queue.map.len(), queue.len());
                for (idx, (trace_id, span_id)) in queue.into_iter().enumerate() {
                    let span_context = get_span_context(trace_id, span_id);
                    assert_eq!(span_queue.map.get(&span_context).map(|idx| *idx), Some(idx));
                }
            }
        }
    }
}
