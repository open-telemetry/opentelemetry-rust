//! # OpenTelemetry Baggage API
//!
//! Baggage is used to annotate telemetry, adding context and
//! information to metrics, traces, and logs. It is an abstract data type
//! represented by a set of name/value pairs describing user-defined properties.
//! Each name in a [`Baggage`] is associated with exactly one value.
//! `Baggage`s are serialized according to the editor's draft of
//! the [W3C Baggage] specification.
//!
//! [`CorrelationContext`]: struct.Baggage.html
//! [W3C Baggage]: https://w3c.github.io/baggage/
//!
//! # Examples
//!
//! ```
//! use opentelemetry::api::{
//!     BaggageExt, BaggagePropagator, TextMapFormat, Key
//! };
//! use std::collections::HashMap;
//!
//! // Example baggage value passed in externally via http headers
//! let mut headers = HashMap::new();
//! headers.insert("baggage".to_string(), "user_id=1".to_string());
//!
//! let propagator = BaggagePropagator::new();
//! // can extract from any type that impls `Extractor`, usually an HTTP header map
//! let cx = propagator.extract(&headers);
//!
//! // Iterate over extracted name / value pairs
//! for (name, value) in cx.baggage() {
//!   // ...
//! }
//!
//! // Add new baggage
//! let cx_with_additions = cx.with_baggage(vec![Key::new("server_id").u64(42)]);
//!
//! // Inject aggage into http request
//! propagator.inject_context(&cx_with_additions, &mut headers);
//!
//! let header_value = headers.get("baggage").expect("header is injected");
//! assert!(header_value.contains("user_id=1"), "still contains previous name / value");
//! assert!(header_value.contains("server_id=42"), "contains new name / value pair");
//! ```
use crate::api;
use std::collections::{hash_map, HashMap};
use std::iter::FromIterator;

mod propagation;

#[cfg(feature = "trace")]
pub use propagation::{BaggageExt, BaggagePropagator};

/// A set of name/value pairs describing user-defined properties across systems.
#[derive(Debug, Default)]
pub struct Baggage {
    inner: HashMap<api::Key, api::Value>,
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
    pub fn get<T: Into<api::Key>>(&self, key: T) -> Option<&api::Value> {
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
    pub fn insert<K, V>(&mut self, key: K, value: V) -> Option<api::Value>
    where
        K: Into<api::Key>,
        V: Into<api::Value>,
    {
        self.inner.insert(key.into(), value.into())
    }

    /// Removes a name from the baggage, returning the value
    /// corresponding to the name if the pair was previously in the map.
    pub fn remove<K: Into<api::Key>>(&mut self, key: K) -> Option<api::Value> {
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
pub struct Iter<'a>(hash_map::Iter<'a, api::Key, api::Value>);
impl<'a> Iterator for Iter<'a> {
    type Item = (&'a api::Key, &'a api::Value);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> IntoIterator for &'a Baggage {
    type Item = (&'a api::Key, &'a api::Value);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.inner.iter())
    }
}

impl FromIterator<(api::Key, api::Value)> for Baggage {
    fn from_iter<I: IntoIterator<Item = (api::Key, api::Value)>>(iter: I) -> Self {
        Baggage {
            inner: iter.into_iter().collect(),
        }
    }
}

impl FromIterator<api::KeyValue> for Baggage {
    fn from_iter<I: IntoIterator<Item = api::KeyValue>>(iter: I) -> Self {
        Baggage {
            inner: iter.into_iter().map(|kv| (kv.key, kv.value)).collect(),
        }
    }
}
