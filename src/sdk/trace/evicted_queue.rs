//! # Evicted Queue

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// This queue maintains an ordered list of elements, and a count of
/// dropped elements. Elements are removed from the queue in a first
/// in first out fashion.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct EvictedQueue<T> {
    queue: VecDeque<T>,
    capacity: u32,
    dropped_count: u32,
}

impl<T> EvictedQueue<T> {
    /// Create a new `EvictedQueue` with a given capacity.
    pub(crate) fn new(capacity: u32) -> Self {
        EvictedQueue {
            queue: Default::default(),
            capacity,
            dropped_count: 0,
        }
    }

    /// Push a new element to the back of the queue, dropping and
    /// recording dropped count if over capacity.
    pub(crate) fn push_back(&mut self, value: T) {
        if self.queue.len() as u32 == self.capacity {
            self.queue.pop_back();
            self.dropped_count += 1;
        }
        self.queue.push_back(value);
    }

    /// Moves all the elements of other into self, leaving other empty.
    pub fn append_vec(&mut self, other: &mut Vec<T>) {
        self.extend(other.drain(..));
    }

    /// Returns `true` if the `EvictedQueue` is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Returns a front-to-back iterator.
    pub fn iter(&self) -> std::collections::vec_deque::Iter<T> {
        self.queue.iter()
    }

    /// Returns the number of elements in the `EvictedQueue`.
    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

impl<T> IntoIterator for EvictedQueue<T> {
    type Item = T;
    type IntoIter = std::collections::vec_deque::IntoIter<T>;

    /// Consumes the `EvictedQueue` into a front-to-back iterator yielding elements by
    /// value.
    fn into_iter(self) -> Self::IntoIter {
        self.queue.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a EvictedQueue<T> {
    type Item = &'a T;
    type IntoIter = std::collections::vec_deque::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.queue.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut EvictedQueue<T> {
    type Item = &'a mut T;
    type IntoIter = std::collections::vec_deque::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.queue.iter_mut()
    }
}

impl<T> Extend<T> for EvictedQueue<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        iter.into_iter().for_each(move |elt| self.push_back(elt));
    }
}
