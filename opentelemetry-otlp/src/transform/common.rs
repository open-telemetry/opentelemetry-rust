use crate::proto::common::{AnyValue, ArrayValue, KeyValue};
use opentelemetry::sdk::trace::EvictedHashMap;
use opentelemetry::Value;
use protobuf::RepeatedField;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) struct Attributes(pub(crate) ::protobuf::RepeatedField<crate::proto::common::KeyValue>);

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
            Value::String(val) => any_value.set_string_value(val),
            Value::Array(vals) => any_value.set_array_value({
                let mut array_value = ArrayValue::new();
                array_value.set_values(RepeatedField::from_vec(
                    vals.into_iter().map(AnyValue::from).collect(),
                ));
                array_value
            }),
        };

        any_value
    }
}

pub(crate) fn to_nanos(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos() as u64
}
