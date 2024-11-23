use opentelemetry::KeyValue;

use super::{Resource, ResourceDetector};

/// Builder to allow easy composition of a Resource
#[derive(Debug, Default)]
pub struct ResourceBuilder {
    resource: Resource,
}

impl ResourceBuilder {
    /// Create ResourceBuilder with an empty [Resource].
    pub fn new_empty() -> Self {
        ResourceBuilder {
            resource: Resource::empty(),
        }
    }

    /// Create ResourceBuilder with a default [Resource].
    pub fn new_default() -> Self {
        ResourceBuilder {
            resource: Resource::default(),
        }
    }

    /// Add a single [ResourceDetector] to your resource.
    pub fn with_detector(self, detector: Box<dyn ResourceDetector>) -> Self {
        self.with_detectors(vec![detector])
    }

    /// Add multiple [ResourceDetector] to your resource.
    pub fn with_detectors(mut self, detectors: Vec<Box<dyn ResourceDetector>>) -> Self {
        self.resource = self.resource.merge(&Resource::from_detectors(detectors));
        self
    }

    /// Add a [KeyValue] to the resource.
    pub fn with_key_value(self, kv: KeyValue) -> Self {
        self.with_key_values(vec![kv])
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
