/// The default max capacity for the stack portion of `GrowableArray`.
const DEFAULT_MAX_INLINE_CAPACITY: usize = 10;
/// The default initial capacity for the vector portion of `GrowableArray`.
const DEFAULT_INITIAL_OVERFLOW_CAPACITY: usize = 5;

#[derive(Debug, Clone, PartialEq)]
/// A hybrid vector that starts with a fixed-size array and grows dynamically with a vector.
///
/// `GrowableArray` uses an internal fixed-size array (`inline`) for storing elements until it reaches
/// `MAX_INLINE_CAPACITY`. When this capacity is exceeded, additional elements are stored in a heap-allocated
/// vector (`overflow`). This structure allows for efficient use of stack memory for small numbers of elements,
/// while still supporting dynamic growth.
///
pub(crate) struct GrowableArray<
    T: Default + Clone + PartialEq,
    const MAX_INLINE_CAPACITY: usize = DEFAULT_MAX_INLINE_CAPACITY,
    const INITIAL_OVERFLOW_CAPACITY: usize = DEFAULT_INITIAL_OVERFLOW_CAPACITY,
> {
    inline: [T; MAX_INLINE_CAPACITY],
    overflow: Option<Vec<T>>,
    count: usize,
}

impl<
        T: Default + Clone + PartialEq,
        const MAX_INLINE_CAPACITY: usize,
        const INITIAL_OVERFLOW_CAPACITY: usize,
    > Default for GrowableArray<T, MAX_INLINE_CAPACITY, INITIAL_OVERFLOW_CAPACITY>
{
    fn default() -> Self {
        Self {
            inline: [(); MAX_INLINE_CAPACITY].map(|_| T::default()),
            overflow: None,
            count: 0,
        }
    }
}

impl<
        T: Default + Clone + PartialEq,
        const MAX_INLINE_CAPACITY: usize,
        const INITIAL_OVERFLOW_CAPACITY: usize,
    > GrowableArray<T, MAX_INLINE_CAPACITY, INITIAL_OVERFLOW_CAPACITY>
{
    /// Creates a new `GrowableArray` with the default initial capacity.
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Pushes a value into the `GrowableArray`.
    ///
    /// If the internal array (`inline`) has reached its capacity (`MAX_INLINE_CAPACITY`), the value is pushed
    /// into the heap-allocated vector (`overflow`). Otherwise, it is stored in the array.
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn push(&mut self, value: T) {
        if self.count < MAX_INLINE_CAPACITY {
            self.inline[self.count] = value;
            self.count += 1;
        } else {
            self.overflow
                .get_or_insert_with(|| Vec::with_capacity(INITIAL_OVERFLOW_CAPACITY))
                .push(value);
        }
    }

    /// Gets a reference to the value at the specified index.
    ///
    /// Returns `None` if the index is out of bounds.
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn get(&self, index: usize) -> Option<&T> {
        if index < self.count {
            Some(&self.inline[index])
        } else if let Some(ref overflow) = self.overflow {
            overflow.get(index - MAX_INLINE_CAPACITY)
        } else {
            None
        }
    }

    /// Returns the number of elements in the `GrowableArray`.
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.count + self.overflow.as_ref().map_or(0, Vec::len)
    }

    /// Retains only the elements for which `f` returns `true`, and preserves their order.
    ///
    /// This function first retains elements in the interal array and then moves items
    /// from the overflow vector into the internal array to fill any gaps.
    /// It ensures that the internal array remains full while the overflow vector shrinks.
    ///
    /// Elements are moved and not cloned. The closure gets a mutable reference, so it can
    /// also change the elements that it retains.
    ///
    /// # Arguments
    ///
    /// - `f`: The predicate. Returns `true` to retain the element, `false` to remove it.
    ///
    /// # Returns
    ///
    /// - The number of removed elements.
    ///
    #[cfg(any(feature = "logs", test))]
    #[inline]
    pub(crate) fn retain_mut<F>(&mut self, mut f: F) -> usize
    where
        F: FnMut(&mut T) -> bool,
    {
        let len_before = self.len();

        let mut kept = 0;
        for i in 0..self.count {
            if f(&mut self.inline[i]) {
                // Slots `kept..i` hold only removed elements, so the swap keeps the order.
                self.inline.swap(kept, i);
                kept += 1;
            } else {
                self.inline[i] = T::default();
            }
        }

        if let Some(overflow) = self.overflow.as_mut() {
            overflow.retain_mut(&mut f);
            let moved = (MAX_INLINE_CAPACITY - kept).min(overflow.len());
            for (slot, value) in self.inline[kept..kept + moved]
                .iter_mut()
                .zip(overflow.drain(..moved))
            {
                *slot = value;
            }
            kept += moved;
            // `get` expects a full internal array while the overflow vector exists.
            if overflow.is_empty() {
                self.overflow = None;
            }
        }
        self.count = kept;

        len_before - self.len()
    }

    /// Returns an iterator over the elements in the `GrowableArray`.
    ///
    /// The iterator yields elements from the internal array (`initial`) first, followed by elements
    /// from the vector (`overflow`) if present. This allows for efficient iteration over both
    /// stack-allocated and heap-allocated portions.
    ///
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        let overflow_slice = match &self.overflow {
            Some(v) if !v.is_empty() => v.as_slice(),
            _ => &[],
        };
        self.inline
            .iter()
            .take(self.count)
            .chain(overflow_slice.iter())
    }
}

// Implement `IntoIterator` for `GrowableArray`
impl<T: Default + Clone + PartialEq, const INLINE_CAPACITY: usize> IntoIterator
    for GrowableArray<T, INLINE_CAPACITY>
{
    type Item = T;
    type IntoIter = GrowableArrayIntoIter<T, INLINE_CAPACITY>;

    fn into_iter(self) -> Self::IntoIter {
        GrowableArrayIntoIter::<T, INLINE_CAPACITY>::new(self)
    }
}

/// Iterator for consuming a `GrowableArray`.
///
#[derive(Debug)]
pub(crate) struct GrowableArrayIntoIter<
    T: Default + Clone + PartialEq,
    const INLINE_CAPACITY: usize,
> {
    iter: std::iter::Chain<
        std::iter::Take<std::array::IntoIter<T, INLINE_CAPACITY>>,
        std::vec::IntoIter<T>,
    >,
}

impl<T: Default + Clone + PartialEq, const INLINE_CAPACITY: usize>
    GrowableArrayIntoIter<T, INLINE_CAPACITY>
{
    fn new(source: GrowableArray<T, INLINE_CAPACITY>) -> Self {
        Self {
            iter: Self::get_iterator(source),
        }
    }

    fn get_iterator(
        source: GrowableArray<T, INLINE_CAPACITY>,
    ) -> std::iter::Chain<
        std::iter::Take<std::array::IntoIter<T, INLINE_CAPACITY>>,
        std::vec::IntoIter<T>,
    > {
        let overflow_vec = match source.overflow {
            Some(v) if !v.is_empty() => v,
            _ => Vec::new(),
        };
        source
            .inline
            .into_iter()
            .take(source.count)
            .chain(overflow_vec)
    }
}

impl<T: Default + Clone + PartialEq, const INITIAL_CAPACITY: usize> Iterator
    for GrowableArrayIntoIter<T, INITIAL_CAPACITY>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[cfg(test)]
mod tests {
    use crate::growable_array::{
        GrowableArray, DEFAULT_INITIAL_OVERFLOW_CAPACITY, DEFAULT_MAX_INLINE_CAPACITY,
    };
    use opentelemetry::logs::AnyValue;
    use opentelemetry::Key;

    type KeyValuePair = Option<(Key, AnyValue)>;

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
        let iter = collection.iter();
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

        collection.push(Some((key1.clone(), value1.clone())));
        collection.push(Some((key2.clone(), value2.clone())));

        assert_eq!(
            collection
                .get(0)
                .and_then(|kv| kv.as_ref().map(|kv| (&kv.0, &kv.1))),
            Some((&key1, &value1))
        );
        assert_eq!(
            collection
                .get(1)
                .and_then(|kv| kv.as_ref().map(|kv| (&kv.0, &kv.1))),
            Some((&key2, &value2))
        );
        assert_eq!(collection.len(), 2);

        // Test iterating over the key-value pairs
        let mut iter = collection.into_iter();
        assert_eq!(iter.next(), Some(Some((key1, value1))));
        assert_eq!(iter.next(), Some(Some((key2, value2))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_empty_attributes() {
        let collection = GrowableArray::<KeyValuePair>::new();
        assert_eq!(collection.len(), 0);
        assert_eq!(collection.get(0), None);

        let mut iter = collection.into_iter();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_less_than_max_stack_capacity() {
        let mut collection = GrowableArray::<i32>::new();
        for i in 0..DEFAULT_MAX_INLINE_CAPACITY - 1 {
            collection.push(i as i32);
        }
        assert_eq!(collection.len(), DEFAULT_MAX_INLINE_CAPACITY - 1);

        for i in 0..DEFAULT_MAX_INLINE_CAPACITY - 1 {
            assert_eq!(collection.get(i), Some(&(i as i32)));
        }
        assert_eq!(collection.get(DEFAULT_MAX_INLINE_CAPACITY - 1), None);
        assert_eq!(collection.get(DEFAULT_MAX_INLINE_CAPACITY), None);

        let mut iter = collection.into_iter();
        for i in 0..DEFAULT_MAX_INLINE_CAPACITY - 1 {
            assert_eq!(iter.next(), Some(i as i32));
        }
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_exactly_max_stack_capacity() {
        let mut collection = GrowableArray::<i32>::new();
        for i in 0..DEFAULT_MAX_INLINE_CAPACITY {
            collection.push(i as i32);
        }
        assert_eq!(collection.len(), DEFAULT_MAX_INLINE_CAPACITY);

        for i in 0..DEFAULT_MAX_INLINE_CAPACITY {
            assert_eq!(collection.get(i), Some(&(i as i32)));
        }
        assert_eq!(collection.get(DEFAULT_MAX_INLINE_CAPACITY), None);

        let mut iter = collection.into_iter();
        for i in 0..DEFAULT_MAX_INLINE_CAPACITY {
            assert_eq!(iter.next(), Some(i as i32));
        }
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_exceeds_stack_but_not_initial_vec_capacity() {
        let mut collection = GrowableArray::<i32>::new();
        for i in 0..(DEFAULT_MAX_INLINE_CAPACITY + DEFAULT_INITIAL_OVERFLOW_CAPACITY - 1) {
            collection.push(i as i32);
        }
        assert_eq!(
            collection.len(),
            DEFAULT_MAX_INLINE_CAPACITY + DEFAULT_INITIAL_OVERFLOW_CAPACITY - 1
        );

        for i in 0..(DEFAULT_MAX_INLINE_CAPACITY + DEFAULT_INITIAL_OVERFLOW_CAPACITY - 1) {
            assert_eq!(collection.get(i), Some(&(i as i32)));
        }
        assert_eq!(
            collection.get(DEFAULT_MAX_INLINE_CAPACITY + DEFAULT_INITIAL_OVERFLOW_CAPACITY - 1),
            None
        );
        assert_eq!(
            collection.get(DEFAULT_MAX_INLINE_CAPACITY + DEFAULT_INITIAL_OVERFLOW_CAPACITY),
            None
        );

        let mut iter = collection.into_iter();
        for i in 0..(DEFAULT_MAX_INLINE_CAPACITY + DEFAULT_INITIAL_OVERFLOW_CAPACITY - 1) {
            assert_eq!(iter.next(), Some(i as i32));
        }
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_exceeds_both_stack_and_initial_vec_capacities() {
        let mut collection = GrowableArray::<i32>::new();
        for i in 0..(DEFAULT_MAX_INLINE_CAPACITY + DEFAULT_INITIAL_OVERFLOW_CAPACITY + 5) {
            collection.push(i as i32);
        }
        assert_eq!(
            collection.len(),
            DEFAULT_MAX_INLINE_CAPACITY + DEFAULT_INITIAL_OVERFLOW_CAPACITY + 5
        );

        for i in 0..(DEFAULT_MAX_INLINE_CAPACITY + DEFAULT_INITIAL_OVERFLOW_CAPACITY + 5) {
            assert_eq!(collection.get(i), Some(&(i as i32)));
        }
        assert_eq!(
            collection.get(DEFAULT_MAX_INLINE_CAPACITY + DEFAULT_INITIAL_OVERFLOW_CAPACITY + 5),
            None
        );

        let mut iter = collection.into_iter();
        for i in 0..(DEFAULT_MAX_INLINE_CAPACITY + DEFAULT_INITIAL_OVERFLOW_CAPACITY + 5) {
            assert_eq!(iter.next(), Some(i as i32));
        }
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_retain_mut_all_cases() {
        let mut collection = GrowableArray::<i32>::new();

        // Case 1: Try to modify values in an empty list
        collection.retain_mut(|value| {
            *value *= 2; // This should not be executed
            true
        });
        assert_eq!(collection.len(), 0);
        assert_eq!(collection.get(0), None);

        // Case 2: Add a single element and modify it
        collection.push(5);
        collection.retain_mut(|value| {
            *value *= 2;
            true
        });
        assert_eq!(collection.get(0), Some(&10));
        assert_eq!(collection.len(), 1);

        // Case 3: Add more elements and modify them
        for i in 1..10 {
            collection.push(i);
        }
        let mut i = 0;
        collection.retain_mut(|value| {
            *value = i * 3; // Set values to i * 3
            i += 1;
            true
        });
        for i in 0..(DEFAULT_MAX_INLINE_CAPACITY + 3) {
            assert_eq!(collection.get(i), Some(&(i as i32 * 3)));
        }
    }
    #[test]
    fn test_retain_mut_inline() {
        let mut collection = GrowableArray::<i32>::new();
        for i in 0..DEFAULT_MAX_INLINE_CAPACITY {
            collection.push(i as i32);
        }
        assert_eq!(collection.len(), DEFAULT_MAX_INLINE_CAPACITY);

        // Remove a value from the inline array using retain_mut
        let removed = collection.retain_mut(|v| *v != 3);
        assert_eq!(removed, 1);
        assert_eq!(collection.len(), DEFAULT_MAX_INLINE_CAPACITY - 1);

        // Ensure the array shifted correctly and the value was removed
        for i in 0..3 {
            assert_eq!(collection.get(i), Some(&(i as i32)));
        }
        for i in 3..collection.len() {
            assert_eq!(collection.get(i), Some(&((i + 1) as i32)));
        }

        // Try to remove a value out of bounds
        let non_existent = collection.retain_mut(|v| *v != 99);
        assert_eq!(non_existent, 0);
    }

    #[test]
    fn test_retain_mut_overflow() {
        let mut collection = GrowableArray::<i32>::new();
        // Fill inline array
        for i in 0..DEFAULT_MAX_INLINE_CAPACITY {
            collection.push(i as i32);
        }
        // Add elements to the overflow
        for i in DEFAULT_MAX_INLINE_CAPACITY..(DEFAULT_MAX_INLINE_CAPACITY + 5) {
            collection.push(i as i32);
        }
        assert_eq!(collection.len(), DEFAULT_MAX_INLINE_CAPACITY + 5);

        // Remove a value from the overflow vector using retain_mut
        let removed = collection.retain_mut(|v| *v != 12);
        assert_eq!(removed, 1);
        assert_eq!(collection.len(), DEFAULT_MAX_INLINE_CAPACITY + 4);

        // Ensure the rest of the elements are in order
        for i in 0..DEFAULT_MAX_INLINE_CAPACITY {
            assert_eq!(collection.get(i), Some(&(i as i32)));
        }
        assert_eq!(collection.get(DEFAULT_MAX_INLINE_CAPACITY), Some(&10));
        assert_eq!(collection.get(DEFAULT_MAX_INLINE_CAPACITY + 1), Some(&11));
        assert_eq!(collection.get(DEFAULT_MAX_INLINE_CAPACITY + 2), Some(&13));
    }

    #[test]
    fn test_retain_mut_last_element() {
        let mut collection = GrowableArray::<i32>::new();
        collection.push(10);
        assert_eq!(collection.len(), 1);

        // Remove the only element in the collection using retain_mut
        let removed = collection.retain_mut(|v| *v != 10);
        assert_eq!(removed, 1);
        assert_eq!(collection.len(), 0);

        // Ensure it's empty
        assert_eq!(collection.get(0), None);
    }

    #[test]
    fn test_retain_mut_from_inline_and_replace_with_overflow() {
        let mut collection = GrowableArray::<i32>::new();

        // Fill inline array
        for i in 0..DEFAULT_MAX_INLINE_CAPACITY {
            collection.push(i as i32);
        }

        // Add overflow elements
        for i in DEFAULT_MAX_INLINE_CAPACITY..(DEFAULT_MAX_INLINE_CAPACITY + 3) {
            collection.push(i as i32);
        }

        // Before removing, ensure that the count is correct
        assert_eq!(collection.len(), DEFAULT_MAX_INLINE_CAPACITY + 3);

        // Remove an inline value and ensure that an overflow value takes its place using retain_mut
        let removed = collection.retain_mut(|v| *v != 5);
        assert_eq!(removed, 1);
        assert_eq!(collection.len(), DEFAULT_MAX_INLINE_CAPACITY + 2);

        // The last inline position should now be filled with the first overflow element
        assert_eq!(collection.get(DEFAULT_MAX_INLINE_CAPACITY - 1), Some(&10));
    }

    #[test]
    fn test_retain_mut_all_elements() {
        let mut collection = GrowableArray::<i32>::new();
        // Fill inline array
        for i in 0..DEFAULT_MAX_INLINE_CAPACITY {
            collection.push(i as i32);
        }
        // Add elements to the overflow
        for i in DEFAULT_MAX_INLINE_CAPACITY..(DEFAULT_MAX_INLINE_CAPACITY + 5) {
            collection.push(i as i32);
        }

        // Remove all values from the inline array and the overflow vector using retain_mut
        let removed = collection.retain_mut(|_| false);
        assert_eq!(removed, DEFAULT_MAX_INLINE_CAPACITY + 5);

        // Ensure it's empty
        assert_eq!(collection.len(), 0);
        assert_eq!(collection.get(0), None);

        // Ensure the collection is usable after it was emptied
        collection.push(1);
        assert_eq!(collection.get(0), Some(&1));
        assert_eq!(collection.len(), 1);
    }
}
