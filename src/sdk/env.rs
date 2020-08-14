//! EnvResourceDetector
//!
//! Implementation of `ResourceDetector` to extract a `Resource` from environment variables.
use crate::sdk::resource::ResourceDetector;
use crate::sdk::Resource;
use crate::api::{KeyValue, Key, Value};
use std::env;
use std::time::Duration;

const OTEL_RESOURCE_ATTRIBUTES: &str = "OTEL_RESOURCE_ATTRIBUTES";

/// Resource detector implements ResourceDetector and is used to extract
/// general SDK configuration from environment.
///
/// See
/// [semantic conventions](https://github.com/open-telemetry/opentelemetry-specification/tree/master/specification/resource/semantic_conventions#telemetry-sdk)
/// for details.
#[derive(Debug)]
pub struct EnvResourceDetector {
    _private: (),
}

impl ResourceDetector for EnvResourceDetector {
    fn detect(&self, _timeout: Duration) -> Resource {
        match env::var(OTEL_RESOURCE_ATTRIBUTES) {
            Ok(s) if !s.is_empty() => construct_otel_resources(s),
            Ok(_) | Err(_) => Resource::new(vec![]), // return empty resource
        }
    }
}

impl EnvResourceDetector {
    /// Create `EnvResourceDetector` instance.
    pub fn new() -> Self {
        EnvResourceDetector {
            _private: (),
        }
    }
}

impl Default for EnvResourceDetector {
    fn default() -> Self {
        EnvResourceDetector::new()
    }
}

/// Extract key value pairs and construct a resource from resources string like key1=value1,key2=value2,...
fn construct_otel_resources(s: String) -> Resource {
    let mut key_values = vec![];
    for entries in s.split_terminator(',') {
        let key_value_strs = entries.split('=').map(str::trim).collect::<Vec<&str>>();
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
    use std::{env, time};
    use crate::sdk::env::OTEL_RESOURCE_ATTRIBUTES;
    use crate::sdk::EnvResourceDetector;
    use crate::sdk::resource::{Resource, ResourceDetector};
    use crate::api::{KeyValue, Key, Value};

    #[test]
    fn test_read_from_env() {
        env::set_var(OTEL_RESOURCE_ATTRIBUTES, "key=value, k = v , a= x, a=z");
        env::set_var("irrelevant".to_uppercase(), "20200810");

        let detector = EnvResourceDetector::new();
        let resource = detector.detect(time::Duration::from_secs(5));
        assert_eq!(resource, Resource::new(vec![
            KeyValue::new(Key::new("key".to_string()), Value::String("value".to_string())),
            KeyValue::new(Key::new("k".to_string()), Value::String("v".to_string())),
            KeyValue::new(Key::new("a".to_string()), Value::String("x".to_string())),
            KeyValue::new(Key::new("a".to_string()), Value::String("z".to_string()))
        ]))
    }

    #[test]
    fn test_empty() {
        env::set_var(OTEL_RESOURCE_ATTRIBUTES, "");

        let detector = EnvResourceDetector::new();
        let resource = detector.detect(time::Duration::from_secs(5));
        assert!(resource.is_empty())
    }
}