use crate::api::core::{KeyValueMetadata, Metadata};
use crate::api::{Context, Key, KeyValue, Value};
use std::collections::{hash_map, HashMap};
use std::iter::FromIterator;

lazy_static::lazy_static! {
    static ref DEFAULT_BAGGAGE: Baggage = Baggage::default();
}

/// A set of name/value pairs describing user-defined properties across systems.
#[derive(Debug, Default)]
pub struct Baggage {
    inner: HashMap<Key, (Value, Metadata)>,
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
        self.inner.get(&key.into()).map(|(value, _metadata)| value)
    }

    /// Returns a reference to the value and metadata associated with a given name
    ///
    /// # Examples
    /// ```
    /// use opentelemetry::api::{Baggage, Value};
    ///
    /// let mut cc = Baggage::new();
    /// let _ = cc.insert("my-name", "my-value");
    ///
    /// // By default, the metadata is empty
    /// assert_eq!(cc.get_with_metadata("my-name"), Some(&(Value::String("my-value".to_string()), "".into())))
    /// ```
    pub fn get_with_metadata<T: Into<Key>>(&self, key: T) -> Option<&(Value, Metadata)> {
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
        self.inner
            .insert(key.into(), (value.into(), Metadata::default()))
            .map(|pair| pair.0)
    }

    /// Inserts a name-value pair into the baggage.
    ///
    /// Same with `insert`, if the name was not present, [`None`] will be returned.
    /// If the name is present, the old value and metadata will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::api::{Baggage, Value};
    ///
    /// let mut cc = Baggage::new();
    /// let _ = cc.insert_with_metadata("my-name", "my-value", "test");
    ///
    /// assert_eq!(cc.get_with_metadata("my-name"), Some(&(Value::String("my-value".to_string()), "test".into())))
    /// ```
    pub fn insert_with_metadata<K, V, S>(
        &mut self,
        key: K,
        value: V,
        metadata: S,
    ) -> Option<(Value, Metadata)>
    where
        K: Into<Key>,
        V: Into<Value>,
        S: Into<Metadata>,
    {
        self.inner
            .insert(key.into(), (value.into(), metadata.into()))
    }

    /// Removes a name from the baggage, returning the value
    /// corresponding to the name if the pair was previously in the map.
    pub fn remove<K: Into<Key>>(&mut self, key: K) -> Option<(Value, Metadata)> {
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
    pub fn iter(&self) -> Iter {
        self.into_iter()
    }
}

/// An iterator over the entries of a `Baggage`.
#[derive(Debug)]
pub struct Iter<'a>(hash_map::Iter<'a, Key, (Value, Metadata)>);

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a Key, &'a (Value, Metadata));

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> IntoIterator for &'a Baggage {
    type Item = (&'a Key, &'a (Value, Metadata));
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.inner.iter())
    }
}

impl FromIterator<(Key, (Value, Metadata))> for Baggage {
    fn from_iter<I: IntoIterator<Item = (Key, (Value, Metadata))>>(iter: I) -> Self {
        Baggage {
            inner: iter.into_iter().collect(),
        }
    }
}

impl FromIterator<KeyValue> for Baggage {
    fn from_iter<I: IntoIterator<Item = KeyValue>>(iter: I) -> Self {
        Baggage {
            inner: iter
                .into_iter()
                .map(|kv| (kv.key, (kv.value, Metadata::default())))
                .collect(),
        }
    }
}

impl FromIterator<KeyValueMetadata> for Baggage {
    fn from_iter<I: IntoIterator<Item = KeyValueMetadata>>(iter: I) -> Self {
        Baggage {
            inner: iter
                .into_iter()
                .map(|kvm| (kvm.key, (kvm.value, kvm.metadata)))
                .collect(),
        }
    }
}

/// Methods for sorting and retrieving baggage data in a context.
pub trait BaggageExt {
    /// Returns a clone of the given context with the included name / value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::api::{Context, KeyValue, Value, BaggageExt};
    ///
    /// let some_context = Context::current();
    /// let cx = some_context.with_baggage(vec![KeyValue::new("my-name", "my-value")]);
    ///
    /// assert_eq!(
    ///     cx.baggage().get("my-name"),
    ///     Some(&Value::String("my-value".to_string())),
    /// )
    /// ```
    fn with_baggage<T: IntoIterator<Item = I>, I: Into<KeyValueMetadata>>(
        &self,
        baggage: T,
    ) -> Self;

    /// Returns a clone of the current context with the included name / value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::api::{Context, KeyValue, Value, BaggageExt};
    ///
    /// let cx = Context::current_with_baggage(vec![KeyValue::new("my-name", "my-value")]);
    ///
    /// assert_eq!(
    ///     cx.baggage().get("my-name"),
    ///     Some(&Value::String("my-value".to_string())),
    /// )
    /// ```
    fn current_with_baggage<T: IntoIterator<Item = I>, I: Into<KeyValueMetadata>>(
        baggage: T,
    ) -> Self;

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
    fn with_baggage<T: IntoIterator<Item = I>, I: Into<KeyValueMetadata>>(&self, baggage: T) -> Self {
        let merged: Baggage = self
            .baggage()
            .iter()
            .map(|(key, (value, metadata))| {
                KeyValueMetadata::new(key.clone(), value.clone(), metadata.clone())
            })
            .chain(baggage.into_iter().map(|kv| kv.into()))
            .collect();

        self.with_value(merged)
    }

    fn current_with_baggage<T: IntoIterator<Item = I>, I: Into<KeyValueMetadata>>(kvs: T) -> Self {
        Context::current().with_baggage(kvs)
    }

    fn with_cleared_baggage(&self) -> Self {
        self.with_value(Baggage::new())
    }

    fn baggage(&self) -> &Baggage {
        self.get::<Baggage>().unwrap_or_else(|| &DEFAULT_BAGGAGE)
    }
}
