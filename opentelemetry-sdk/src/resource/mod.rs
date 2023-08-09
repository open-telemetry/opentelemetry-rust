//! Representations of entities producing telemetry.
//!
//! A [Resource] is an immutable representation of the entity producing
//! telemetry as attributes. For example, a process producing telemetry that is
//! running in a container on Kubernetes has a Pod name, it is in a namespace
//! and possibly is part of a Deployment which also has a name. All three of
//! these attributes can be included in the `Resource`. Note that there are
//! certain ["standard attributes"] that have prescribed meanings.
//!
//! ["standard attributes"]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/resource/semantic_conventions/README.md
//!
//! # Resource detectors
//!
//! [`ResourceDetector`]s are used to detect resource from runtime or
//! environmental variables. The following are provided by default with this
//! SDK.
//!
//! - [`EnvResourceDetector`] - detect resource from environmental variables.
//! - [`OsResourceDetector`] - detect OS from runtime.
//! - [`ProcessResourceDetector`] - detect process information.
//! - [`TelemetryResourceDetector`] - detect telemetry SDK's information.
mod env;
mod os;
mod process;
mod telemetry;

pub use env::EnvResourceDetector;
pub use env::SdkProvidedResourceDetector;
pub use os::OsResourceDetector;
pub use process::ProcessResourceDetector;
pub use telemetry::TelemetryResourceDetector;

use opentelemetry_api::{Key, KeyValue, Value};
use std::borrow::Cow;
use std::collections::{hash_map, HashMap};
use std::ops::Deref;
use std::time::Duration;

/// An immutable representation of the entity producing telemetry as attributes.
#[derive(Clone, Debug, PartialEq)]
pub struct Resource {
    attrs: HashMap<Key, Value>,
    schema_url: Option<Cow<'static, str>>,
}

impl Default for Resource {
    fn default() -> Self {
        Self::from_detectors(
            Duration::from_secs(0),
            vec![
                Box::new(SdkProvidedResourceDetector),
                Box::new(TelemetryResourceDetector),
                Box::new(EnvResourceDetector::new()),
            ],
        )
    }
}

impl Resource {
    /// Creates an empty resource.
    pub fn empty() -> Self {
        Self {
            attrs: Default::default(),
            schema_url: None,
        }
    }

    /// Create a new `Resource` from key value pairs.
    ///
    /// Values are de-duplicated by key, and the first key-value pair with a non-empty string value
    /// will be retained
    pub fn new<T: IntoIterator<Item = KeyValue>>(kvs: T) -> Self {
        let mut resource = Resource::empty();

        for kv in kvs.into_iter() {
            resource.attrs.insert(kv.key, kv.value);
        }

        resource
    }

    /// Create a new `Resource` from a key value pairs and [schema url].
    ///
    /// Values are de-duplicated by key, and the first key-value pair with a non-empty string value
    /// will be retained.
    ///
    /// schema_url must be a valid URL using HTTP or HTTPS protocol.
    ///
    /// [schema url]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/schemas/overview.md#schema-url
    pub fn from_schema_url<KV, S>(kvs: KV, schema_url: S) -> Self
    where
        KV: IntoIterator<Item = KeyValue>,
        S: Into<Cow<'static, str>>,
    {
        let mut resource = Self::new(kvs);
        resource.schema_url = Some(schema_url.into());
        resource
    }

    /// Create a new `Resource` from resource detectors.
    ///
    /// timeout will be applied to each detector.
    pub fn from_detectors(timeout: Duration, detectors: Vec<Box<dyn ResourceDetector>>) -> Self {
        let mut resource = Resource::empty();
        for detector in detectors {
            let detected_res = detector.detect(timeout);
            for (key, value) in detected_res.into_iter() {
                // using insert instead of merge to avoid clone.
                resource.attrs.insert(key, value);
            }
        }

        resource
    }

    /// Create a new `Resource` by combining two resources.
    ///
    /// ### Key value pairs
    /// Keys from the `other` resource have priority over keys from this resource, even if the
    /// updated value is empty.
    ///
    /// ### [Schema url]
    /// If both of the resource are not empty. Schema url is determined by the following rules, in order:
    /// 1. If this resource has a schema url, it will be used.
    /// 2. If this resource does not have a schema url, and the other resource has a schema url, it will be used.
    /// 3. If both resources have a schema url and it's the same, it will be used.
    /// 4. If both resources have a schema url and it's different, the schema url will be empty.
    /// 5. If both resources do not have a schema url, the schema url will be empty.
    ///
    /// [Schema url]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/schemas/overview.md#schema-url
    pub fn merge<T: Deref<Target = Self>>(&self, other: T) -> Self {
        if self.attrs.is_empty() {
            return other.clone();
        }
        if other.attrs.is_empty() {
            return self.clone();
        }

        let mut resource = Resource::empty();

        // attrs from self take the less priority, even when the new value is empty.
        for (k, v) in self.attrs.iter() {
            resource.attrs.insert(k.clone(), v.clone());
        }
        for (k, v) in other.attrs.iter() {
            resource.attrs.insert(k.clone(), v.clone());
        }

        if self.schema_url == other.schema_url {
            resource.schema_url = self.schema_url.clone();
        } else if self.schema_url.is_none() {
            // if the other resource has schema url, use it.
            if other.schema_url.is_some() {
                resource.schema_url = other.schema_url.clone();
            }
            // else empty schema url.
        } else {
            // if self has schema url, use it.
            if other.schema_url.is_none() {
                resource.schema_url = self.schema_url.clone();
            }
        }

        resource
    }

    /// Return the [schema url] of the resource. If the resource does not have a schema url, return `None`.
    ///
    /// [schema url]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/schemas/overview.md#schema-url
    pub fn schema_url(&self) -> Option<&str> {
        self.schema_url.as_ref().map(|s| s.as_ref())
    }

    /// Returns the number of attributes for this resource
    pub fn len(&self) -> usize {
        self.attrs.len()
    }

    /// Returns `true` if the resource contains no attributes.
    pub fn is_empty(&self) -> bool {
        self.attrs.is_empty()
    }

    /// Gets an iterator over the attributes of this resource, sorted by key.
    pub fn iter(&self) -> Iter<'_> {
        self.into_iter()
    }

    /// Retrieve the value from resource associate with given key.
    pub fn get(&self, key: Key) -> Option<Value> {
        self.attrs.get(&key).cloned()
    }
}

/// An owned iterator over the entries of a `Resource`.
#[derive(Debug)]
pub struct IntoIter(hash_map::IntoIter<Key, Value>);

impl Iterator for IntoIter {
    type Item = (Key, Value);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl IntoIterator for Resource {
    type Item = (Key, Value);
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.attrs.into_iter())
    }
}

/// An iterator over the entries of a `Resource`.
#[derive(Debug)]
pub struct Iter<'a>(hash_map::Iter<'a, Key, Value>);

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a Key, &'a Value);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> IntoIterator for &'a Resource {
    type Item = (&'a Key, &'a Value);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.attrs.iter())
    }
}

/// ResourceDetector detects OpenTelemetry resource information
///
/// Implementations of this trait can be passed to
/// the [`Resource::from_detectors`] function to generate a Resource from the merged information.
pub trait ResourceDetector {
    /// detect returns an initialized Resource based on gathered information.
    ///
    /// timeout is used in case the detection operation takes too much time.
    ///
    /// If source information to construct a Resource is inaccessible, an empty Resource should be returned
    ///
    /// If source information to construct a Resource is invalid, for example,
    /// missing required values. an empty Resource should be returned.
    fn detect(&self, timeout: Duration) -> Resource;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::EnvResourceDetector;
    use std::collections::HashMap;
    use std::{env, time};

    #[test]
    fn new_resource() {
        let args_with_dupe_keys = vec![KeyValue::new("a", ""), KeyValue::new("a", "final")];

        let mut expected_attrs = HashMap::new();
        expected_attrs.insert(Key::new("a"), Value::from("final"));

        assert_eq!(
            Resource::new(args_with_dupe_keys),
            Resource {
                attrs: expected_attrs,
                schema_url: None,
            }
        );
    }

    #[test]
    fn merge_resource_key_value_pairs() {
        let resource_a = Resource::new(vec![
            KeyValue::new("a", ""),
            KeyValue::new("b", "b-value"),
            KeyValue::new("d", "d-value"),
        ]);

        let resource_b = Resource::new(vec![
            KeyValue::new("a", "a-value"),
            KeyValue::new("c", "c-value"),
            KeyValue::new("d", ""),
        ]);

        let mut expected_attrs = HashMap::new();
        expected_attrs.insert(Key::new("a"), Value::from("a-value"));
        expected_attrs.insert(Key::new("b"), Value::from("b-value"));
        expected_attrs.insert(Key::new("c"), Value::from("c-value"));
        expected_attrs.insert(Key::new("d"), Value::from(""));

        assert_eq!(
            resource_a.merge(&resource_b),
            Resource {
                attrs: expected_attrs,
                schema_url: None,
            }
        );
    }

    #[test]
    fn merge_resource_schema_url() {
        // if both resources contains key value pairs
        let test_cases = vec![
            (Some("http://schema/a"), None, Some("http://schema/a")),
            (Some("http://schema/a"), Some("http://schema/b"), None),
            (None, Some("http://schema/b"), Some("http://schema/b")),
            (
                Some("http://schema/a"),
                Some("http://schema/a"),
                Some("http://schema/a"),
            ),
            (None, None, None),
        ];

        for (schema_url, other_schema_url, expect_schema_url) in test_cases.into_iter() {
            let mut resource = Resource::new(vec![KeyValue::new("key", "")]);
            resource.schema_url = schema_url.map(Into::into);

            let mut other_resource = Resource::new(vec![KeyValue::new("key", "")]);
            other_resource.schema_url = other_schema_url.map(Into::into);

            assert_eq!(
                resource.merge(&other_resource).schema_url,
                expect_schema_url.map(Into::into)
            );
        }

        // if only one resource contains key value pairs
        let resource = Resource::from_schema_url(vec![], "http://schema/a");
        let other_resource = Resource::new(vec![KeyValue::new("key", "")]);

        assert_eq!(resource.merge(&other_resource).schema_url, None);
    }

    #[test]
    fn detect_resource() {
        env::set_var("OTEL_RESOURCE_ATTRIBUTES", "key=value, k = v , a= x, a=z");
        env::set_var("irrelevant".to_uppercase(), "20200810");

        let detector = EnvResourceDetector::new();
        let resource =
            Resource::from_detectors(time::Duration::from_secs(5), vec![Box::new(detector)]);
        assert_eq!(
            resource,
            Resource::new(vec![
                KeyValue::new("key", "value"),
                KeyValue::new("k", "v"),
                KeyValue::new("a", "x"),
                KeyValue::new("a", "z"),
            ])
        )
    }
}
