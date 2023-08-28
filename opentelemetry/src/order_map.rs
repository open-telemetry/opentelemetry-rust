use crate::{Key, KeyValue, Value};
use indexmap::map::{
    Drain, Entry, IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys, Values, ValuesMut,
};
use indexmap::{Equivalent, IndexMap};
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash};
use std::iter::FromIterator;
use std::ops::{Index, IndexMut, RangeBounds};

/// A hash table implementation that preserves insertion order across all operations.
///
/// Entries will be returned according to their insertion order when iterating over the collection.
#[derive(Clone, Debug)]
pub struct OrderMap<K, V, S = RandomState>(IndexMap<K, V, S>);

impl<K, V> OrderMap<K, V> {
    /// Create a new map. (Does not allocate)
    #[inline]
    pub fn new() -> Self {
        Self(IndexMap::new())
    }

    /// Create a new map with capacity for `n` key-value pairs. (Does not
    /// allocate if `n` is zero.)
    ///
    /// Computes in **O(n)** time.
    #[inline]
    pub fn with_capacity(n: usize) -> Self {
        Self(IndexMap::with_capacity(n))
    }
}

impl<K, V, S> OrderMap<K, V, S> {
    /// Create a new map with capacity for `n` key-value pairs. (Does not
    /// allocate if `n` is zero.)
    ///
    /// Computes in **O(n)** time.
    #[inline]
    pub fn with_capacity_and_hasher(n: usize, hash_builder: S) -> Self {
        Self(IndexMap::with_capacity_and_hasher(n, hash_builder))
    }

    /// Create a new map with `hash_builder`.
    ///
    /// This function is `const`, so it
    /// can be called in `static` contexts.
    pub const fn with_hasher(hash_builder: S) -> Self {
        Self(IndexMap::with_hasher(hash_builder))
    }

    /// Computes in **O(1)** time.
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Return a reference to the map's `BuildHasher`.
    pub fn hasher(&self) -> &S {
        self.0.hasher()
    }

    /// Return the number of key-value pairs in the map.
    ///
    /// Computes in **O(1)** time.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the map contains no elements.
    ///
    /// Computes in **O(1)** time.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return an iterator over the key-value pairs of the map, in their order
    pub fn iter(&self) -> Iter<'_, K, V> {
        self.0.iter()
    }

    /// Return an iterator over the key-value pairs of the map, in their order
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        self.0.iter_mut()
    }

    /// Return an iterator over the keys of the map, in their order
    pub fn keys(&self) -> Keys<'_, K, V> {
        self.0.keys()
    }

    /// Return an owning iterator over the keys of the map, in their order
    pub fn into_keys(self) -> IntoKeys<K, V> {
        self.0.into_keys()
    }

    /// Return an iterator over the values of the map, in their order
    pub fn values(&self) -> Values<'_, K, V> {
        self.0.values()
    }

    /// Return an iterator over mutable references to the values of the map,
    /// in their order
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        self.0.values_mut()
    }

    /// Return an owning iterator over the values of the map, in their order
    pub fn into_values(self) -> IntoValues<K, V> {
        self.0.into_values()
    }

    /// Remove all key-value pairs in the map, while preserving its capacity.
    ///
    /// Computes in **O(n)** time.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Shortens the map, keeping the first `len` elements and dropping the rest.
    ///
    /// If `len` is greater than the map's current length, this has no effect.
    pub fn truncate(&mut self, len: usize) {
        self.0.truncate(len);
    }

    /// Clears the `IndexMap` in the given index range, returning those
    /// key-value pairs as a drain iterator.
    ///
    /// The range may be any type that implements `RangeBounds<usize>`,
    /// including all of the `std::ops::Range*` types, or even a tuple pair of
    /// `Bound` start and end values. To drain the map entirely, use `RangeFull`
    /// like `map.drain(..)`.
    ///
    /// This shifts down all entries following the drained range to fill the
    /// gap, and keeps the allocated memory for reuse.
    ///
    /// ***Panics*** if the starting point is greater than the end point or if
    /// the end point is greater than the length of the map.
    pub fn drain<R>(&mut self, range: R) -> Drain<'_, K, V>
    where
        R: RangeBounds<usize>,
    {
        self.0.drain(range)
    }

    /// Splits the collection into two at the given index.
    ///
    /// Returns a newly allocated map containing the elements in the range
    /// `[at, len)`. After the call, the original map will be left containing
    /// the elements `[0, at)` with its previous capacity unchanged.
    ///
    /// ***Panics*** if `at > len`.
    pub fn split_off(&mut self, at: usize) -> Self
    where
        S: Clone,
    {
        Self(self.0.split_off(at))
    }
}

impl<K, V, S> OrderMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    /// Reserve capacity for `additional` more key-value pairs.
    ///
    /// Computes in **O(n)** time.
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    /// Shrink the capacity of the map as much as possible.
    ///
    /// Computes in **O(n)** time.
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    /// Insert a key-value pair in the map.
    ///
    /// If an equivalent key already exists in the map: the key remains and
    /// retains in its place in the order, its corresponding value is updated
    /// with `value` and the older value is returned inside `Some(_)`.
    ///
    /// If no equivalent key existed in the map: the new key-value pair is
    /// inserted, last in order, and `None` is returned.
    ///
    /// Computes in **O(1)** time (amortized average).
    ///
    /// See also [`entry`](#method.entry) if you you want to insert *or* modify
    /// or if you need to get the index of the corresponding key-value pair.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.0.insert(key, value)
    }

    /// Insert a key-value pair in the map, and get their index.
    ///
    /// If an equivalent key already exists in the map: the key remains and
    /// retains in its place in the order, its corresponding value is updated
    /// with `value` and the older value is returned inside `(index, Some(_))`.
    ///
    /// If no equivalent key existed in the map: the new key-value pair is
    /// inserted, last in order, and `(index, None)` is returned.
    ///
    /// Computes in **O(1)** time (amortized average).
    ///
    /// See also [`entry`](#method.entry) if you you want to insert *or* modify
    /// or if you need to get the index of the corresponding key-value pair.
    pub fn insert_full(&mut self, key: K, value: V) -> (usize, Option<V>) {
        self.0.insert_full(key, value)
    }

    /// Get the given key’s corresponding entry in the map for insertion and/or
    /// in-place manipulation.
    ///
    /// Computes in **O(1)** time (amortized average).
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        self.0.entry(key)
    }

    /// Return `true` if an equivalent to `key` exists in the map.
    ///
    /// Computes in **O(1)** time (average).
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        Q: Hash + Equivalent<K>,
    {
        self.0.contains_key(key)
    }

    /// Return a reference to the value stored for `key`, if it is present,
    /// else `None`.
    ///
    /// Computes in **O(1)** time (average).
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        Q: Hash + Equivalent<K>,
    {
        self.0.get(key)
    }

    /// Return references to the key-value pair stored for `key`,
    /// if it is present, else `None`.
    ///
    /// Computes in **O(1)** time (average).
    pub fn get_key_value<Q: ?Sized>(&self, key: &Q) -> Option<(&K, &V)>
    where
        Q: Hash + Equivalent<K>,
    {
        self.0.get_key_value(key)
    }

    /// Return item index, key and value
    pub fn get_full<Q: ?Sized>(&self, key: &Q) -> Option<(usize, &K, &V)>
    where
        Q: Hash + Equivalent<K>,
    {
        self.0.get_full(key)
    }

    /// Return item index, if it exists in the map
    ///
    /// Computes in **O(1)** time (average).
    pub fn get_index_of<Q: ?Sized>(&self, key: &Q) -> Option<usize>
    where
        Q: Hash + Equivalent<K>,
    {
        self.0.get_index_of(key)
    }

    /// Return a mutable reference to the element pointed at by `key`, if it exists.
    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        Q: Hash + Equivalent<K>,
    {
        self.0.get_mut(key)
    }

    /// Return a mutable reference to the element pointed at by `key`, if it exists.
    /// It also returns the element's index and its key.
    pub fn get_full_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<(usize, &K, &mut V)>
    where
        Q: Hash + Equivalent<K>,
    {
        self.0.get_full_mut(key)
    }

    /// Remove the key-value pair equivalent to `key` and return
    /// its value.
    ///
    /// Like `Vec::remove`, the pair is removed by shifting all of the
    /// elements that follow it, preserving their relative order.
    /// **This perturbs the index of all of those elements!**
    ///
    /// Return `None` if `key` is not in map.
    ///
    /// Computes in **O(n)** time (average).
    pub fn shift_remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        Q: Hash + Equivalent<K>,
    {
        self.0.shift_remove(key)
    }

    /// Remove and return the key-value pair equivalent to `key`.
    ///
    /// Like `Vec::remove`, the pair is removed by shifting all of the
    /// elements that follow it, preserving their relative order.
    /// **This perturbs the index of all of those elements!**
    ///
    /// Return `None` if `key` is not in map.
    ///
    /// Computes in **O(n)** time (average).
    pub fn shift_remove_entry<Q: ?Sized>(&mut self, key: &Q) -> Option<(K, V)>
    where
        Q: Hash + Equivalent<K>,
    {
        self.0.shift_remove_entry(key)
    }

    /// Remove the key-value pair equivalent to `key` and return it and
    /// the index it had.
    ///
    /// Like `Vec::remove`, the pair is removed by shifting all of the
    /// elements that follow it, preserving their relative order.
    /// **This perturbs the index of all of those elements!**
    ///
    /// Return `None` if `key` is not in map.
    ///
    /// Computes in **O(n)** time (average).
    pub fn shift_remove_full<Q: ?Sized>(&mut self, key: &Q) -> Option<(usize, K, V)>
    where
        Q: Hash + Equivalent<K>,
    {
        self.0.shift_remove_full(key)
    }

    /// Remove the last key-value pair
    ///
    /// This preserves the order of the remaining elements.
    ///
    /// Computes in **O(1)** time (average).
    pub fn pop(&mut self) -> Option<(K, V)> {
        self.0.pop()
    }

    /// Scan through each key-value pair in the map and keep those where the
    /// closure `keep` returns `true`.
    ///
    /// The elements are visited in order, and remaining elements keep their
    /// order.
    ///
    /// Computes in **O(n)** time (average).
    pub fn retain<F>(&mut self, keep: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        self.0.retain(keep);
    }
}

impl<K, V, S> OrderMap<K, V, S> {
    /// Get a key-value pair by index
    ///
    /// Valid indices are *0 <= index < self.len()*
    ///
    /// Computes in **O(1)** time.
    pub fn get_index(&self, index: usize) -> Option<(&K, &V)> {
        self.0.get_index(index)
    }

    /// Get a key-value pair by index
    ///
    /// Valid indices are *0 <= index < self.len()*
    ///
    /// Computes in **O(1)** time.
    pub fn get_index_mut(&mut self, index: usize) -> Option<(&K, &mut V)> {
        self.0.get_index_mut(index)
    }

    /// Get the first key-value pair
    ///
    /// Computes in **O(1)** time.
    pub fn first(&self) -> Option<(&K, &V)> {
        self.0.first()
    }

    /// Get the first key-value pair, with mutable access to the value
    ///
    /// Computes in **O(1)** time.
    pub fn first_mut(&mut self) -> Option<(&K, &mut V)> {
        self.0.first_mut()
    }

    /// Get the last key-value pair
    ///
    /// Computes in **O(1)** time.
    pub fn last(&self) -> Option<(&K, &V)> {
        self.0.last()
    }

    /// Get the last key-value pair, with mutable access to the value
    ///
    /// Computes in **O(1)** time.
    pub fn last_mut(&mut self) -> Option<(&K, &mut V)> {
        self.0.last_mut()
    }

    /// Remove the key-value pair by index
    ///
    /// Valid indices are *0 <= index < self.len()*
    ///
    /// Like `Vec::remove`, the pair is removed by shifting all of the
    /// elements that follow it, preserving their relative order.
    /// **This perturbs the index of all of those elements!**
    ///
    /// Computes in **O(n)** time (average).
    pub fn shift_remove_index(&mut self, index: usize) -> Option<(K, V)> {
        self.0.shift_remove_index(index)
    }
}

impl<'a, K, V, S> IntoIterator for &'a OrderMap<K, V, S> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut OrderMap<K, V, S> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<K, V, S> IntoIterator for OrderMap<K, V, S> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Access `OrderMap` values corresponding to a key.
///
/// Panics if the value is missing.
impl<K, V, Q: ?Sized, S> Index<&Q> for OrderMap<K, V, S>
where
    Q: Hash + Equivalent<K>,
    K: Hash + Eq,
    S: BuildHasher,
{
    type Output = V;

    /// Returns a reference to the value corresponding to the supplied `key`.
    ///
    /// ***Panics*** if `key` is not present in the map.
    fn index(&self, key: &Q) -> &V {
        self.0.index(key)
    }
}

/// Access `Ordermap` values corresponding to a key.
///
/// Mutable indexing allows changing / updating values of key-value
/// pairs that are already present.
///
/// You can **not** insert new pairs with index syntax, use `.insert()`.
impl<K, V, Q: ?Sized, S> IndexMut<&Q> for OrderMap<K, V, S>
where
    Q: Hash + Equivalent<K>,
    K: Hash + Eq,
    S: BuildHasher,
{
    /// Returns a mutable reference to the value corresponding to the supplied `key`.
    ///
    /// ***Panics*** if `key` is not present in the map.
    fn index_mut(&mut self, key: &Q) -> &mut V {
        self.0.index_mut(key)
    }
}

/// Access `IndexMap` values at indexed positions.
///
/// It panics if the index is out of bounds.
impl<K, V, S> Index<usize> for OrderMap<K, V, S> {
    type Output = V;

    /// Returns a reference to the value at the supplied `index`.
    ///
    /// ***Panics*** if `index` is out of bounds.
    fn index(&self, index: usize) -> &V {
        self.0.index(index)
    }
}

/// Access `IndexMap` values at indexed positions.
///
/// Mutable indexing allows changing / updating indexed values
/// that are already present.
///
/// You can **not** insert new values with index syntax, use `.insert()`.
///
/// # Examples
///
/// ```
/// use indexmap::IndexMap;
///
/// let mut map = IndexMap::new();
/// for word in "Lorem ipsum dolor sit amet".split_whitespace() {
///     map.insert(word.to_lowercase(), word.to_string());
/// }
/// let lorem = &mut map[0];
/// assert_eq!(lorem, "Lorem");
/// lorem.retain(char::is_lowercase);
/// assert_eq!(map["lorem"], "orem");
/// ```
///
/// ```should_panic
/// use indexmap::IndexMap;
///
/// let mut map = IndexMap::new();
/// map.insert("foo", 1);
/// map[10] = 1; // panics!
/// ```
impl<K, V, S> IndexMut<usize> for OrderMap<K, V, S> {
    /// Returns a mutable reference to the value at the supplied `index`.
    ///
    /// ***Panics*** if `index` is out of bounds.
    fn index_mut(&mut self, index: usize) -> &mut V {
        self.0.index_mut(index)
    }
}

impl<K, V, S> FromIterator<(K, V)> for OrderMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher + Default,
{
    /// Create an `OrderMap` from the sequence of key-value pairs in the
    /// iterable.
    ///
    /// `from_iter` uses the same logic as `extend`. See
    /// [`extend`](#method.extend) for more details.
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iterable: I) -> Self {
        Self(IndexMap::from_iter(iterable))
    }
}

// todo: uncomment when the MSRV bumps
// impl<K, V, const N: usize> From<[(K, V); N]> for OrderMap<K, V, RandomState>
// where
//     K: Hash + Eq,
// {
//     fn from(arr: [(K, V); N]) -> Self {
//         Self(IndexMap::from(arr))
//     }
// }

impl<K, V, S> Extend<(K, V)> for OrderMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    /// Extend the map with all key-value pairs in the iterable.
    ///
    /// This is equivalent to calling [`insert`](#method.insert) for each of
    /// them in order, which means that for keys that already existed
    /// in the map, their value is updated but it keeps the existing order.
    ///
    /// New keys are inserted in the order they appear in the sequence. If
    /// equivalents of a key occur more than once, the last corresponding value
    /// prevails.
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iterable: I) {
        self.0.extend(iterable)
    }
}

impl<'a, K, V, S> Extend<(&'a K, &'a V)> for OrderMap<K, V, S>
where
    K: 'a + Hash + Eq + Copy,
    V: 'a + Copy,
    S: BuildHasher,
{
    /// Extend the map with all key-value pairs in the iterable.
    ///
    /// See the first extend method for more details.
    fn extend<I: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iterable: I) {
        self.0.extend(iterable)
    }
}

impl<K, V, S> Default for OrderMap<K, V, S>
where
    S: Default,
{
    /// Return an empty `OrderMap`
    fn default() -> Self {
        Self(IndexMap::default())
    }
}

impl<K, V1, S1, V2, S2> PartialEq<OrderMap<K, V2, S2>> for OrderMap<K, V1, S1>
where
    K: Hash + Eq,
    V1: PartialEq<V2>,
    S1: BuildHasher,
    S2: BuildHasher,
{
    fn eq(&self, other: &OrderMap<K, V2, S2>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<K, V, S> Eq for OrderMap<K, V, S>
where
    K: Eq + Hash,
    V: Eq,
    S: BuildHasher,
{
}

impl<S> FromIterator<KeyValue> for OrderMap<Key, Value, S>
where
    S: BuildHasher + Default,
{
    /// Create an `OrderMap` from the sequence of key-value pairs in the
    /// iterable.
    ///
    /// `from_iter` uses the same logic as `extend`. See
    /// [`extend`](#method.extend) for more details.
    fn from_iter<I: IntoIterator<Item = KeyValue>>(iterable: I) -> Self {
        Self(IndexMap::from_iter(
            iterable.into_iter().map(|kv| (kv.key, kv.value)),
        ))
    }
}

// todo: uncomment below when bumping MSRV
// impl<const N: usize> From<[KeyValue; N]> for OrderMap<Key, Value, RandomState> {
//     fn from(arr: [KeyValue; N]) -> Self {
//         let arr = arr.map(|kv| (kv.key, kv.value));
//         Self(IndexMap::from(arr))
//     }
// }

impl<S> Extend<KeyValue> for OrderMap<Key, Value, S>
where
    S: BuildHasher,
{
    /// Extend the map with all key-value pairs in the iterable.
    ///
    /// This is equivalent to calling [`insert`](#method.insert) for each of
    /// them in order, which means that for keys that already existed
    /// in the map, their value is updated but it keeps the existing order.
    ///
    /// New keys are inserted in the order they appear in the sequence. If
    /// equivalents of a key occur more than once, the last corresponding value
    /// prevails.
    fn extend<I: IntoIterator<Item = KeyValue>>(&mut self, iterable: I) {
        self.0
            .extend(iterable.into_iter().map(|kv| (kv.key, kv.value)))
    }
}
