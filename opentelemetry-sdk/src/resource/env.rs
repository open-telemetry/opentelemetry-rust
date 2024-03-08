//! Environment variables resource detector
//!
//! Implementation of `ResourceDetector` to extract a `Resource` from environment
//! variables.
use crate::resource::{Resource, ResourceDetector};
use opentelemetry::{Key, KeyValue, Value};
use std::env;
use std::time::Duration;

const OTEL_RESOURCE_ATTRIBUTES: &str = "OTEL_RESOURCE_ATTRIBUTES";
const OTEL_SERVICE_NAME: &str = "OTEL_SERVICE_NAME";

/// Resource detector implements ResourceDetector and is used to extract
/// general SDK configuration from environment.
///
/// See
/// [semantic conventions](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/resource/sdk.md#specifying-resource-information-via-an-environment-variable)
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
        EnvResourceDetector { _private: () }
    }
}

impl Default for EnvResourceDetector {
    fn default() -> Self {
        EnvResourceDetector::new()
    }
}

/// Extract key value pairs and construct a resource from resources string like
/// key1=value1,key2=value2,...
fn construct_otel_resources(s: String) -> Resource {
    Resource::new(s.split_terminator(',').filter_map(|entry| {
        let mut parts = entry.splitn(2, '=');
        let key = parts.next()?.trim();
        let value = parts.next()?.trim();
        if value.find('=').is_some() {
            return None;
        }

        Some(KeyValue::new(key.to_owned(), value.to_owned()))
    }))
}

/// There are attributes which MUST be provided by the SDK as specified in
/// [the Resource SDK specification]. This detector detects those attributes and
/// if the attribute cannot be detected, it uses the default value.
///
/// This detector will first try `OTEL_SERVICE_NAME` env. If it's not available,
/// then it will check the `OTEL_RESOURCE_ATTRIBUTES` env and see if it contains
/// `service.name` resource. If it's also not available, it will use `unknown_service`.
///
/// If users want to set an empty service name, they can provide
/// a resource with empty value and `service.name` key.
///
/// [the Resource SDK specification]:https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/resource/sdk.md#sdk-provided-resource-attributes
#[derive(Debug)]
pub struct SdkProvidedResourceDetector;

impl ResourceDetector for SdkProvidedResourceDetector {
    fn detect(&self, _timeout: Duration) -> Resource {
        Resource::new(vec![KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            env::var(OTEL_SERVICE_NAME)
                .ok()
                .filter(|s| !s.is_empty())
                .map(Value::from)
                .or_else(|| {
                    EnvResourceDetector::new()
                        .detect(Duration::from_secs(0))
                        .get(Key::new(
                            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                        ))
                })
                .unwrap_or_else(|| "unknown_service".into()),
        )])
    }
}

#[cfg(test)]
mod tests {
    use crate::resource::env::{
        SdkProvidedResourceDetector, OTEL_RESOURCE_ATTRIBUTES, OTEL_SERVICE_NAME,
    };
    use crate::resource::{EnvResourceDetector, Resource, ResourceDetector};
    use opentelemetry::{Key, KeyValue, Value};
    use std::time::Duration;

    #[test]
    fn test_read_from_env() {
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
                let resource = detector.detect(Duration::from_secs(5));
                assert_eq!(
                    resource,
                    Resource::new(vec![
                        KeyValue::new("key", "value"),
                        KeyValue::new("k", "v"),
                        KeyValue::new("a", "x"),
                        KeyValue::new("a", "z"),
                    ])
                );
            },
        );

        let detector = EnvResourceDetector::new();
        let resource = detector.detect(Duration::from_secs(5));
        assert!(resource.is_empty());
    }

    #[test]
    fn test_sdk_provided_resource_detector() {
        // Ensure no env var set
        let no_env = SdkProvidedResourceDetector.detect(Duration::from_secs(1));
        assert_eq!(
            no_env.get(Key::from_static_str(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME
            )),
            Some(Value::from("unknown_service")),
        );

        temp_env::with_var(OTEL_SERVICE_NAME, Some("test service"), || {
            let with_service = SdkProvidedResourceDetector.detect(Duration::from_secs(1));
            assert_eq!(
                with_service.get(Key::from_static_str(
                    opentelemetry_semantic_conventions::resource::SERVICE_NAME
                )),
                Some(Value::from("test service")),
            )
        });

        temp_env::with_var(
            OTEL_RESOURCE_ATTRIBUTES,
            Some("service.name=test service1"),
            || {
                let with_service = SdkProvidedResourceDetector.detect(Duration::from_secs(1));
                assert_eq!(
                    with_service.get(Key::from_static_str(
                        opentelemetry_semantic_conventions::resource::SERVICE_NAME
                    )),
                    Some(Value::from("test service1")),
                )
            },
        );

        // OTEL_SERVICE_NAME takes priority
        temp_env::with_vars(
            [
                (OTEL_SERVICE_NAME, Some("test service")),
                (OTEL_RESOURCE_ATTRIBUTES, Some("service.name=test service3")),
            ],
            || {
                let with_service = SdkProvidedResourceDetector.detect(Duration::from_secs(1));
                assert_eq!(
                    with_service.get(Key::from_static_str(
                        opentelemetry_semantic_conventions::resource::SERVICE_NAME
                    )),
                    Some(Value::from("test service"))
                );
            },
        );
    }
}
