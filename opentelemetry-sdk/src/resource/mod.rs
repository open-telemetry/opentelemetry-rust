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
//! - [`TelemetryResourceDetector`] - detect telemetry SDK's information.
//!
//! The OS and Process resource detectors are now packaged separately in the `opentelemetry-resource-detector` [contrib crate](https://github.com/open-telemetry/opentelemetry-rust-contrib/tree/main/opentelemetry-resource-detectors), due to their experimental nature in semantic conventions.
mod env;
mod telemetry;

mod attributes;
pub(crate) use attributes::*;

pub use env::EnvResourceDetector;
pub use env::SdkProvidedResourceDetector;
pub use telemetry::TelemetryResourceDetector;

use opentelemetry::{Key, KeyValue, Value};
use std::borrow::Cow;
use std::collections::{hash_map, HashMap};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

/// Inner structure of `Resource` holding the actual data.
/// This structure is designed to be shared among `Resource` instances via `Arc`.
#[derive(Debug, Clone, PartialEq)]
struct ResourceInner {
    attrs: HashMap<Key, Value>,
    schema_url: Option<Cow<'static, str>>,
}

/// An immutable representation of the entity producing telemetry as attributes.
/// Utilizes `Arc` for efficient sharing and cloning.
#[derive(Clone, Debug, PartialEq)]
pub struct Resource {
    inner: Arc<ResourceInner>,
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
    /// This is the basic constructor that initializes a resource with no attributes and no schema URL.
    pub fn empty() -> Self {
        Resource {
            inner: Arc::new(ResourceInner {
                attrs: HashMap::new(),
                schema_url: None,
            }),
        }
    }

    /// Create a new `Resource` from key value pairs.
    ///
    /// Values are de-duplicated by key, and the first key-value pair with a non-empty string value
    /// will be retained
    pub fn new<T: IntoIterator<Item = KeyValue>>(kvs: T) -> Self {
        let mut attrs = HashMap::new();
        for kv in kvs {
            attrs.insert(kv.key, kv.value);
        }

        Resource {
            inner: Arc::new(ResourceInner {
                attrs,
                schema_url: None,
            }),
        }
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
        let schema_url_str = schema_url.into();
        let normalized_schema_url = if schema_url_str.is_empty() {
            None
        } else {
            Some(schema_url_str)
        };
        let mut attrs = HashMap::new();
        for kv in kvs {
            attrs.insert(kv.key, kv.value);
        }
        Resource {
            inner: Arc::new(ResourceInner {
                attrs,
                schema_url: normalized_schema_url,
            }),
        }
    }

    /// Create a new `Resource` from resource detectors.
    ///
    /// timeout will be applied to each detector.
    pub fn from_detectors(timeout: Duration, detectors: Vec<Box<dyn ResourceDetector>>) -> Self {
        let mut resource = Resource::empty();
        for detector in detectors {
            let detected_res = detector.detect(timeout);
            // This call ensures that if the Arc is not uniquely owned,
            // the data is cloned before modification, preserving safety.
            // If the Arc is uniquely owned, it simply returns a mutable reference to the data.
            let inner = Arc::make_mut(&mut resource.inner);
            for (key, value) in detected_res.into_iter() {
                inner.attrs.insert(Key::new(key.clone()), value.clone());
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
        if self.is_empty() {
            return other.clone();
        }
        if other.is_empty() {
            return self.clone();
        }
        let mut combined_attrs = self.inner.attrs.clone();
        for (k, v) in other.inner.attrs.iter() {
            combined_attrs.insert(k.clone(), v.clone());
        }
        // Resolve the schema URL according to the precedence rules
        let combined_schema_url = match (&self.inner.schema_url, &other.inner.schema_url) {
            // If both resources have a schema URL and it's the same, use it
            (Some(url1), Some(url2)) if url1 == url2 => Some(url1.clone()),
            // If both resources have a schema URL but they are not the same, the schema URL will be empty
            (Some(_), Some(_)) => None,
            // If this resource does not have a schema URL, and the other resource has a schema URL, it will be used
            (None, Some(url)) => Some(url.clone()),
            // If this resource has a schema URL, it will be used (covers case 1 and any other cases where `self` has a schema URL)
            (Some(url), _) => Some(url.clone()),
            // If both resources do not have a schema URL, the schema URL will be empty
            (None, None) => None,
        };
        Resource {
            inner: Arc::new(ResourceInner {
                attrs: combined_attrs,
                schema_url: combined_schema_url,
            }),
        }
    }

    /// Return the [schema url] of the resource. If the resource does not have a schema url, return `None`.
    ///
    /// [schema url]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/schemas/overview.md#schema-url
    pub fn schema_url(&self) -> Option<&str> {
        self.inner.schema_url.as_ref().map(|s| s.as_ref())
    }

    /// Returns the number of attributes for this resource
    pub fn len(&self) -> usize {
        self.inner.attrs.len()
    }

    /// Returns `true` if the resource contains no attributes.
    pub fn is_empty(&self) -> bool {
        self.inner.attrs.is_empty()
    }

    /// Gets an iterator over the attributes of this resource, sorted by key.
    pub fn iter(&self) -> Iter<'_> {
        Iter(self.inner.attrs.iter())
    }

    /// Retrieve the value from resource associate with given key.
    pub fn get(&self, key: Key) -> Option<Value> {
        self.inner.attrs.get(&key).cloned()
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
        Iter(self.inner.attrs.iter())
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
    use std::time;

    #[test]
    fn new_resource() {
        let args_with_dupe_keys = vec![KeyValue::new("a", ""), KeyValue::new("a", "final")];

        let mut expected_attrs = HashMap::new();
        expected_attrs.insert(Key::new("a"), Value::from("final"));

        let resource = Resource::new(args_with_dupe_keys);
        let resource_inner = Arc::try_unwrap(resource.inner).expect("Failed to unwrap Arc");
        assert_eq!(resource_inner.attrs, expected_attrs);
        assert_eq!(resource_inner.schema_url, None);
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

        let expected_resource = Resource {
            inner: Arc::new(ResourceInner {
                attrs: expected_attrs,
                schema_url: None, // Assuming schema_url handling if needed
            }),
        };

        assert_eq!(resource_a.merge(&resource_b), expected_resource);
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

        for (schema_url_a, schema_url_b, expected_schema_url) in test_cases.into_iter() {
            let resource_a = Resource::from_schema_url(
                vec![KeyValue::new("key", "")],
                schema_url_a.unwrap_or(""),
            );
            let resource_b = Resource::from_schema_url(
                vec![KeyValue::new("key", "")],
                schema_url_b.unwrap_or(""),
            );

            let merged_resource = resource_a.merge(&resource_b);
            let result_schema_url = merged_resource.schema_url();

            assert_eq!(
                result_schema_url.map(|s| s as &str),
                expected_schema_url,
                "Merging schema_url_a {:?} with schema_url_b {:?} did not yield expected result {:?}",
                schema_url_a, schema_url_b, expected_schema_url
            );
        }

        // if only one resource contains key value pairs
        let resource = Resource::from_schema_url(vec![], "http://schema/a");
        let other_resource = Resource::new(vec![KeyValue::new("key", "")]);

        assert_eq!(resource.merge(&other_resource).schema_url(), None);
    }

    #[test]
    fn detect_resource() {
        temp_env::with_vars(
            [
                (
                    "OTEL_RESOURCE_ATTRIBUTES",
                    Some("key=value, k = v , a= x, a=z"),
                ),
                ("IRRELEVANT", Some("20200810")),
            ],
            || {
                let detector = EnvResourceDetector::new();
                let resource = Resource::from_detectors(
                    time::Duration::from_secs(5),
                    vec![Box::new(detector)],
                );
                assert_eq!(
                    resource,
                    Resource::new(vec![
                        KeyValue::new("key", "value"),
                        KeyValue::new("k", "v"),
                        KeyValue::new("a", "x"),
                        KeyValue::new("a", "z"),
                    ])
                )
            },
        )
    }
}
