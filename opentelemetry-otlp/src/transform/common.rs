#[cfg(any(feature = "trace", feature = "metrics", feature = "logs"))]
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(any(feature = "trace", feature = "metrics", feature = "logs"))]
pub(crate) fn to_nanos(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos() as u64
}

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
        if let Some(t) = target {
            InstrumentationScope {
                name: t.to_string(),
                version: String::new(),
                attributes: vec![],
                ..Default::default()
            }
        } else {
            InstrumentationScope {
                name: library.name().to_owned(),
                version: library.version().map(ToOwned::to_owned).unwrap_or_default(),
                attributes: keyvalues_to_proto(library.attributes().cloned()),
                ..Default::default()
            }
        }
    }

    pub(crate) fn instrumentation_scope_ref_to_proto(
        library: &opentelemetry::InstrumentationScope,
        target: Option<Cow<'static, str>>,
    ) -> InstrumentationScope {
        if let Some(t) = target {
            InstrumentationScope {
                name: t.to_string(),
                version: String::new(),
                attributes: vec![],
                ..Default::default()
            }
        } else {
            InstrumentationScope {
                name: library.name().to_owned(),
                version: library.version().map(ToOwned::to_owned).unwrap_or_default(),
                attributes: keyvalues_to_proto(library.attributes().cloned()),
                ..Default::default()
            }
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
}
