use crate::api::{Context, Key, KeyValue, Value};
use std::collections::{hash_map, HashMap};
use std::iter::FromIterator;

lazy_static::lazy_static! {
    static ref DEFAULT_BAGGAGE: Baggage = Baggage::default();
}

/// A set of name/value pairs describing user-defined properties across systems.
#[derive(Debug, Default)]
pub struct Baggage {
    inner: HashMap<Key, Value>,
}

impl Baggage {
    /// Creates an empty `Baggage`.
    pub fn new() -> Self {
        Baggage {
            inner: HashMap::default(),
        }
    }

    /// Returns a reference to the value associated with a given name
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::api::{Baggage, Value};
    ///
    /// let mut cc = Baggage::new();
    /// let _ = cc.insert("my-name", "my-value");
    ///
    /// assert_eq!(cc.get("my-name"), Some(&Value::String("my-value".to_string())))
    /// ```
    pub fn get<T: Into<Key>>(&self, key: T) -> Option<&Value> {
        self.inner.get(&key.into())
    }

    /// Inserts a name-value pair into the baggage.
    ///
    /// If the name was not present, [`None`] is returned. If the name was present,
    /// the value is updated, and the old value is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::api::{Baggage, Value};
    ///
    /// let mut cc = Baggage::new();
    /// let _ = cc.insert("my-name", "my-value");
    ///
    /// assert_eq!(cc.get("my-name"), Some(&Value::String("my-value".to_string())))
    /// ```
    pub fn insert<K, V>(&mut self, key: K, value: V) -> Option<Value>
    where
        K: Into<Key>,
        V: Into<Value>,
    {
        self.inner.insert(key.into(), value.into())
    }

    /// Removes a name from the baggage, returning the value
    /// corresponding to the name if the pair was previously in the map.
    pub fn remove<K: Into<Key>>(&mut self, key: K) -> Option<Value> {
        self.inner.remove(&key.into())
    }

    /// Returns the number of attributes for this baggage
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the baggage contains no items.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Gets an iterator over the baggage items, sorted by name.
    pub fn iter(&self) -> Iter<'_> {
        self.into_iter()
    }
}

/// An iterator over the entries of a `Baggage`.
#[derive(Debug)]
pub struct Iter<'a>(hash_map::Iter<'a, Key, Value>);
impl<'a> Iterator for Iter<'a> {
    type Item = (&'a Key, &'a Value);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> IntoIterator for &'a Baggage {
    type Item = (&'a Key, &'a Value);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.inner.iter())
    }
}

impl FromIterator<(Key, Value)> for Baggage {
    fn from_iter<I: IntoIterator<Item = (Key, Value)>>(iter: I) -> Self {
        Baggage {
            inner: iter.into_iter().collect(),
        }
    }
}

impl FromIterator<KeyValue> for Baggage {
    fn from_iter<I: IntoIterator<Item = KeyValue>>(iter: I) -> Self {
        Baggage {
            inner: iter.into_iter().map(|kv| (kv.key, kv.value)).collect(),
        }
    }
}

/// Methods for sorting and retrieving baggage data in a context.
pub trait BaggageExt {
    /// Returns a clone of the current context with the included name / value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::api::{BaggageExt, Context, KeyValue, Value};
    ///
    /// let cx = Context::current_with_baggage(vec![KeyValue::new("my-name", "my-value")]);
    ///
    /// assert_eq!(
    ///     cx.baggage().get("my-name"),
    ///     Some(&Value::String("my-value".to_string())),
    /// )
    /// ```
    fn current_with_baggage<T: IntoIterator<Item = KeyValue>>(baggage: T) -> Self;

    /// Returns a clone of the given context with the included name / value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::api::{BaggageExt, Context, KeyValue, Value};
    ///
    /// let some_context = Context::current();
    /// let cx = some_context.with_baggage(vec![KeyValue::new("my-name", "my-value")]);
    ///
    /// assert_eq!(
    ///     cx.baggage().get("my-name"),
    ///     Some(&Value::String("my-value".to_string())),
    /// )
    /// ```
    fn with_baggage<T: IntoIterator<Item = KeyValue>>(&self, baggage: T) -> Self;

    /// Returns a clone of the given context with the included name / value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::api::{BaggageExt, Context, KeyValue, Value};
    ///
    /// let cx = Context::current().with_cleared_baggage();
    ///
    /// assert_eq!(cx.baggage().len(), 0);
    /// ```
    fn with_cleared_baggage(&self) -> Self;

    /// Returns a reference to this context's baggage, or the default
    /// empty baggage if none has been set.
    fn baggage(&self) -> &Baggage;
}
impl BaggageExt for Context {
    fn current_with_baggage<T: IntoIterator<Item = KeyValue>>(kvs: T) -> Self {
        Context::current().with_baggage(kvs)
    }

    fn with_baggage<T: IntoIterator<Item = KeyValue>>(&self, kvs: T) -> Self {
        let merged: Baggage = self
            .baggage()
            .iter()
            .map(|(key, value)| KeyValue::new(key.clone(), value.clone()))
            .chain(kvs.into_iter())
            .collect();

        self.with_value(merged)
    }

    fn with_cleared_baggage(&self) -> Self {
        self.with_value(Baggage::new())
    }

    fn baggage(&self) -> &Baggage {
        self.get::<Baggage>().unwrap_or_else(|| &DEFAULT_BAGGAGE)
    }
}
