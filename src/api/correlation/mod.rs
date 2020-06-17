//! # OpenTelemetry Correlation Context API
//!
//! A Correlation Context is used to annotate telemetry, adding context and
//! information to metrics, traces, and logs. It is an abstract data type
//! represented by a set of name/value pairs describing user-defined properties.
//! Each name in a [`CorrelationContext`] is associated with exactly one value.
//! `CorrelationContext`s are serialized according to the editor's draft of
//! the [W3C Correlation Context] specification.
//!
//! [`CorrelationContext`]: struct.CorrelationContext.html
//! [W3C Correlation Context]: https://w3c.github.io/correlation-context/
//!
//! # Examples
//!
//! ```
//! use opentelemetry::api::{
//!     CorrelationContextExt, CorrelationContextPropagator, HttpTextFormat, Key
//! };
//! use std::collections::HashMap;
//!
//! // Example correlation value passed in externally via http headers
//! let mut headers = HashMap::new();
//! headers.insert("Correlation-Context".to_string(), "user_id=1".to_string());
//!
//! let propagator = CorrelationContextPropagator::new();
//! // can extract from any type that impls `Carrier`, usually an HTTP header map
//! let cx = propagator.extract(&headers);
//!
//! // Iterate over extracted name / value pairs
//! for (name, value) in cx.correlation_context() {
//!   // ...
//! }
//!
//! // Add new correlations
//! let cx_with_additions = cx.with_correlations(vec![Key::new("server_id").u64(42)]);
//!
//! // Inject correlations into http request
//! propagator.inject_context(&cx_with_additions, &mut headers);
//!
//! let header_value = headers.get("Correlation-Context").expect("header is injected");
//! assert!(header_value.contains("user_id=1"), "still contains previous name / value");
//! assert!(header_value.contains("server_id=42"), "contains new name / value pair");
//! ```
use crate::api;
use std::collections::{hash_map, HashMap};
use std::iter::FromIterator;

mod propagation;

#[cfg(feature = "trace")]
pub use propagation::{CorrelationContextExt, CorrelationContextPropagator};

/// A set of name/value pairs describing user-defined properties across systems.
#[derive(Debug, Default)]
pub struct CorrelationContext {
    inner: HashMap<api::Key, api::Value>,
}

impl CorrelationContext {
    /// Creates an empty `CorrelationContext`.
    pub fn new() -> Self {
        CorrelationContext {
            inner: HashMap::default(),
        }
    }

    /// Returns a reference to the value associated with a given name
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::api::{CorrelationContext, Value};
    ///
    /// let mut cc = CorrelationContext::new();
    /// let _ = cc.insert("my-name", "my-value");
    ///
    /// assert_eq!(cc.get("my-name"), Some(&Value::String("my-value".to_string())))
    /// ```
    pub fn get<T: Into<api::Key>>(&self, key: T) -> Option<&api::Value> {
        self.inner.get(&key.into())
    }

    /// Inserts a name-value pair into the correlation context.
    ///
    /// If the name was not present, [`None`] is returned. If the name was present,
    /// the value is updated, and the old value is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::api::{CorrelationContext, Value};
    ///
    /// let mut cc = CorrelationContext::new();
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

    /// Removes a name from the correlation context, returning the value
    /// corresponding to the name if the pair was previously in the map.
    pub fn remove<K: Into<api::Key>>(&mut self, key: K) -> Option<api::Value> {
        self.inner.remove(&key.into())
    }

    /// Returns the number of attributes for this correlation context
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the correlation context contains no items.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Gets an iterator over the correlation context items, sorted by name.
    pub fn iter(&self) -> Iter {
        self.into_iter()
    }
}

/// An iterator over the entries of a `CorrelationContext`.
#[derive(Debug)]
pub struct Iter<'a>(hash_map::Iter<'a, api::Key, api::Value>);
impl<'a> Iterator for Iter<'a> {
    type Item = (&'a api::Key, &'a api::Value);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> IntoIterator for &'a CorrelationContext {
    type Item = (&'a api::Key, &'a api::Value);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.inner.iter())
    }
}

impl FromIterator<(api::Key, api::Value)> for CorrelationContext {
    fn from_iter<I: IntoIterator<Item = (api::Key, api::Value)>>(iter: I) -> Self {
        CorrelationContext {
            inner: iter.into_iter().collect(),
        }
    }
}

impl FromIterator<api::KeyValue> for CorrelationContext {
    fn from_iter<I: IntoIterator<Item = api::KeyValue>>(iter: I) -> Self {
        CorrelationContext {
            inner: iter.into_iter().map(|kv| (kv.key, kv.value)).collect(),
        }
    }
}
