use opentelemetry::sdk::trace::EvictedHashMap;
use opentelemetry::{Array, Value};

use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(feature = "tonic")]
pub(crate) mod tonic {
    use super::*;
    use crate::proto::common::v1::{
        any_value, AnyValue, ArrayValue, InstrumentationLibrary, KeyValue,
    };
    use std::borrow::Cow;

    impl From<opentelemetry::sdk::InstrumentationLibrary> for InstrumentationLibrary {
        fn from(library: opentelemetry::sdk::InstrumentationLibrary) -> Self {
            InstrumentationLibrary {
                name: library.name.to_string(),
                version: library.version.unwrap_or(Cow::Borrowed("")).to_string(),
            }
        }
    }

    pub(crate) struct Attributes(pub(crate) ::std::vec::Vec<crate::proto::common::v1::KeyValue>);

    impl From<EvictedHashMap> for Attributes {
        fn from(attributes: EvictedHashMap) -> Self {
            Attributes(
                attributes
                    .into_iter()
                    .map(|(key, value)| KeyValue {
                        key: key.as_str().to_string(),
                        value: Some(value.into()),
                    })
                    .collect(),
            )
        }
    }

    impl From<Vec<opentelemetry::KeyValue>> for Attributes {
        fn from(kvs: Vec<opentelemetry::KeyValue>) -> Self {
            Attributes(
                kvs.into_iter()
                    .map(|api_kv| KeyValue {
                        key: api_kv.key.as_str().to_string(),
                        value: Some(api_kv.value.into()),
                    })
                    .collect(),
            )
        }
    }

    impl From<Value> for AnyValue {
        fn from(value: Value) -> Self {
            AnyValue {
                value: match value {
                    Value::Bool(val) => Some(any_value::Value::BoolValue(val)),
                    Value::I64(val) => Some(any_value::Value::IntValue(val)),
                    Value::F64(val) => Some(any_value::Value::DoubleValue(val)),
                    Value::String(val) => Some(any_value::Value::StringValue(val.into_owned())),
                    Value::Array(array) => Some(any_value::Value::ArrayValue(match array {
                        Array::Bool(vals) => array_into_proto(vals),
                        Array::I64(vals) => array_into_proto(vals),
                        Array::F64(vals) => array_into_proto(vals),
                        Array::String(vals) => array_into_proto(vals),
                    })),
                },
            }
        }
    }

    fn array_into_proto<T>(vals: Vec<T>) -> ArrayValue
    where
        Value: From<T>,
    {
        let values = vals
            .into_iter()
            .map(|val| AnyValue::from(Value::from(val)))
            .collect();

        ArrayValue { values }
    }
}

#[cfg(feature = "http-proto")]
pub(crate) mod prost {
    use super::*;
    use crate::proto::prost::common::v1::{
        any_value, AnyValue, ArrayValue, InstrumentationLibrary, KeyValue,
    };
    use std::borrow::Cow;

    impl From<opentelemetry::sdk::InstrumentationLibrary> for InstrumentationLibrary {
        fn from(library: opentelemetry::sdk::InstrumentationLibrary) -> Self {
            InstrumentationLibrary {
                name: library.name.to_string(),
                version: library.version.unwrap_or(Cow::Borrowed("")).to_string(),
            }
        }
    }

    pub(crate) struct Attributes(
        pub(crate) ::std::vec::Vec<crate::proto::prost::common::v1::KeyValue>,
    );

    impl From<EvictedHashMap> for Attributes {
        fn from(attributes: EvictedHashMap) -> Self {
            Attributes(
                attributes
                    .into_iter()
                    .map(|(key, value)| KeyValue {
                        key: key.as_str().to_string(),
                        value: Some(value.into()),
                    })
                    .collect(),
            )
        }
    }

    impl From<Vec<opentelemetry::KeyValue>> for Attributes {
        fn from(kvs: Vec<opentelemetry::KeyValue>) -> Self {
            Attributes(
                kvs.into_iter()
                    .map(|api_kv| KeyValue {
                        key: api_kv.key.as_str().to_string(),
                        value: Some(api_kv.value.into()),
                    })
                    .collect(),
            )
        }
    }

    impl From<Value> for AnyValue {
        fn from(value: Value) -> Self {
            AnyValue {
                value: match value {
                    Value::Bool(val) => Some(any_value::Value::BoolValue(val)),
                    Value::I64(val) => Some(any_value::Value::IntValue(val)),
                    Value::F64(val) => Some(any_value::Value::DoubleValue(val)),
                    Value::String(val) => Some(any_value::Value::StringValue(val.into_owned())),
                    Value::Array(array) => Some(any_value::Value::ArrayValue(match array {
                        Array::Bool(vals) => array_into_proto(vals),
                        Array::I64(vals) => array_into_proto(vals),
                        Array::F64(vals) => array_into_proto(vals),
                        Array::String(vals) => array_into_proto(vals),
                    })),
                },
            }
        }
    }

    fn array_into_proto<T>(vals: Vec<T>) -> ArrayValue
    where
        Value: From<T>,
    {
        let values = vals
            .into_iter()
            .map(|val| AnyValue::from(Value::from(val)))
            .collect();

        ArrayValue { values }
    }
}

#[cfg(feature = "grpc-sys")]
pub(crate) mod grpcio {
    use super::*;
    use crate::proto::grpcio::common::{AnyValue, ArrayValue, KeyValue};
    use protobuf::RepeatedField;

    pub(crate) struct Attributes(
        pub(crate) ::protobuf::RepeatedField<crate::proto::grpcio::common::KeyValue>,
    );

    impl From<EvictedHashMap> for Attributes {
        fn from(attributes: EvictedHashMap) -> Self {
            Attributes(RepeatedField::from_vec(
                attributes
                    .into_iter()
                    .map(|(key, value)| {
                        let mut kv: KeyValue = KeyValue::new();
                        kv.set_key(key.as_str().to_string());
                        kv.set_value(value.into());
                        kv
                    })
                    .collect(),
            ))
        }
    }

    impl From<Vec<opentelemetry::KeyValue>> for Attributes {
        fn from(kvs: Vec<opentelemetry::KeyValue>) -> Self {
            Attributes(RepeatedField::from_vec(
                kvs.into_iter()
                    .map(|api_kv| {
                        let mut kv: KeyValue = KeyValue::new();
                        kv.set_key(api_kv.key.as_str().to_string());
                        kv.set_value(api_kv.value.into());
                        kv
                    })
                    .collect(),
            ))
        }
    }

    impl From<Value> for AnyValue {
        fn from(value: Value) -> Self {
            let mut any_value = AnyValue::new();
            match value {
                Value::Bool(val) => any_value.set_bool_value(val),
                Value::I64(val) => any_value.set_int_value(val),
                Value::F64(val) => any_value.set_double_value(val),
                Value::String(val) => any_value.set_string_value(val.into_owned()),
                Value::Array(array) => any_value.set_array_value(match array {
                    Array::Bool(vals) => array_into_proto(vals),
                    Array::I64(vals) => array_into_proto(vals),
                    Array::F64(vals) => array_into_proto(vals),
                    Array::String(vals) => array_into_proto(vals),
                }),
            };

            any_value
        }
    }

    fn array_into_proto<T>(vals: Vec<T>) -> ArrayValue
    where
        Value: From<T>,
    {
        let values = RepeatedField::from_vec(
            vals.into_iter()
                .map(|val| AnyValue::from(Value::from(val)))
                .collect(),
        );

        let mut array_value = ArrayValue::new();
        array_value.set_values(values);
        array_value
    }
}

pub(crate) fn to_nanos(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos() as u64
}
