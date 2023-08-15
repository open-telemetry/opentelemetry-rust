use std::{
    borrow::Cow,
    collections::BTreeMap,
    hash::{Hash, Hasher},
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{LocalResult, TimeZone, Utc};
use ordered_float::OrderedFloat;
use serde::{Serialize, Serializer};

#[derive(Debug, Serialize, Clone, Hash, Eq, PartialEq)]
pub(crate) struct AttributeSet(pub BTreeMap<Key, Value>);

impl From<&opentelemetry_sdk::AttributeSet> for AttributeSet {
    fn from(value: &opentelemetry_sdk::AttributeSet) -> Self {
        AttributeSet(
            value
                .iter()
                .map(|(key, value)| (Key::from(key.clone()), Value::from(value.clone())))
                .collect(),
        )
    }
}

impl From<&opentelemetry_sdk::Resource> for AttributeSet {
    fn from(value: &opentelemetry_sdk::Resource) -> Self {
        AttributeSet(
            value
                .iter()
                .map(|(key, value)| (Key::from(key.clone()), Value::from(value.clone())))
                .collect(),
        )
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Resource {
    attributes: Vec<KeyValue>,
    #[serde(skip_serializing_if = "is_zero")]
    dropped_attributes_count: u64,
}

fn is_zero(v: &u64) -> bool {
    *v == 0
}

impl From<&opentelemetry_sdk::Resource> for Resource {
    fn from(value: &opentelemetry_sdk::Resource) -> Self {
        Resource {
            attributes: value
                .iter()
                .map(|(key, value)| KeyValue {
                    key: key.clone().into(),
                    value: value.clone().into(),
                })
                .collect(),
            dropped_attributes_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub(crate) struct Key(Cow<'static, str>);

impl From<Cow<'static, str>> for Key {
    fn from(value: Cow<'static, str>) -> Self {
        Key(value)
    }
}

impl From<opentelemetry_api::Key> for Key {
    fn from(value: opentelemetry_api::Key) -> Self {
        Key(value.as_str().to_string().into())
    }
}

#[derive(Debug, Serialize, Clone)]
#[allow(dead_code)]
pub(crate) enum Value {
    #[serde(rename = "boolValue")]
    Bool(bool),
    #[serde(rename = "intValue")]
    Int(i64),
    #[serde(rename = "doubleValue")]
    Double(f64),
    #[serde(rename = "stringValue")]
    String(String),
    #[serde(rename = "arrayValue")]
    Array(Vec<Value>),
    #[serde(rename = "kvListValue")]
    KeyValues(Vec<KeyValue>),
    #[serde(rename = "bytesValue")]
    BytesValue(Vec<u8>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (&self, &other) {
            (Value::Double(f), Value::Double(of)) => OrderedFloat(*f).eq(&OrderedFloat(*of)),
            (non_double, other_non_double) => non_double.eq(other_non_double),
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self {
            Value::Bool(b) => b.hash(state),
            Value::Int(i) => i.hash(state),
            Value::Double(f) => OrderedFloat(*f).hash(state),
            Value::String(s) => s.hash(state),
            Value::Array(a) => a.iter().for_each(|v| v.hash(state)),
            Value::KeyValues(kv) => kv.iter().for_each(|kv| {
                kv.key.hash(state);
                kv.value.hash(state);
            }),
            Value::BytesValue(b) => b.hash(state),
        }
    }
}

impl From<opentelemetry_api::Value> for Value {
    fn from(value: opentelemetry_api::Value) -> Self {
        match value {
            opentelemetry_api::Value::Bool(b) => Value::Bool(b),
            opentelemetry_api::Value::I64(i) => Value::Int(i),
            opentelemetry_api::Value::F64(f) => Value::Double(f),
            opentelemetry_api::Value::String(s) => Value::String(s.into()),
            opentelemetry_api::Value::Array(a) => match a {
                opentelemetry_api::Array::Bool(b) => {
                    Value::Array(b.into_iter().map(Value::Bool).collect())
                }
                opentelemetry_api::Array::I64(i) => {
                    Value::Array(i.into_iter().map(Value::Int).collect())
                }
                opentelemetry_api::Array::F64(f) => {
                    Value::Array(f.into_iter().map(Value::Double).collect())
                }
                opentelemetry_api::Array::String(s) => {
                    Value::Array(s.into_iter().map(|s| Value::String(s.into())).collect())
                }
            },
        }
    }
}

#[cfg(feature = "logs")]
impl From<opentelemetry_api::logs::AnyValue> for Value {
    fn from(value: opentelemetry_api::logs::AnyValue) -> Self {
        match value {
            opentelemetry_api::logs::AnyValue::Boolean(b) => Value::Bool(b),
            opentelemetry_api::logs::AnyValue::Int(i) => Value::Int(i),
            opentelemetry_api::logs::AnyValue::Double(d) => Value::Double(d),
            opentelemetry_api::logs::AnyValue::String(s) => Value::String(s.into()),
            opentelemetry_api::logs::AnyValue::ListAny(a) => {
                Value::Array(a.into_iter().map(Into::into).collect())
            }
            opentelemetry_api::logs::AnyValue::Map(m) => Value::KeyValues(
                m.into_iter()
                    .map(|(key, value)| KeyValue {
                        key: key.into(),
                        value: value.into(),
                    })
                    .collect(),
            ),
            opentelemetry_api::logs::AnyValue::Bytes(b) => Value::BytesValue(b),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KeyValue {
    key: Key,
    value: Value,
}

#[cfg(feature = "logs")]
impl From<(opentelemetry_api::Key, opentelemetry_api::logs::AnyValue)> for KeyValue {
    fn from((key, value): (opentelemetry_api::Key, opentelemetry_api::logs::AnyValue)) -> Self {
        KeyValue {
            key: key.into(),
            value: value.into(),
        }
    }
}

impl From<opentelemetry_api::KeyValue> for KeyValue {
    fn from(value: opentelemetry_api::KeyValue) -> Self {
        KeyValue {
            key: value.key.into(),
            value: value.value.into(),
        }
    }
}

impl From<&opentelemetry_api::KeyValue> for KeyValue {
    fn from(value: &opentelemetry_api::KeyValue) -> Self {
        KeyValue {
            key: value.key.clone().into(),
            value: value.value.clone().into(),
        }
    }
}

impl From<(opentelemetry_api::Key, opentelemetry_api::Value)> for KeyValue {
    fn from((key, value): (opentelemetry_api::Key, opentelemetry_api::Value)) -> Self {
        KeyValue {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Scope {
    #[serde(skip_serializing_if = "str::is_empty")]
    name: Cow<'static, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    attributes: Vec<KeyValue>,
    #[serde(skip_serializing_if = "is_zero")]
    dropped_attributes_count: u64,
}

impl From<opentelemetry_sdk::Scope> for Scope {
    fn from(value: opentelemetry_sdk::Scope) -> Self {
        Scope {
            name: value.name,
            version: value.version,
            attributes: Vec::new(),
            dropped_attributes_count: 0,
        }
    }
}

pub(crate) fn as_human_readable<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let duration_since_epoch = time.duration_since(UNIX_EPOCH).unwrap_or_default();

    match Utc.timestamp_opt(
        duration_since_epoch.as_secs() as i64,
        duration_since_epoch.subsec_nanos(),
    ) {
        LocalResult::Single(datetime) => serializer.serialize_str(
            datetime
                .format("%Y-%m-%d %H:%M:%S.%3f")
                .to_string()
                .as_ref(),
        ),
        _ => Err(serde::ser::Error::custom("Invalid Timestamp.")),
    }
}

#[allow(dead_code)]
pub(crate) fn as_opt_human_readable<S>(
    time: &Option<SystemTime>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match time {
        None => serializer.serialize_none(),
        Some(time) => as_human_readable(time, serializer),
    }
}

pub(crate) fn as_unix_nano<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let nanos = time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    serializer.serialize_u128(nanos)
}

#[allow(dead_code)]
pub(crate) fn as_opt_unix_nano<S>(
    time: &Option<SystemTime>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match time {
        None => serializer.serialize_none(),
        Some(time) => as_unix_nano(time, serializer),
    }
}
