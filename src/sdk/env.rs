//! EnvResourceDetector
//!
//! Implementation of ResourceDetector in extract information from environment.
use crate::sdk::resource::ResourceDetector;
use crate::sdk::Resource;
use crate::api::{KeyValue, Key, Value};
use std::env;

const OTEL_RESOURCE_ATTRIBUTES: &'static str = "OTEL_RESOURCE_ATTRIBUTES";

/// Resource detector implements ResourceDetector and is used to extractor general SDK configuration from environment
/// See https://github.com/open-telemetry/opentelemetry-specification/tree/master/specification/resource/semantic_conventions#telemetry-sdk for semantic conventions.
#[derive(Debug)]
pub struct EnvResourceDetector {}

impl ResourceDetector for EnvResourceDetector {
    fn detect(&self) -> Option<Resource> {
        match env::var(OTEL_RESOURCE_ATTRIBUTES) {
            Ok(s) => Some(construct_otel_resources(s)),
            Err(_) => None
        }
    }
}

impl EnvResourceDetector {
    /// Create `EnvResourceDetector` instance.
    pub fn new() -> Self {
        EnvResourceDetector {}
    }
}

fn construct_otel_resources(s: String) -> Resource {
    let mut key_values = vec![];
    for entries in s.split(",") {
        let key_value_strs = entries.split("=").map(str::trim).collect::<Vec<&str>>();
        if key_value_strs.len() != 2 {
            continue;
        }

        let key = Key::from(key_value_strs.get(0).unwrap().to_string());
        let value = Value::String(key_value_strs.get(1).unwrap().to_string());

        key_values.push(KeyValue::new(key, value));
    }

    Resource::new(key_values)
}

#[cfg(test)]
mod tests {
    use std::env;
    use crate::sdk::env::OTEL_RESOURCE_ATTRIBUTES;
    use crate::sdk::EnvResourceDetector;
    use crate::sdk::resource::{Resource, ResourceDetector};
    use crate::api::{KeyValue, Key, Value};

    #[test]
    fn test_read_from_env() {
        env::set_var(OTEL_RESOURCE_ATTRIBUTES, "key=value, k = v , a= x, a=z");
        env::set_var("irrelevant".to_uppercase(), "20200810");

        let detector = EnvResourceDetector::new();
        let resource = detector.detect();
        assert!(resource.is_some());
        assert_eq!(resource.unwrap(), Resource::new(vec![
            KeyValue::new(Key::new("key".to_string()), Value::String("value".to_string())),
            KeyValue::new(Key::new("k".to_string()), Value::String("v".to_string())),
            KeyValue::new(Key::new("a".to_string()), Value::String("x".to_string())),
            KeyValue::new(Key::new("a".to_string()), Value::String("z".to_string()))
        ]))
    }

    #[test]
    fn test_empty() {
        let detector = EnvResourceDetector::new();
        let resource = detector.detect();
        assert!(resource.is_none())
    }
}