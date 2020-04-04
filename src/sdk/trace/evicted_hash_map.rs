//! # Evicted Map

use crate::api;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};

/// A hash map with a capped number of attributes that retains the most
/// recently set entries.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct EvictedHashMap {
    map: HashMap<api::Key, api::Value>,
    evict_list: LinkedList<api::Key>,
    capacity: u32,
    dropped_count: u32,
}

impl EvictedHashMap {
    /// Create a new `EvictedHashMap` with a given capacity.
    pub fn new(capacity: u32) -> Self {
        EvictedHashMap {
            map: HashMap::new(),
            evict_list: Default::default(),
            capacity,
            dropped_count: 0,
        }
    }

    /// Inserts a key-value pair into the map.
    pub fn insert(&mut self, item: api::KeyValue) {
        // Check for existing item
        if let Some(value) = self.map.get_mut(&item.key) {
            *value = item.value;
            self.move_key_to_front(item.key);
            return;
        }

        // Add new item
        self.evict_list.push_front(item.key.clone());
        self.map.insert(item.key, item.value);

        // Verify size not exceeded
        if self.evict_list.len() as u32 > self.capacity {
            self.remove_oldest();
            self.dropped_count += 1;
        }
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns a front-to-back iterator.
    pub fn iter(&self) -> std::collections::hash_map::Iter<api::Key, api::Value> {
        self.map.iter()
    }

    fn move_key_to_front(&mut self, key: api::Key) {
        if self.evict_list.is_empty() {
            // If empty, push front
            self.evict_list.push_front(key);
        } else if self.evict_list.front() == Some(&key) {
            // Already the front, ignore
        } else {
            // Else split linked lists around key and combine
            let key_idx = self
                .evict_list
                .iter()
                .position(|k| k == &key)
                .expect("key must exist in evicted hash map, this is a bug");
            let mut tail = self.evict_list.split_off(key_idx);
            let item = tail.pop_front().unwrap();
            self.evict_list.push_front(item);
            self.evict_list.append(&mut tail);
        }
    }

    fn remove_oldest(&mut self) {
        if let Some(oldest_item) = self.evict_list.pop_back() {
            self.map.remove(&oldest_item);
        }
    }
}

impl IntoIterator for EvictedHashMap {
    type Item = (api::Key, api::Value);
    type IntoIter = std::collections::hash_map::IntoIter<api::Key, api::Value>;

    /// Creates a consuming iterator, that is, one that moves each key-value
    /// pair out of the map in arbitrary order. The map cannot be used after
    /// calling this.
    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<'a> IntoIterator for &'a EvictedHashMap {
    type Item = (&'a api::Key, &'a api::Value);
    type IntoIter = std::collections::hash_map::Iter<'a, api::Key, api::Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}

impl<'a> IntoIterator for &'a mut EvictedHashMap {
    type Item = (&'a api::Key, &'a mut api::Value);
    type IntoIter = std::collections::hash_map::IterMut<'a, api::Key, api::Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::EvictedHashMap;
    use crate::api::Key;
    use std::collections::HashSet;

    #[test]
    fn insert_over_capacity_test() {
        let capacity = 10;
        let mut map = EvictedHashMap::new(capacity);

        for i in 0..=capacity {
            map.insert(Key::new(i.to_string()).bool(true))
        }

        assert_eq!(map.dropped_count, 1);
        assert_eq!(map.len(), capacity as usize);
        assert_eq!(
            map.map.keys().cloned().collect::<HashSet<_>>(),
            (1..=capacity)
                .map(|i| Key::new(i.to_string()))
                .collect::<HashSet<_>>()
        );
    }
}
