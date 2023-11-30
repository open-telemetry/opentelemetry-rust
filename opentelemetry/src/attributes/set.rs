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

    fn from_key_values<T: IntoIterator<Item = KeyValue>>(iter: T) -> Self
    where
        <T as IntoIterator>::IntoIter: DoubleEndedIterator + ExactSizeIterator,
    {
        // Note: this doesn't implement `FromIter` because of the additional constraints
        let iter = iter.into_iter();
        let mut seen_keys = HashSet::with_capacity(iter.len());
        let vec = iter
            .into_iter()
            .rev()
            .filter_map(|kv| {
                if seen_keys.insert(kv.key.clone()) {
                    Some(HashKeyValue(kv.clone()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        InternalAttributeSet::new(vec)
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
                if seen_keys.insert(kv.key.clone()) {
                    Some(HashKeyValue(kv.clone()))
                } else {
                    None
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

/// A unique set of attributes that can be used as instrument identifiers.
///
/// This must implement [Hash], [PartialEq], and [Eq] so it may be used as
/// HashMap keys and other de-duplication methods.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct AttributeSet(Arc<InternalAttributeSet>);

impl<I: IntoIterator<Item = KeyValue>> From<I> for AttributeSet
where
    <I as IntoIterator>::IntoIter: DoubleEndedIterator + ExactSizeIterator,
{
    fn from(values: I) -> Self {
        AttributeSet(Arc::new(InternalAttributeSet::from(values)))
    }
}

impl AttributeSet {
    /// Creates an attribute set from an iterator of `KeyValue`s
    pub fn from_key_values<T: IntoIterator<Item = KeyValue>>(iter: T) -> Self
    where
        <T as IntoIterator>::IntoIter: DoubleEndedIterator + ExactSizeIterator,
    {
        AttributeSet(Arc::new(InternalAttributeSet::from_key_values(iter)))
    }

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
