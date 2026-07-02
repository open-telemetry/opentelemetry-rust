#[cfg(all(
    any(feature = "trace", feature = "metrics", feature = "logs"),
    any(feature = "http-proto", feature = "http-json", feature = "grpc-tonic")
))]
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(all(
    any(feature = "trace", feature = "metrics", feature = "logs"),
    any(feature = "http-proto", feature = "http-json", feature = "grpc-tonic")
))]
pub(crate) fn to_nanos(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos() as u64
}

#[cfg(all(
    any(feature = "trace", feature = "metrics", feature = "logs"),
    any(feature = "http-proto", feature = "http-json", feature = "grpc-tonic")
))]
pub(crate) mod tonic {
    use opentelemetry::{Array, Value};
    use opentelemetry_proto::tonic::common::v1::{
        any_value, AnyValue, ArrayValue, InstrumentationScope, KeyValue,
    };
    use std::borrow::Cow;

    #[cfg(any(feature = "trace", feature = "logs"))]
    #[derive(Debug, Default)]
    pub(crate) struct ResourceAttributesWithSchema {
        pub(crate) attributes: Attributes,
        pub(crate) schema_url: Option<String>,
    }

    #[cfg(any(feature = "trace", feature = "logs"))]
    use opentelemetry_sdk::Resource;

    #[cfg(any(feature = "trace", feature = "logs"))]
    pub(crate) fn resource_to_attributes_with_schema(
        resource: &Resource,
    ) -> ResourceAttributesWithSchema {
        ResourceAttributesWithSchema {
            attributes: resource_attributes(resource),
            schema_url: resource.schema_url().map(ToString::to_string),
        }
    }

    #[cfg(any(feature = "trace", feature = "logs"))]
    impl From<&Resource> for ResourceAttributesWithSchema {
        fn from(resource: &Resource) -> Self {
            resource_to_attributes_with_schema(resource)
        }
    }

    pub(crate) fn instrumentation_scope_to_proto(
        library: opentelemetry::InstrumentationScope,
        target: Option<Cow<'static, str>>,
    ) -> InstrumentationScope {
        InstrumentationScope {
            name: target.map_or_else(|| library.name().to_owned(), Cow::into_owned),
            version: library.version().unwrap_or_default().to_owned(),
            attributes: keyvalues_to_proto(library.attributes().cloned()),
            ..Default::default()
        }
    }

    pub(crate) fn instrumentation_scope_ref_to_proto(
        library: &opentelemetry::InstrumentationScope,
        target: Option<Cow<'static, str>>,
    ) -> InstrumentationScope {
        InstrumentationScope {
            name: target.map_or_else(|| library.name().to_owned(), Cow::into_owned),
            version: library.version().unwrap_or_default().to_owned(),
            attributes: keyvalues_to_proto(library.attributes().cloned()),
            ..Default::default()
        }
    }

    /// Wrapper type for Vec<`KeyValue`>
    #[derive(Default, Debug)]
    pub(crate) struct Attributes(pub(crate) ::std::vec::Vec<KeyValue>);

    pub(crate) fn keyvalues_to_proto<I: IntoIterator<Item = opentelemetry::KeyValue>>(
        kvs: I,
    ) -> Vec<KeyValue> {
        kvs.into_iter()
            .map(|api_kv| KeyValue {
                key: api_kv.key.as_str().to_string(),
                value: Some(value_to_any_value(api_kv.value)),
                key_strindex: 0,
            })
            .collect()
    }

    // Kept as a `From` impl since `Attributes` is a local type, so orphan rule is satisfied.
    impl<I: IntoIterator<Item = opentelemetry::KeyValue>> From<I> for Attributes {
        fn from(kvs: I) -> Self {
            Attributes(keyvalues_to_proto(kvs))
        }
    }

    #[cfg(feature = "logs")]
    impl<K: Into<String>, V: Into<AnyValue>> FromIterator<(K, V)> for Attributes {
        fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
            Attributes(
                iter.into_iter()
                    .map(|(k, v)| KeyValue {
                        key: k.into(),
                        value: Some(v.into()),
                        key_strindex: 0,
                    })
                    .collect(),
            )
        }
    }

    pub(crate) fn value_to_any_value(value: Value) -> AnyValue {
        AnyValue {
            value: match value {
                Value::Bool(val) => Some(any_value::Value::BoolValue(val)),
                Value::I64(val) => Some(any_value::Value::IntValue(val)),
                Value::F64(val) => Some(any_value::Value::DoubleValue(val)),
                Value::String(val) => Some(any_value::Value::StringValue(val.to_string())),
                Value::Array(array) => Some(any_value::Value::ArrayValue(match array {
                    Array::Bool(vals) => array_into_proto(vals),
                    Array::I64(vals) => array_into_proto(vals),
                    Array::F64(vals) => array_into_proto(vals),
                    Array::String(vals) => array_into_proto(vals),
                    _ => unreachable!("Nonexistent array type"),
                })),
                _ => unreachable!("Nonexistent value type"),
            },
        }
    }

    fn array_into_proto<T>(vals: Vec<T>) -> ArrayValue
    where
        Value: From<T>,
    {
        let values = vals
            .into_iter()
            .map(|val| value_to_any_value(Value::from(val)))
            .collect();

        ArrayValue { values }
    }

    #[cfg(any(feature = "trace", feature = "logs"))]
    pub(crate) fn resource_attributes(resource: &Resource) -> Attributes {
        Attributes(keyvalues_to_proto(
            resource
                .iter()
                .map(|(k, v)| opentelemetry::KeyValue::new(k.clone(), v.clone())),
        ))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use opentelemetry::KeyValue;

        fn assert_scope_fields(
            proto_scope: &InstrumentationScope,
            expected_name: &str,
            expected_version: &str,
            expected_attr_key: &str,
        ) {
            assert_eq!(proto_scope.name, expected_name);
            assert_eq!(proto_scope.version, expected_version);
            assert_eq!(proto_scope.attributes.len(), 1);
            assert_eq!(proto_scope.attributes[0].key, expected_attr_key);
        }

        #[test]
        fn instrumentation_scope_with_target_overrides_name_but_preserves_version_and_attributes() {
            let scope = opentelemetry::InstrumentationScope::builder("my-lib")
                .with_version("1.0.0")
                .with_attributes([KeyValue::new("feature", "metrics")])
                .build();
            let target: Option<Cow<'static, str>> = Some(Cow::Borrowed("my_app::handlers"));

            let from_owned = instrumentation_scope_to_proto(scope.clone(), target.clone());
            let from_ref = instrumentation_scope_ref_to_proto(&scope, target);

            assert_scope_fields(&from_owned, "my_app::handlers", "1.0.0", "feature");
            assert_scope_fields(&from_ref, "my_app::handlers", "1.0.0", "feature");
        }

        #[test]
        fn instrumentation_scope_without_target_preserves_all_fields() {
            let scope = opentelemetry::InstrumentationScope::builder("my-lib")
                .with_version("1.0.0")
                .with_attributes([KeyValue::new("feature", "metrics")])
                .build();
            let target: Option<Cow<'static, str>> = None;

            let from_owned = instrumentation_scope_to_proto(scope.clone(), target.clone());
            let from_ref = instrumentation_scope_ref_to_proto(&scope, target);

            assert_scope_fields(&from_owned, "my-lib", "1.0.0", "feature");
            assert_scope_fields(&from_ref, "my-lib", "1.0.0", "feature");
        }
    }
}
