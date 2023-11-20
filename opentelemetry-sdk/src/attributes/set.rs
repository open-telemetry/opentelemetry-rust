use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};
use std::collections::hash_map::DefaultHasher;

use opentelemetry::{Array, Key, KeyValue, Value};
use ordered_float::OrderedFloat;

use crate::Resource;

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

/// A unique set of attributes that can be used as instrument identifiers.
///
/// This must implement [Hash], [PartialEq], and [Eq] so it may be used as
/// HashMap keys and other de-duplication methods.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct AttributeSet(Vec<HashKeyValue>, u64);

impl From<&[KeyValue]> for AttributeSet {
    fn from(values: &[KeyValue]) -> Self {
        let mut vec = values
            .iter()
            .map(|k| HashKeyValue(k.clone()))
            .collect::<Vec<_>>();
        vec.sort_by(|a, b| a.0.key.cmp(&b.0.key));
        vec.dedup_by(|a, b| a.0.key.eq(&b.0.key));

        let mut hasher = DefaultHasher::new();
        for value in &vec {
            value.hash(&mut hasher);
        }

        AttributeSet(vec, hasher.finish())
    }
}

impl From<&Resource> for AttributeSet {
    fn from(values: &Resource) -> Self {
        let vec = values
            .iter()
            .map(|(key, value)| HashKeyValue(KeyValue::new(key.clone(), value.clone())))
            .collect::<Vec<_>>();

        let mut hasher = DefaultHasher::new();
        for value in &vec {
            value.hash(&mut hasher);
        }
        AttributeSet(vec, hasher.finish())
    }
}

impl AttributeSet {
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
        self.0.retain(|kv| f(&kv.0))
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
