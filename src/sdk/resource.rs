//! # Resource
//!
//! A `Resource` is an immutable representation of the entity producing telemetry. For example, a
//! process producing telemetry that is running in a container on Kubernetes has a Pod name, it is
//! in a namespace, and possibly is part of a Deployment which also has a name. All three of these
//! attributes can be included in the `Resource`.
//!
//! The primary purpose of resources as a first-class concept in the SDK is decoupling of discovery
//! of resource information from exporters. This allows for independent development and easy
//! customization for users that need to integrate with closed source environments. When used with
//! distributed tracing, a resource can be associated with the [`Provider`] when it is created.
//! That association cannot be changed later. When associated with a `Provider`, all `Span`s
//! produced by any `Tracer` from the provider are associated with this `Resource`.
//!
//! [`Provider`]: ../../api/trace/provider/trait.Provider.html
use crate::api;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::collections::{btree_map, btree_map::Entry, BTreeMap};
use crate::api::KeyValue;
use std::time::Duration;

/// Describes an entity about which identifying information and metadata is exposed.
///
/// Items are sorted by their key, and are only overwritten if the value is an empty string.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Resource {
    attrs: BTreeMap<api::Key, api::Value>,
}

impl Resource {
    /// Create a new `Resource` from key value pairs.
    ///
    /// Values are de-duplicated by key, and the first key-value pair with a non-empty string value
    /// will be retained
    pub fn new<T: IntoIterator<Item=api::KeyValue>>(kvs: T) -> Self {
        let mut resource = Resource::default();

        for kv in kvs.into_iter() {
            resource.insert(kv);
        }

        resource
    }

    /// Create a new `Resource` from resource detectors.
    ///
    /// timeout will be applied to each detector.
    pub fn from_detectors(timeout: Duration, detectors: Vec<Box<dyn ResourceDetector>>) -> Self {
        let mut resource = Resource::default();
        for detector in detectors {
            let detected_res = detector.detect(timeout);
            for (key, value) in detected_res.into_iter() {
                // using insert instead of merge to avoid clone.
                resource.insert(KeyValue::new(key, value));
            }
        }

        resource
    }

    /// Create a new `Resource` by combining two resources.
    ///
    /// Keys from this resource have priority over keys from the merged resource.
    pub fn merge(&self, other: &Self) -> Self {
        if self.attrs.is_empty() {
            return other.clone();
        }
        if other.attrs.is_empty() {
            return self.clone();
        }

        let mut resource = Resource::default();

        // attrs from self must be added first so they have priority
        for (k, v) in self.attrs.iter() {
            resource.insert(api::KeyValue {
                key: k.clone(),
                value: v.clone(),
            });
        }
        for (k, v) in other.attrs.iter() {
            resource.insert(api::KeyValue {
                key: k.clone(),
                value: v.clone(),
            });
        }

        resource
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
    pub fn iter(&self) -> Iter {
        self.into_iter()
    }

    /// Insert a key-value pair into a `Resource`
    fn insert(&mut self, item: api::KeyValue) {
        match self.attrs.entry(item.key) {
            Entry::Occupied(mut existing_item) => {
                if let api::Value::String(s) = existing_item.get() {
                    if s.is_empty() {
                        existing_item.insert(item.value);
                    }
                }
            }
            Entry::Vacant(v) => {
                v.insert(item.value);
            }
        }
    }
}

/// An owned iterator over the entries of a `Resource`.
#[derive(Debug)]
pub struct IntoIter(btree_map::IntoIter<api::Key, api::Value>);

impl Iterator for IntoIter {
    type Item = (api::Key, api::Value);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl IntoIterator for Resource {
    type Item = (api::Key, api::Value);
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.attrs.into_iter())
    }
}

/// An iterator over the entries of a `Resource`.
#[derive(Debug)]
pub struct Iter<'a>(btree_map::Iter<'a, api::Key, api::Value>);

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a api::Key, &'a api::Value);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> IntoIterator for &'a Resource {
    type Item = (&'a api::Key, &'a api::Value);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.attrs.iter())
    }
}

/// ResourceDetector detects OpenTelemetry resource information
///
/// Implementations of this trait can be passed to
/// the `Resource::from_detectors` function to generate a Resource from the merged information.
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
    use super::Resource;
    use crate::api;
    use std::collections::BTreeMap;
    use std::{env, time};
    use crate::sdk::EnvResourceDetector;

    #[test]
    fn new_resource() {
        let args_with_dupe_keys = vec![
            api::KeyValue::new("a", ""),
            api::KeyValue::new("a", "final"),
        ];

        let mut expected_attrs = BTreeMap::new();
        expected_attrs.insert(api::Key::new("a"), api::Value::from("final"));

        assert_eq!(
            Resource::new(args_with_dupe_keys),
            Resource {
                attrs: expected_attrs
            }
        );
    }

    #[test]
    fn merge_resource() {
        let resource_a = Resource::new(vec![
            api::KeyValue::new("a", ""),
            api::KeyValue::new("b", "b-value"),
        ]);

        let resource_b = Resource::new(vec![
            api::KeyValue::new("a", "final"),
            api::KeyValue::new("c", "c-value"),
        ]);

        let mut expected_attrs = BTreeMap::new();
        expected_attrs.insert(api::Key::new("a"), api::Value::from("final"));
        expected_attrs.insert(api::Key::new("b"), api::Value::from("b-value"));
        expected_attrs.insert(api::Key::new("c"), api::Value::from("c-value"));

        assert_eq!(
            resource_a.merge(&resource_b),
            Resource {
                attrs: expected_attrs
            }
        );
    }

    #[test]
    fn detect_resource() {
        env::set_var("OTEL_RESOURCE_ATTRIBUTES", "key=value, k = v , a= x, a=z");
        env::set_var("irrelevant".to_uppercase(), "20200810");

        let detector = EnvResourceDetector::new();
        let resource = Resource::from_detectors(time::Duration::from_secs(5), vec![Box::new(detector)]);
        assert_eq!(resource, Resource::new(vec![
            api::KeyValue::new(api::Key::new("key".to_string()), api::Value::String("value".to_string())),
            api::KeyValue::new(api::Key::new("k".to_string()), api::Value::String("v".to_string())),
            api::KeyValue::new(api::Key::new("a".to_string()), api::Value::String("x".to_string())),
            api::KeyValue::new(api::Key::new("a".to_string()), api::Value::String("z".to_string()))
        ]))
    }
}
