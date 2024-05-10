use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use opentelemetry::{Array, Key, KeyValue, Value};
use ordered_float::OrderedFloat;

#[derive(Clone, Debug)]
struct HashKeyValue(KeyValue);

impl Hash for HashKeyValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.key.hash(state);
        match &self.0.value {
            Value::F64(f) => OrderedFloat(*f).hash(state),
            Value::Array(a) => match a {
                Array::Bool(b) => b.hash(state),
                Array::I64(i) => i.hash(state),
                Array::F64(f) => f.iter().for_each(|f| OrderedFloat(*f).hash(state)),
                Array::String(s) => s.hash(state),
            },
            Value::Bool(b) => b.hash(state),
            Value::I64(i) => i.hash(state),
            Value::String(s) => s.hash(state),
        };
    }
}

impl PartialOrd for HashKeyValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HashKeyValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.key.cmp(&other.0.key)
    }
}

impl PartialEq for HashKeyValue {
    fn eq(&self, other: &Self) -> bool {
        self.0.key == other.0.key
            && match (&self.0.value, &other.0.value) {
                (Value::F64(f), Value::F64(of)) => OrderedFloat(*f).eq(&OrderedFloat(*of)),
                (Value::Array(Array::F64(f)), Value::Array(Array::F64(of))) => {
                    f.len() == of.len()
                        && f.iter()
                            .zip(of.iter())
                            .all(|(f, of)| OrderedFloat(*f).eq(&OrderedFloat(*of)))
                }
                (non_float, other_non_float) => non_float.eq(other_non_float),
            }
    }
}

impl Eq for HashKeyValue {}

/// A unique set of attributes that can be used as instrument identifiers.
///
/// This must implement [Hash], [PartialEq], and [Eq] so it may be used as
/// HashMap keys and other de-duplication methods.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct AttributeSet(Vec<HashKeyValue>, u64);

impl From<&[KeyValue]> for AttributeSet {
    fn from(values: &[KeyValue]) -> Self {
        let mut seen_keys = HashSet::with_capacity(values.len());
        let vec = values
            .iter()
            .rev()
            .filter_map(|kv| {
                if seen_keys.insert(kv.key.clone()) {
                    Some(HashKeyValue(kv.clone()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        AttributeSet::new(vec)
    }
}

fn calculate_hash(values: &[HashKeyValue]) -> u64 {
    let mut hasher = DefaultHasher::new();
    values.iter().fold(&mut hasher, |mut hasher, item| {
        item.hash(&mut hasher);
        hasher
    });
    hasher.finish()
}

impl AttributeSet {
    fn new(mut values: Vec<HashKeyValue>) -> Self {
        values.sort_unstable();
        let hash = calculate_hash(&values);
        AttributeSet(values, hash)
    }

    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the set contains no elements.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Retains only the attributes specified by the predicate.
    pub fn retain<F>(&mut self, f: F)
    where
        F: Fn(&KeyValue) -> bool,
    {
        self.0.retain(|kv| f(&kv.0));

        // Recalculate the hash as elements are changed.
        self.1 = calculate_hash(&self.0);
    }

    /// Iterate over key value pairs in the set
    pub fn iter(&self) -> impl Iterator<Item = (&Key, &Value)> {
        self.0.iter().map(|kv| (&kv.0.key, &kv.0.value))
    }
}

impl Hash for AttributeSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.1)
    }
}

#[cfg(test)]
mod tests {
    use std::hash::DefaultHasher;
    use std::hash::{Hash, Hasher};

    use crate::attributes::set::HashKeyValue;
    use opentelemetry::KeyValue;

    #[test]
    fn equality_kv_float() {
        let kv1 = HashKeyValue(KeyValue::new("key", 1.0));
        let kv2 = HashKeyValue(KeyValue::new("key", 1.0));
        assert_eq!(kv1, kv2);

        let kv1 = HashKeyValue(KeyValue::new("key", 1.0));
        let kv2 = HashKeyValue(KeyValue::new("key", 1.01));
        assert_ne!(kv1, kv2);

        let kv1 = HashKeyValue(KeyValue::new("key", std::f64::NAN));
        let kv2 = HashKeyValue(KeyValue::new("key", std::f64::NAN));
        assert_eq!(kv1, kv2);

        let kv1 = HashKeyValue(KeyValue::new("key", std::f64::INFINITY));
        let kv2 = HashKeyValue(KeyValue::new("key", std::f64::INFINITY));
        assert_eq!(kv1, kv2);
    }

    #[test]
    fn hash_kv_float() {
        let kv1 = HashKeyValue(KeyValue::new("key", 1.0));
        let kv2 = HashKeyValue(KeyValue::new("key", 1.0));
        assert_eq!(hash_helper(&kv1), hash_helper(&kv2));

        let kv1 = HashKeyValue(KeyValue::new("key", std::f64::NAN));
        let kv2 = HashKeyValue(KeyValue::new("key", std::f64::NAN));
        assert_eq!(hash_helper(&kv1), hash_helper(&kv2));

        let kv1 = HashKeyValue(KeyValue::new("key", std::f64::INFINITY));
        let kv2 = HashKeyValue(KeyValue::new("key", std::f64::INFINITY));
        assert_eq!(hash_helper(&kv1), hash_helper(&kv2));
    }

    fn hash_helper<T: Hash>(item: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        hasher.finish()
    }
}
