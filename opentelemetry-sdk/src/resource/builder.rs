use std::{borrow::Cow, time::Duration};

use opentelemetry::{KeyValue, Value};

use super::{Resource, ResourceDetector, SERVICE_NAME};

/// Builder to allow easy composition of a Resource
#[derive(Debug)]
pub struct ResourceBuilder {
    resource: Resource,
}

impl Default for ResourceBuilder {
    /// Create ResourceBuilder with [Resource::default()].
    ///
    /// The default resource will contain the following detectors:
    /// - SdkProvidedResourceDetector
    /// - TelemetryResourceDetector
    /// - EnvResourceDetector
    fn default() -> Self {
        ResourceBuilder {
            resource: Resource::default(),
        }
    }
}

impl ResourceBuilder {
    /// Create ResourceBuilder with [Resource::empty()].
    pub fn new() -> Self {
        ResourceBuilder {
            resource: Resource::empty(),
        }
    }

    /// Add a single [ResourceDetector] to your resource.
    pub fn with_detector(self, detector: Box<dyn ResourceDetector>) -> Self {
        self.with_detectors(vec![detector])
    }

    /// Add multiple [ResourceDetector] to your resource.
    pub fn with_detectors(mut self, detectors: Vec<Box<dyn ResourceDetector>>) -> Self {
        self.resource = self
            .resource
            .merge(&Resource::from_detectors(Duration::from_secs(0), detectors));
        self
    }

    /// Add a [KeyValue] to the resource.
    pub fn with_attribute(self, kv: KeyValue) -> Self {
        self.with_attributes(vec![kv])
    }

    /// Add multiple [KeyValue]s to the resource.
    pub fn with_attributes<T: IntoIterator<Item = KeyValue>>(mut self, kvs: T) -> Self {
        self.resource = self.resource.merge(&Resource::new(kvs));
        self
    }

    /// Add `service.name` resource attribute.
    pub fn with_service_name(self, name: impl Into<Value>) -> Self {
        self.with_attribute(KeyValue::new(SERVICE_NAME, name.into()))
    }

    /// This will merge the provided `schema_url` with the current state of the Resource being built. It
    /// will use the following rules to determine which `schema_url` should be used.
    ///
    /// ### [Schema url]
    /// Schema url is determined by the following rules, in order:
    /// 1. If the current builder resource doesn't have a `schema_url`, the provided `schema_url` will be used.
    /// 2. If the current builder resource has a `schema_url`, and the provided `schema_url` is different from the builder resource, `schema_url` will be empty.
    /// 3. If the provided `schema_url` is the same as the current builder resource, it will be used.
    ///
    /// [Schema url]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/schemas/overview.md#schema-url
    pub fn with_schema_url<KV, S>(mut self, attributes: KV, schema_url: S) -> Self
    where
        KV: IntoIterator<Item = KeyValue>,
        S: Into<Cow<'static, str>>,
    {
        self.resource = Resource::from_schema_url(attributes, schema_url).merge(&self.resource);
        self
    }

    /// Create a [Resource] with the options provided to the [ResourceBuilder].
    pub fn build(self) -> Resource {
        self.resource
    }
}

#[cfg(test)]
mod tests {
    use opentelemetry::KeyValue;

    use crate::resource::EnvResourceDetector;

    use super::*;

    #[test]
    fn with_schema_url() {
        let test_cases = vec![
            (Some("http://schema/a"), Some("http://schema/b"), None),
            (None, Some("http://schema/b"), Some("http://schema/b")),
            (
                Some("http://schema/a"),
                Some("http://schema/a"),
                Some("http://schema/a"),
            ),
        ];

        for (schema_url_a, schema_url_b, expected_schema_url) in test_cases.into_iter() {
            let base_builder = if let Some(url) = schema_url_a {
                ResourceBuilder {
                    resource: Resource::from_schema_url(vec![KeyValue::new("key", "")], url),
                }
            } else {
                ResourceBuilder::new()
            };

            let resource = base_builder
                .with_schema_url(
                    vec![KeyValue::new("key", "")],
                    schema_url_b.expect("should always be Some for this test"),
                )
                .build();

            assert_eq!(
                resource.schema_url().map(|s| s as &str),
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
                let resource = Resource::builder()
                    .with_detector(Box::new(EnvResourceDetector::new()))
                    .with_service_name("testing_service")
                    .with_attribute(KeyValue::new("test1", "test_value"))
                    .with_attributes(vec![
                        KeyValue::new("test1", "test_value1"),
                        KeyValue::new("test2", "test_value2"),
                    ])
                    .build();

                assert_eq!(
                    resource,
                    Resource::new(vec![
                        KeyValue::new("key", "value"),
                        KeyValue::new("test1", "test_value1"),
                        KeyValue::new("test2", "test_value2"),
                        KeyValue::new(SERVICE_NAME, "testing_service"),
                        KeyValue::new("k", "v"),
                        KeyValue::new("a", "x"),
                        KeyValue::new("a", "z"),
                    ])
                )
            },
        )
    }
}
