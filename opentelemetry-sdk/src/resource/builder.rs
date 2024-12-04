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
