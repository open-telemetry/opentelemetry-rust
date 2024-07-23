use std::array::IntoIter;

/// The default initial capacity for `HybridVec`.
const DEFAULT_INITIAL_CAPACITY: usize = 10;

#[derive(Debug)]
/// A hybrid vector that starts with a fixed-size array and grows dynamically with a vector.
pub struct HybridVec<T: Default, const INITIAL_CAPACITY: usize = DEFAULT_INITIAL_CAPACITY> {
    initial: [T; INITIAL_CAPACITY],
    additional: Vec<T>,
    count: usize,
}

impl<T: Default, const INITIAL_CAPACITY: usize> HybridVec<T, INITIAL_CAPACITY> {
    /// Creates a new `HybridVec` with the default initial capacity.
    pub fn new() -> Self {
        Self {
            initial: [(); INITIAL_CAPACITY].map(|_| T::default()),
            additional: Vec::new(),
            count: 0,
        }
    }

    /// Pushes a value into the `HybridVec`.
    pub fn push(&mut self, value: T) {
        if self.count < INITIAL_CAPACITY {
            self.initial[self.count] = value;
            self.count += 1;
        } else {
            self.additional.push(value);
        }
    }

    /// Gets a reference to the value at the specified index.
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.count {
            Some(&self.initial[index])
        } else if index < self.count + self.additional.len() {
            self.additional.get(index - INITIAL_CAPACITY)
        } else {
            None
        }
    }

    /// Returns the number of elements in the `HybridVec`.
    pub fn len(&self) -> usize {
        self.count + self.additional.len()
    }
}

// Implement `IntoIterator` for `HybridVec`
impl<T: Default, const INITIAL_CAPACITY: usize> IntoIterator for HybridVec<T, INITIAL_CAPACITY> {
    type Item = T;
    type IntoIter = HybridVecIntoIter<T, INITIAL_CAPACITY>;

    fn into_iter(self) -> Self::IntoIter {
        if self.additional.is_empty() {
            HybridVecIntoIter::StackOnly {
                iter: self.initial.into_iter().take(self.count),
            }
        } else {
            HybridVecIntoIter::Mixed {
                stack_iter: self.initial.into_iter().take(self.count),
                heap_iter: self.additional.into_iter(),
            }
        }
    }
}

#[derive(Debug)]
/// Iterator for consuming a `HybridVec`.
pub enum HybridVecIntoIter<T: Default, const INITIAL_CAPACITY: usize> {
    /// stackonly
    StackOnly {
        /// iter
        iter: std::iter::Take<IntoIter<T, INITIAL_CAPACITY>>,
    },
    /// hybrid
    Mixed {
        /// stack_iter
        stack_iter: std::iter::Take<IntoIter<T, INITIAL_CAPACITY>>,
        /// heap_iter
        heap_iter: std::vec::IntoIter<T>,
    },
}

impl<T: Default, const INITIAL_CAPACITY: usize> Iterator
    for HybridVecIntoIter<T, INITIAL_CAPACITY>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            HybridVecIntoIter::StackOnly { iter } => iter.next(),
            HybridVecIntoIter::Mixed {
                stack_iter,
                heap_iter,
            } => stack_iter.next().or_else(|| heap_iter.next()),
        }
    }
}

// Implement `IntoIterator` for a reference to `HybridVec`
impl<'a, T: Default + 'a, const INITIAL_CAPACITY: usize> IntoIterator
    for &'a HybridVec<T, INITIAL_CAPACITY>
{
    type Item = &'a T;
    type IntoIter = HybridVecIter<'a, T, INITIAL_CAPACITY>;

    fn into_iter(self) -> Self::IntoIter {
        if self.additional.is_empty() {
            HybridVecIter::StackOnly {
                iter: self.initial.iter().take(self.count),
            }
        } else {
            HybridVecIter::Mixed {
                stack_iter: self.initial.iter().take(self.count),
                heap_iter: self.additional.iter(),
            }
        }
    }
}

#[derive(Debug)]
/// Iterator for referencing elements in a `HybridVec`.
pub enum HybridVecIter<'a, T: Default, const INITIAL_CAPACITY: usize> {
    /// stackonly
    StackOnly {
        /// iter
        iter: std::iter::Take<std::slice::Iter<'a, T>>,
    },
    /// hybrid
    Mixed {
        /// stack_iter
        stack_iter: std::iter::Take<std::slice::Iter<'a, T>>,
        /// heap_iter
        heap_iter: std::slice::Iter<'a, T>,
    },
}

impl<'a, T: Default, const INITIAL_CAPACITY: usize> Iterator
    for HybridVecIter<'a, T, INITIAL_CAPACITY>
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            HybridVecIter::StackOnly { iter } => iter.next(),
            HybridVecIter::Mixed {
                stack_iter,
                heap_iter,
            } => stack_iter.next().or_else(|| heap_iter.next()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::logs::AnyValue;
    use opentelemetry::Key;

    #[cfg(feature = "memory-profiling")]
    use crate::testing::global_allocator;

    #[cfg(feature = "memory-profiling")]
    use jemalloc_ctl::{epoch, stats};

    /// A struct to hold a key-value pair and implement `Default`.
    #[derive(Clone, Debug, PartialEq)]
    struct KeyValuePair(Key, AnyValue);

    impl Default for KeyValuePair {
        fn default() -> Self {
            KeyValuePair(Key::from_static_str(""), AnyValue::String("".into()))
        }
    }

    #[test]
    fn test_push_and_get() {
        let mut collection = HybridVec::<i32>::new();
        for i in 0..15 {
            collection.push(i);
        }
        for i in 0..15 {
            assert_eq!(collection.get(i), Some(&(i as i32)));
        }
    }

    #[test]
    fn test_len() {
        let mut collection = HybridVec::<i32>::new();
        for i in 0..15 {
            collection.push(i);
        }
        assert_eq!(collection.len(), 15);
    }

    #[test]
    fn test_into_iter() {
        let mut collection = HybridVec::<i32>::new();
        for i in 0..15 {
            collection.push(i);
        }
        let mut iter = collection.into_iter();
        for i in 0..15 {
            assert_eq!(iter.next(), Some(i));
        }
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_ref_iter() {
        let mut collection = HybridVec::<i32>::new();
        for i in 0..15 {
            collection.push(i);
        }
        let iter = &collection;
        let mut count = 0;
        for value in iter {
            assert_eq!(*value, count);
            count += 1;
        }
        assert_eq!(count, 15);
    }

    #[test]
    fn test_key_value_pair_storage_hybridvec() {
        let mut collection = HybridVec::<KeyValuePair>::new();

        let key1 = Key::from("key1");
        let value1 = AnyValue::String("value1".into());
        let key2 = Key::from("key2");
        let value2 = AnyValue::Int(42);

        collection.push(KeyValuePair(key1.clone(), value1.clone()));
        collection.push(KeyValuePair(key2.clone(), value2.clone()));

        assert_eq!(
            collection.get(0).map(|kv| (&kv.0, &kv.1)),
            Some((&key1, &value1))
        );
        assert_eq!(
            collection.get(1).map(|kv| (&kv.0, &kv.1)),
            Some((&key2, &value2))
        );
        assert_eq!(collection.len(), 2);

        // Test iterating over the key-value pairs
        let mut iter = collection.into_iter();
        assert_eq!(iter.next(), Some(KeyValuePair(key1, value1)));
        assert_eq!(iter.next(), Some(KeyValuePair(key2, value2)));
        assert_eq!(iter.next(), None);
    }

    #[cfg(feature = "memory-profiling")]
    #[test]
    fn test_memory_allocation_string() {
        // Reset jemalloc epoch to refresh stats
        let e = epoch::mib().unwrap();
        e.advance().unwrap();

        // Get memory stats before the code block
        let allocated_before = stats::allocated::read().unwrap();

        // Code block to measure: Allocate a large string
        let large_string: String = "a".repeat(100_000);

        // Refresh jemalloc stats
        e.advance().unwrap();

        // Get memory stats after the code block
        let allocated_after = stats::allocated::read().unwrap();

        // Calculate the difference
        let allocated_diff = allocated_after - allocated_before;

        // Assert or print the difference
        println!("Memory allocated for String: {} bytes", allocated_diff);
        assert!(allocated_diff > 0);
    }

    #[cfg(feature = "memory-profiling")]
    #[test]
    fn test_memory_allocation_int() {
        // Reset jemalloc epoch to refresh stats
        let e = epoch::mib().unwrap();
        e.advance().unwrap();

        // Get memory stats before the code block
        let allocated_before = stats::allocated::read().unwrap();

        // Code block to measure: Allocate a vector of integers
        let vec: Vec<i32> = (0..100_000).collect();

        // Refresh jemalloc stats
        e.advance().unwrap();

        // Get memory stats after the code block
        let allocated_after = stats::allocated::read().unwrap();

        // Calculate the difference
        let allocated_diff = allocated_after - allocated_before;

        // Assert or print the difference
        println!("Memory allocated for Vec<i32>: {} bytes", allocated_diff);
        assert!(allocated_diff > 0);
    }
}
