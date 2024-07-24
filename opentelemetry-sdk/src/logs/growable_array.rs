use std::array::IntoIter;

/// The default max capacity for the stack portation of `GrowableArray`.
const DEFAULT_MAX_STACK_CAPACITY: usize = 10;
/// The default initial capacity for the vector portion of `GrowableArray`.
const DEFAULT_INITIAL_VEC_CAPACITY: usize = 5;

#[derive(Debug, Clone, PartialEq)]
/// A hybrid vector that starts with a fixed-size array and grows dynamically with a vector.
pub struct GrowableArray<
    T: Default + Clone + PartialEq,
    const MAX_STACK_CAPACITY: usize = DEFAULT_MAX_STACK_CAPACITY,
    const INITIAL_VEC_CAPACITY: usize = DEFAULT_INITIAL_VEC_CAPACITY,
> {
    initial: [T; MAX_STACK_CAPACITY],
    additional: Option<Vec<T>>,
    count: usize,
}

impl<
        T: Default + Clone + PartialEq,
        const MAX_STACK_CAPACITY: usize,
        const INITIAL_VEC_CAPACITY: usize,
    > Default for GrowableArray<T, MAX_STACK_CAPACITY, INITIAL_VEC_CAPACITY>
{
    fn default() -> Self {
        Self {
            initial: [(); MAX_STACK_CAPACITY].map(|_| T::default()),
            additional: None,
            count: 0,
        }
    }
}

impl<
        T: Default + Clone + PartialEq,
        const MAX_STACK_CAPACITY: usize,
        const INITIAL_VEC_CAPACITY: usize,
    > GrowableArray<T, MAX_STACK_CAPACITY, INITIAL_VEC_CAPACITY>
{
    /// Creates a new `GrowableArray` with the default initial capacity.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes a value into the `GrowableArray`.
    pub fn push(&mut self, value: T) {
        if self.count < MAX_STACK_CAPACITY {
            self.initial[self.count] = value;
            self.count += 1;
        } else {
            if self.additional.is_none() {
                // Initialize the vector with a specified capacity
                self.additional = Some(Vec::with_capacity(INITIAL_VEC_CAPACITY));
            }
            self.additional.as_mut().unwrap().push(value);
        }
    }

    /// Gets a reference to the value at the specified index.
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.count {
            Some(&self.initial[index])
        } else if let Some(ref additional) = self.additional {
            additional.get(index - MAX_STACK_CAPACITY)
        } else {
            None
        }
    }

    /// Returns the number of elements in the `GrowableArray`.
    pub fn len(&self) -> usize {
        self.count + self.additional.as_ref().map_or(0, Vec::len)
    }

    /// Returns an iterator over the elements in the `GrowableArray`.
    pub fn iter(&self) -> GrowableArrayIter<'_, T, MAX_STACK_CAPACITY> {
        if self.additional.is_none() || self.additional.as_ref().unwrap().is_empty() {
            GrowableArrayIter::StackOnly {
                iter: self.initial.iter().take(self.count),
            }
        } else {
            GrowableArrayIter::Mixed {
                stack_iter: self.initial.iter().take(self.count),
                heap_iter: self.additional.as_ref().unwrap().iter(),
            }
        }
    }

    /// Checks if the `GrowableArray` contains the specified value.
    pub fn contains(&self, value: &T) -> bool {
        self.initial[..self.count].contains(value)
            || self
                .additional
                .as_ref()
                .map_or(false, |vec| vec.contains(value))
    }

    /// Maps each element to a new `GrowableArray` using the provided function.
    pub fn map<U: Default + Clone + PartialEq, F>(
        &self,
        mut f: F,
    ) -> GrowableArray<U, MAX_STACK_CAPACITY>
    where
        F: FnMut(&T) -> U,
    {
        let mut new_vec = GrowableArray::<U, MAX_STACK_CAPACITY>::new();

        for i in 0..self.count {
            new_vec.push(f(&self.initial[i]));
        }
        if let Some(ref additional) = self.additional {
            for value in additional {
                new_vec.push(f(value));
            }
        }

        new_vec
    }
}

// Implement `IntoIterator` for `GrowableArray`
impl<T: Default + Clone + PartialEq, const INITIAL_CAPACITY: usize> IntoIterator
    for GrowableArray<T, INITIAL_CAPACITY>
{
    type Item = T;
    type IntoIter = GrowableArrayIntoIter<T, INITIAL_CAPACITY>;

    fn into_iter(self) -> Self::IntoIter {
        if self.additional.is_none() || self.additional.as_ref().unwrap().is_empty() {
            GrowableArrayIntoIter::StackOnly {
                iter: self.initial.into_iter().take(self.count),
            }
        } else {
            GrowableArrayIntoIter::Mixed {
                stack_iter: self.initial.into_iter().take(self.count),
                heap_iter: self.additional.unwrap().into_iter(),
            }
        }
    }
}

#[derive(Debug)]
/// Iterator for consuming a `GrowableArray`.
pub enum GrowableArrayIntoIter<T: Default + Clone + PartialEq, const INITIAL_CAPACITY: usize> {
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

impl<T: Default + Clone + PartialEq, const INITIAL_CAPACITY: usize> Iterator
    for GrowableArrayIntoIter<T, INITIAL_CAPACITY>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GrowableArrayIntoIter::StackOnly { iter } => iter.next(),
            GrowableArrayIntoIter::Mixed {
                stack_iter,
                heap_iter,
            } => stack_iter.next().or_else(|| heap_iter.next()),
        }
    }
}

// Implement `IntoIterator` for a reference to `GrowableArray`
impl<'a, T: Default + Clone + PartialEq + 'a, const INITIAL_CAPACITY: usize> IntoIterator
    for &'a GrowableArray<T, INITIAL_CAPACITY>
{
    type Item = &'a T;
    type IntoIter = GrowableArrayIter<'a, T, INITIAL_CAPACITY>;

    fn into_iter(self) -> Self::IntoIter {
        if self.additional.is_none() || self.additional.as_ref().unwrap().is_empty() {
            GrowableArrayIter::StackOnly {
                iter: self.initial.iter().take(self.count),
            }
        } else {
            GrowableArrayIter::Mixed {
                stack_iter: self.initial.iter().take(self.count),
                heap_iter: self.additional.as_ref().unwrap().iter(),
            }
        }
    }
}

#[derive(Debug)]
/// Iterator for referencing elements in a `GrowableArray`.
pub enum GrowableArrayIter<'a, T: Default, const INITIAL_CAPACITY: usize> {
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

impl<'a, T: Default + Clone, const INITIAL_CAPACITY: usize> Iterator
    for GrowableArrayIter<'a, T, INITIAL_CAPACITY>
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GrowableArrayIter::StackOnly { iter } => iter.next(),
            GrowableArrayIter::Mixed {
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
        let mut collection = GrowableArray::<i32>::new();
        for i in 0..15 {
            collection.push(i);
        }
        for i in 0..15 {
            assert_eq!(collection.get(i), Some(&(i as i32)));
        }
    }

    #[test]
    fn test_len() {
        let mut collection = GrowableArray::<i32>::new();
        for i in 0..15 {
            collection.push(i);
        }
        assert_eq!(collection.len(), 15);
    }

    #[test]
    fn test_into_iter() {
        let mut collection = GrowableArray::<i32>::new();
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
        let mut collection = GrowableArray::<i32>::new();
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
    fn test_key_value_pair_storage_growable_array() {
        let mut collection = GrowableArray::<KeyValuePair>::new();

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

    #[test]
    fn test_contains() {
        let mut collection = GrowableArray::<i32>::new();
        for i in 0..10 {
            collection.push(i);
        }
        assert!(collection.contains(&5));
        assert!(!collection.contains(&15));

        collection.push(15);
        assert!(collection.contains(&15));
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
