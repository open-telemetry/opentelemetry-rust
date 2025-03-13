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

    /// Removes the element at a specific position (index) while preserving the order.
    ///
    /// This function performs the following operations:
    ///
    /// - If the index points to an element in the internal array (`inline`):
    ///   - Removes the element at the specified index.
    ///   - Shifts the remaining elements in the internal array to the left to fill the gap, preserving the order.
    ///   - If an overflow vector exists:
    ///     - Moves the first element from the overflow vector into the last position of the internal array, if available.
    /// - If the index points to an element in the overflow vector:
    ///   - The element is removed directly from the overflow vector.
    ///
    /// # Arguments
    ///
    /// - `index`: The index of the element to be deleted.
    ///
    /// # Returns
    ///
    /// - `Some(T)`: The deleted element, if found.
    /// - `None`: If the index is out of bounds for both the internal array and the overflow vector.
    ///
    #[allow(dead_code)]
    pub(crate) fn remove_at(&mut self, index: usize) -> Option<T> {
        if index < self.count {
            let removed_value = self.inline[index].clone();

            // Shift elements in inline array to the left
            for i in index..self.count - 1 {
                self.inline[i] = self.inline[i + 1].clone();
            }

            // Handle moving an overflow element to inline, if available
            let moved_from_overflow = if let Some(ref mut overflow) = self.overflow {
                if let Some(first_from_overflow) = overflow.first().cloned() {
                    self.inline[self.count - 1] = first_from_overflow;
                    overflow.remove(0); // Remove the first element from overflow
                    true
                } else {
                    self.inline[self.count - 1] = Default::default();
                    false
                }
            } else {
                self.inline[self.count - 1] = Default::default();
                false
            };

            // Only decrement count if no item was moved from overflow
            if !moved_from_overflow {
                self.count -= 1;
            }
            return Some(removed_value);
        }

        // Handle removing from the overflow vector
        if let Some(ref mut overflow) = self.overflow {
            let overflow_index = index - MAX_INLINE_CAPACITY;
            if overflow_index < overflow.len() {
                return Some(overflow.remove(overflow_index));
            }
        }

        // Index out of bounds
        None
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
        if self.overflow.is_none() || self.overflow.as_ref().unwrap().is_empty() {
            self.inline.iter().take(self.count).chain([].iter()) // Chaining with an empty array
                                                                 // so that both `if` and `else` branch return the same type
        } else {
            self.inline
                .iter()
                .take(self.count)
                .chain(self.overflow.as_ref().unwrap().iter())
        }
    }

    /// Returns a mutable iterator over the elements in the `GrowableArray`.
    ///
    /// The iterator yields elements from the internal array (`initial`) first, followed by elements
    /// from the vector (`overflow`) if present. This allows for efficient iteration over both
    /// stack-allocated and heap-allocated portions.
    ///
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        if self.overflow.is_none() || self.overflow.as_ref().unwrap().is_empty() {
            self.inline.iter_mut().take(self.count).chain([].iter_mut()) // Chaining with an empty array
                                                                         // so that both `if` and `else` branch return the same type
        } else {
            self.inline
                .iter_mut()
                .take(self.count)
                .chain(self.overflow.as_mut().unwrap().iter_mut())
        }
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
        if source.overflow.is_none() || source.overflow.as_ref().unwrap().is_empty() {
            source
                .inline
                .into_iter()
                .take(source.count)
                .chain(Vec::<T>::new())
        } else {
            source
                .inline
                .into_iter()
                .take(source.count)
                .chain(source.overflow.unwrap())
        }
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
    fn test_mut_iter_all_cases() {
        let mut collection = GrowableArray::<i32>::new();

        // Case 1: Try to modify values in an empty list
        for value in collection.iter_mut() {
            *value *= 2; // This should not be executed
        }
        assert_eq!(collection.len(), 0);
        assert_eq!(collection.get(0), None);

        // Case 2: Add a single element and modify it
        collection.push(5);
        for value in collection.iter_mut() {
            *value *= 2;
        }
        assert_eq!(collection.get(0), Some(&10));
        assert_eq!(collection.len(), 1);

        // Case 3: Add more elements and modify them
        for i in 1..10 {
            collection.push(i);
        }
        for (i, value) in collection.iter_mut().enumerate() {
            *value = i as i32 * 3; // Set values to i * 3
        }
        for i in 0..10 {
            assert_eq!(collection.get(i), Some(&(i as i32 * 3)));
        }
    }
    #[test]
    fn test_remove_at_inline() {
        let mut collection = GrowableArray::<i32>::new();
        for i in 0..DEFAULT_MAX_INLINE_CAPACITY {
            collection.push(i as i32);
        }
        assert_eq!(collection.len(), DEFAULT_MAX_INLINE_CAPACITY);

        // Remove a value from the inline array using remove_at
        let removed = collection.remove_at(3);
        assert_eq!(removed, Some(3));
        assert_eq!(collection.len(), DEFAULT_MAX_INLINE_CAPACITY - 1);

        // Ensure the array shifted correctly and the value was removed
        for i in 0..3 {
            assert_eq!(collection.get(i), Some(&(i as i32)));
        }
        for i in 3..collection.len() {
            assert_eq!(collection.get(i), Some(&((i + 1) as i32)));
        }

        // Try to remove a value out of bounds
        let non_existent = collection.remove_at(99);
        assert_eq!(non_existent, None);
    }

    #[test]
    fn test_remove_at_overflow() {
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

        // Remove a value from the overflow vector using remove_at
        let removed = collection.remove_at(DEFAULT_MAX_INLINE_CAPACITY + 2);
        assert_eq!(removed, Some(12));
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
    fn test_remove_at_last_element() {
        let mut collection = GrowableArray::<i32>::new();
        collection.push(10);
        assert_eq!(collection.len(), 1);

        // Remove the only element in the collection using remove_at
        let removed = collection.remove_at(0);
        assert_eq!(removed, Some(10));
        assert_eq!(collection.len(), 0);

        // Ensure it's empty
        assert_eq!(collection.get(0), None);
    }

    #[test]
    fn test_remove_at_from_inline_and_replace_with_overflow() {
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

        // Remove an inline value and ensure that an overflow value takes its place using remove_at
        let removed = collection.remove_at(5);
        assert_eq!(removed, Some(5));
        assert_eq!(collection.len(), DEFAULT_MAX_INLINE_CAPACITY + 2);

        // The last inline position should now be filled with the first overflow element
        assert_eq!(collection.get(DEFAULT_MAX_INLINE_CAPACITY - 1), Some(&10));
    }
}
