use std::hash::{Hash, Hasher};

use opentelemetry::{Key, KeyValue, Value};
use rustc_hash::FxHasher;

/// A unique set of attributes that can be used as instrument identifiers.
///
/// This must implement [Hash], [PartialEq], and [Eq] so it may be used as
/// HashMap keys and other de-duplication methods.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub(crate) struct AttributeSet(Vec<KeyValue>, u64);

impl From<&[KeyValue]> for AttributeSet {
    fn from(values: &[KeyValue]) -> Self {
        let mut vec = Vec::from_iter(values.into_iter().cloned());
        vec.sort_by(|a, b| a.key.cmp(&b.key));

        // we cannot use vec.dedup_by because it will remove last duplicate not first
        if vec.len() > 1 {
            let mut i = vec.len() - 1;
            while i != 0 {
                let is_same = unsafe { vec.get_unchecked(i - 1).key == vec.get_unchecked(i).key };
                if is_same {
                    vec.remove(i - 1);
                }
                i -= 1;
            }
        }

        let hash = calculate_hash(&vec);
        AttributeSet(vec, hash)
    }
}

fn calculate_hash(values: &[KeyValue]) -> u64 {
    let mut hasher = FxHasher::default();
    values.iter().fold(&mut hasher, |mut hasher, item| {
        item.hash(&mut hasher);
        hasher
    });
    hasher.finish()
}

impl AttributeSet {
    /// Iterate over key value pairs in the set
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&Key, &Value)> {
        self.0.iter().map(|kv| (&kv.key, &kv.value))
    }

    pub(crate) fn into_inner(self) -> Vec<KeyValue> {
        self.0
    }

    pub(crate) fn as_ref(&self) -> &Vec<KeyValue> {
        &self.0
    }
}

impl Hash for AttributeSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.1)
    }
}
