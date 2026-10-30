//! # OpenTelemetry Propagator interface
//! Cross-cutting concerns send their state to the next process using Propagators, which are defined
//! as objects used to read and write context data to and from messages exchanged by the applications.
//!
//! `Propagator`s leverage the [`Context`] to inject and extract data for each cross-cutting concern,
//! such as `TraceContext` and [`Baggage`].
//!
//! The Propagators API is expected to be leveraged by users writing instrumentation libraries.
//!
//! Currently, the following `Propagator` types are supported:
//! -  [`TextMapPropagator`], inject values into and extracts values from carriers as string key/value pairs
//!
//! A binary Propagator type will be added in
//! the future, See [tracking issues](https://github.com/open-telemetry/opentelemetry-specification/issues/437)).
//!
//! `Propagator`s uses [`Injector`] and [`Extractor`] to read and write context data to and from messages.
//! Each specific Propagator type defines its expected carrier type, such as a string map or a byte array.
//!
//! With the `experimental_propagation_env_carrier` feature, the `env` module provides
//! carriers that propagate context to and from child processes through environment variables.
//!
//! [`Baggage`]: crate::baggage::Baggage
//! [`Context`]: crate::Context

use std::borrow::Cow;
use std::collections::HashMap;

pub mod composite;
#[cfg(feature = "experimental_propagation_env_carrier")]
#[cfg_attr(docsrs, doc(cfg(feature = "experimental_propagation_env_carrier")))]
pub mod env;
pub mod text_map_propagator;

pub use composite::TextMapCompositePropagator;
pub use text_map_propagator::TextMapPropagator;

/// Injector provides an interface for adding fields from an underlying struct like `HashMap`
pub trait Injector {
    /// Add a key and value to the underlying data.
    fn set(&mut self, key: &str, value: String);

    #[allow(unused_variables)]
    /// Hint to reserve capacity for at least `additional` more entries to be inserted.
    fn reserve(&mut self, additional: usize) {}
}

/// Extractor provides an interface for removing fields from an underlying struct like `HashMap`
pub trait Extractor {
    /// Get a value from a key from the underlying data.
    ///
    /// Carriers that hold the value return [`Cow::Borrowed`], carriers that have
    /// to produce it on demand return [`Cow::Owned`].
    fn get(&self, key: &str) -> Option<Cow<'_, str>>;

    /// Collect all the keys from the underlying data.
    fn keys(&self) -> Vec<Cow<'_, str>>;

    /// Get all values from a key from the underlying data.
    fn get_all(&self, key: &str) -> Option<Vec<Cow<'_, str>>> {
        self.get(key).map(|value| vec![value])
    }
}

impl<S: std::hash::BuildHasher> Injector for HashMap<String, String, S> {
    /// Set a key and value in the HashMap.
    fn set(&mut self, key: &str, value: String) {
        self.insert(key.to_lowercase(), value);
    }

    /// Reserves capacity for at least `additional` more entries to be inserted.
    fn reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }
}

impl<S: std::hash::BuildHasher> Extractor for HashMap<String, String, S> {
    /// Get a value for a key from the HashMap.
    fn get(&self, key: &str) -> Option<Cow<'_, str>> {
        self.get(&key.to_lowercase())
            .map(|v| Cow::Borrowed(v.as_str()))
    }

    /// Collect all the keys from the HashMap.
    fn keys(&self) -> Vec<Cow<'_, str>> {
        self.keys()
            .map(|k| Cow::Borrowed(k.as_str()))
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_map_get() {
        let mut carrier = HashMap::new();
        carrier.set("headerName", "value".to_string());

        assert_eq!(
            Extractor::get(&carrier, "HEADERNAME").as_deref(),
            Some("value"),
            "case insensitive extraction"
        );
    }

    #[test]
    fn hash_map_get_all() {
        let mut carrier = HashMap::new();
        carrier.set("headerName", "value".to_string());

        assert_eq!(
            Extractor::get_all(&carrier, "HEADERNAME"),
            Some(vec![Cow::Borrowed("value")]),
            "case insensitive get_all extraction"
        );
    }

    #[test]
    fn hash_map_get_all_missing_key() {
        let mut carrier = HashMap::new();
        carrier.set("headerName", "value".to_string());

        assert_eq!(
            Extractor::get_all(&carrier, "missing_key"),
            None,
            "case insensitive get_all extraction"
        );
    }

    #[test]
    fn hash_map_keys() {
        let mut carrier = HashMap::new();
        carrier.set("headerName1", "value1".to_string());
        carrier.set("headerName2", "value2".to_string());

        let got = Extractor::keys(&carrier);
        assert_eq!(got.len(), 2);
        assert!(got.contains(&Cow::Borrowed("headername1")));
        assert!(got.contains(&Cow::Borrowed("headername2")));
    }

    #[test]
    fn hash_map_injector_reserve() {
        let mut carrier = HashMap::new();

        // Test that reserve doesn't panic and works correctly
        Injector::reserve(&mut carrier, 10);

        // Verify the HashMap still works after reserve
        Injector::set(&mut carrier, "test_key", "test_value".to_string());
        assert_eq!(
            Extractor::get(&carrier, "test_key").as_deref(),
            Some("test_value")
        );

        // Test reserve with zero capacity
        Injector::reserve(&mut carrier, 0);
        Injector::set(&mut carrier, "another_key", "another_value".to_string());
        assert_eq!(
            Extractor::get(&carrier, "another_key").as_deref(),
            Some("another_value")
        );

        // Test that capacity is actually reserved (at least the requested amount)
        let mut new_carrier = HashMap::new();
        Injector::reserve(&mut new_carrier, 5);
        let initial_capacity = new_carrier.capacity();

        // Add some elements and verify capacity doesn't decrease
        for i in 0..3 {
            Injector::set(
                &mut new_carrier,
                &format!("key{}", i),
                format!("value{}", i),
            );
        }

        assert!(new_carrier.capacity() >= initial_capacity);
        assert!(new_carrier.capacity() >= 5);
    }

    #[test]
    fn injector_reserve() {
        // Test to have full line coverage of default method
        struct TestInjector();
        impl Injector for TestInjector {
            fn set(&mut self, _key: &str, _value: String) {}
        }
        let mut test_injector = TestInjector();
        Injector::reserve(&mut test_injector, 4711);
        Injector::set(&mut test_injector, "key", "value".to_string());
    }
}
