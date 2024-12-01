use std::time::Duration;

use opentelemetry::{KeyValue, Value};

use super::{Resource, ResourceDetector, SERVICE_NAME};

/// Builder to allow easy composition of a Resource
#[derive(Debug)]
pub struct ResourceBuilder {
    resource: Resource,
}

impl Default for ResourceBuilder {
    /// Create ResourceBuilder with [Resource::default()].
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
    pub fn with_key_value(self, kv: KeyValue) -> Self {
        self.with_key_values(vec![kv])
    }

    /// Add an [Attribute] to your resource for "service.name"
    pub fn with_service_name<V>(self, service_name: V) -> Self
    where
        V: Into<Value>,
    {
        self.with_key_value(KeyValue::new(SERVICE_NAME, service_name))
    }

    /// Add multiple [KeyValue]s to the resource.
    pub fn with_key_values<T: IntoIterator<Item = KeyValue>>(mut self, kvs: T) -> Self {
        self.resource = self.resource.merge(&Resource::new(kvs));
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
                    .with_service_name("my_test_service_name")
                    .with_key_value(KeyValue::new("test1", "test_value"))
                    .with_key_values(vec![
                        KeyValue::new("test1", "test_value1"),
                        KeyValue::new("test2", "test_value2"),
                    ])
                    .build();

                assert_eq!(
                    resource,
                    Resource::new(vec![
                        KeyValue::new("key", "value"),
                        KeyValue::new(SERVICE_NAME, "my_test_service_name"),
                        KeyValue::new("test1", "test_value1"),
                        KeyValue::new("test2", "test_value2"),
                        KeyValue::new("k", "v"),
                        KeyValue::new("a", "x"),
                        KeyValue::new("a", "z"),
                    ])
                )
            },
        )
    }
}
