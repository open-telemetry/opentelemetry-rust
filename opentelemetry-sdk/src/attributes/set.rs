use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use opentelemetry::{Key, KeyValue, Value};

/// A unique set of attributes that can be used as instrument identifiers.
///
/// This must implement [Hash], [PartialEq], and [Eq] so it may be used as
/// HashMap keys and other de-duplication methods.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct AttributeSet(Vec<KeyValue>, u64);

impl From<&[KeyValue]> for AttributeSet {
    fn from(values: &[KeyValue]) -> Self {
        let mut seen_keys = HashSet::with_capacity(values.len());
        let vec = values
            .iter()
            .rev()
            .filter_map(|kv| {
                if seen_keys.insert(kv.key.clone()) {
                    Some(kv.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        AttributeSet::new(vec)
    }
}

fn calculate_hash(values: &[KeyValue]) -> u64 {
    let mut hasher = DefaultHasher::new();
    values.iter().fold(&mut hasher, |mut hasher, item| {
        item.hash(&mut hasher);
        hasher
    });
    hasher.finish()
}

impl AttributeSet {
    fn new(mut values: Vec<KeyValue>) -> Self {
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
        self.0.retain(|kv| f(&kv));

        // Recalculate the hash as elements are changed.
        self.1 = calculate_hash(&self.0);
    }

    /// Iterate over key value pairs in the set
    pub fn iter(&self) -> impl Iterator<Item = (&Key, &Value)> {
        self.0.iter().map(|kv| (&kv.key, &kv.value))
    }
}

impl Hash for AttributeSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.1)
    }
}
