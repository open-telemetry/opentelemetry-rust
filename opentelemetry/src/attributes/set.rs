use once_cell::sync::Lazy;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::sync::Arc;
use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use crate::{Array, Key, KeyValue, Value};
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
        match self.0.key.cmp(&other.0.key) {
            Ordering::Equal => match type_order(&self.0.value).cmp(&type_order(&other.0.value)) {
                Ordering::Equal => match (&self.0.value, &other.0.value) {
                    (Value::F64(f), Value::F64(of)) => OrderedFloat(*f).cmp(&OrderedFloat(*of)),
                    (Value::Array(Array::Bool(b)), Value::Array(Array::Bool(ob))) => b.cmp(ob),
                    (Value::Array(Array::I64(i)), Value::Array(Array::I64(oi))) => i.cmp(oi),
                    (Value::Array(Array::String(s)), Value::Array(Array::String(os))) => s.cmp(os),
                    (Value::Array(Array::F64(f)), Value::Array(Array::F64(of))) => {
                        match f.len().cmp(&of.len()) {
                            Ordering::Equal => f
                                .iter()
                                .map(|x| OrderedFloat(*x))
                                .collect::<Vec<_>>()
                                .cmp(&of.iter().map(|x| OrderedFloat(*x)).collect()),
                            other => other,
                        }
                    }
                    (Value::Bool(b), Value::Bool(ob)) => b.cmp(ob),
                    (Value::I64(i), Value::I64(oi)) => i.cmp(oi),
                    (Value::String(s), Value::String(os)) => s.cmp(os),
                    _ => Ordering::Equal,
                },
                other => other, // 2nd order by value types
            },
            other => other, // 1st order by key
        }
    }
}

fn type_order(v: &Value) -> u8 {
    match v {
        Value::Bool(_) => 1,
        Value::I64(_) => 2,
        Value::F64(_) => 3,
        Value::String(_) => 4,
        Value::Array(a) => match a {
            Array::Bool(_) => 5,
            Array::I64(_) => 6,
            Array::F64(_) => 7,
            Array::String(_) => 8,
        },
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

static EMPTY_SET: Lazy<Arc<InternalAttributeSet>> =
    Lazy::new(|| Arc::new(InternalAttributeSet::new(Vec::with_capacity(0))));

#[derive(Eq, PartialEq, Debug)]
struct InternalAttributeSet {
    key_values: Vec<HashKeyValue>,
    hash: u64,
}

impl InternalAttributeSet {
    fn new(mut values: Vec<HashKeyValue>) -> Self {
        values.sort_unstable();
        let mut hasher = DefaultHasher::new();
        values.iter().fold(&mut hasher, |mut hasher, item| {
            item.hash(&mut hasher);
            hasher
        });

        InternalAttributeSet {
            key_values: values,
            hash: hasher.finish(),
        }
    }
}

impl<I: IntoIterator<Item = KeyValue>> From<I> for InternalAttributeSet
where
    <I as IntoIterator>::IntoIter: DoubleEndedIterator + ExactSizeIterator,
{
    fn from(value: I) -> Self {
        let iter = value.into_iter();
        let mut seen_keys = HashSet::with_capacity(iter.len());
        let vec = iter
            .into_iter()
            .rev()
            .filter_map(|kv| {
                if seen_keys.contains(&kv.key) {
                    None
                } else {
                    seen_keys.insert(kv.key.clone());
                    Some(HashKeyValue(kv))
                }
            })
            .collect::<Vec<_>>();

        InternalAttributeSet::new(vec)
    }
}

impl Hash for InternalAttributeSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash)
    }
}

/// Trait declaring that a type can be converted into a `KeyValue`
pub trait ToKeyValue {
    /// Create a `KeyValue` from the current instance.
    fn to_key_value(self) -> KeyValue;
}

impl ToKeyValue for KeyValue {
    fn to_key_value(self) -> KeyValue {
        self
    }
}

impl ToKeyValue for &KeyValue {
    fn to_key_value(self) -> KeyValue {
        self.clone()
    }
}

/// A unique set of attributes that can be used as instrument identifiers.
///
/// Cloning of an attribute set is cheap, as all clones share a reference to the underlying
/// attribute data.
///
/// This must implement [Hash], [PartialEq], and [Eq] so it may be used as
/// HashMap keys and other de-duplication methods.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct AttributeSet(Arc<InternalAttributeSet>);

impl<KV, I> From<I> for AttributeSet
where
    KV: ToKeyValue,
    I: IntoIterator<Item = KV>,
    <I as IntoIterator>::IntoIter: DoubleEndedIterator + ExactSizeIterator,
{
    fn from(values: I) -> Self {
        AttributeSet(Arc::new(InternalAttributeSet::from(
            values.into_iter().map(ToKeyValue::to_key_value),
        )))
    }
}

impl AttributeSet {
    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        self.0.key_values.len()
    }

    /// Returns `true` if the set contains no elements.
    pub fn is_empty(&self) -> bool {
        self.0.key_values.is_empty()
    }

    /// Creates a new attribute set that retains only the attributes specified by the predicate.
    pub fn clone_with<F>(&self, f: F) -> AttributeSet
    where
        F: Fn(&KeyValue) -> bool,
    {
        let key_values = self
            .0
            .key_values
            .iter()
            .filter(|kv| f(&kv.0))
            .cloned()
            .collect::<Vec<_>>();

        AttributeSet(Arc::new(InternalAttributeSet::new(key_values)))
    }

    /// Iterate over key value pairs in the set
    pub fn iter(&self) -> impl Iterator<Item = (&Key, &Value)> {
        self.0.key_values.iter().map(|kv| (&kv.0.key, &kv.0.value))
    }
}

impl Default for AttributeSet {
    fn default() -> Self {
        AttributeSet(EMPTY_SET.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StringValue;

    #[test]
    fn can_create_attribute_set_from_array() {
        let array = [KeyValue::new("key1", "value1"), KeyValue::new("key2", 3)];

        let set = AttributeSet::from(&array);
        let mut kvs = set.iter().collect::<Vec<_>>();

        assert_eq!(kvs.len(), 2, "Incorrect number of attributes");

        kvs.sort_by(|kv1, kv2| kv1.0.cmp(kv2.0));
        assert_eq!(kvs[0].0, &Key::from("key1"), "Unexpected first key");
        assert_eq!(
            kvs[0].1,
            &Value::String("value1".into()),
            "Unexpected first value"
        );
        assert_eq!(kvs[1].0, &Key::from("key2"), "Unexpected second key");
        assert_eq!(kvs[1].1, &Value::I64(3), "Unexpected second value");
    }

    #[test]
    fn can_create_attribute_set_from_owned_vec() {
        let vec = vec![KeyValue::new("key1", "value1"), KeyValue::new("key2", 3)];

        let set = AttributeSet::from(vec);
        let mut kvs = set.iter().collect::<Vec<_>>();

        assert_eq!(kvs.len(), 2, "Incorrect number of attributes");

        kvs.sort_by(|kv1, kv2| kv1.0.cmp(kv2.0));
        assert_eq!(kvs[0].0, &Key::from("key1"), "Unexpected first key");
        assert_eq!(
            kvs[0].1,
            &Value::String("value1".into()),
            "Unexpected first value"
        );
        assert_eq!(kvs[1].0, &Key::from("key2"), "Unexpected second key");
        assert_eq!(kvs[1].1, &Value::I64(3), "Unexpected second value");
    }

    #[test]
    fn two_sets_with_same_key_values_in_different_orders_are_equal() {
        let array1 = [
            KeyValue::new("key1", "value1"),
            KeyValue::new("key2", 3),
            KeyValue::new("key3", Value::Array(Array::Bool(vec![true]))),
            KeyValue::new("key4", Value::Array(Array::F64(vec![1.5]))),
            KeyValue::new("key5", Value::Array(Array::I64(vec![15]))),
            KeyValue::new(
                "key6",
                Value::Array(Array::String(vec![StringValue::from("test")])),
            ),
        ];

        let array2 = [
            KeyValue::new(
                "key6",
                Value::Array(Array::String(vec![StringValue::from("test")])),
            ),
            KeyValue::new("key1", "value1"),
            KeyValue::new("key3", Value::Array(Array::Bool(vec![true]))),
            KeyValue::new("key4", Value::Array(Array::F64(vec![1.5]))),
            KeyValue::new("key5", Value::Array(Array::I64(vec![15]))),
            KeyValue::new("key2", 3),
        ];

        let set1 = AttributeSet::from(&array1);
        let set2 = AttributeSet::from(&array2);

        assert_eq!(set1, set2);
    }

    #[test]
    fn two_sets_with_same_key_values_in_different_orders_have_same_hash() {
        let array1 = [
            KeyValue::new("key1", "value1"),
            KeyValue::new("key2", 3),
            KeyValue::new("key3", Value::Array(Array::Bool(vec![true]))),
            KeyValue::new("key4", Value::Array(Array::F64(vec![1.5]))),
            KeyValue::new("key5", Value::Array(Array::I64(vec![15]))),
            KeyValue::new(
                "key6",
                Value::Array(Array::String(vec![StringValue::from("test")])),
            ),
        ];

        let array2 = [
            KeyValue::new(
                "key6",
                Value::Array(Array::String(vec![StringValue::from("test")])),
            ),
            KeyValue::new("key1", "value1"),
            KeyValue::new("key3", Value::Array(Array::Bool(vec![true]))),
            KeyValue::new("key4", Value::Array(Array::F64(vec![1.5]))),
            KeyValue::new("key5", Value::Array(Array::I64(vec![15]))),
            KeyValue::new("key2", 3),
        ];

        let set1 = AttributeSet::from(&array1);
        let set2 = AttributeSet::from(&array2);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        set1.hash(&mut hasher1);
        set2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn clone_with_removes_unspecified_key_values() {
        let array = [
            KeyValue::new("key1", "value1"),
            KeyValue::new("key2", 3),
            KeyValue::new("key3", 4),
        ];

        let set = AttributeSet::from(&array);
        let set2 = set.clone_with(|kv| kv.key == Key::new("key2"));

        assert_ne!(set, set2, "Both sets were unexpectedly equal");
        assert_eq!(set2.len(), 1, "Expected only one attribute in new set");

        let kvs = set2.iter().collect::<Vec<_>>();
        assert_eq!(kvs[0].0, &Key::from("key2"), "Unexpected key");
        assert_eq!(kvs[0].1, &Value::I64(3), "Unexpected value");
    }

    #[test]
    fn len_returns_accurate_value() {
        let array = [KeyValue::new("key1", "value1"), KeyValue::new("key2", 3)];

        let set = AttributeSet::from(&array);
        let kvs = set.iter().collect::<Vec<_>>();

        assert_eq!(set.len(), kvs.len());
    }

    #[test]
    fn empty_when_no_attributes_provided() {
        let set = AttributeSet::from(&[]);
        assert!(set.is_empty());
    }

    #[test]
    fn default_set_has_no_attributes() {
        let set = AttributeSet::default();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn last_key_wins_for_deduplication() {
        let array = [KeyValue::new("key1", "value1"), KeyValue::new("key1", 3)];

        let set = AttributeSet::from(&array);
        let kvs = set.iter().collect::<Vec<_>>();

        assert_eq!(set.len(), 1, "Expected only a single key value pair");
        assert_eq!(kvs[0].0, &Key::new("key1"), "Unexpected key");
        assert_eq!(kvs[0].1, &Value::I64(3), "Unexpected value");
    }
}
