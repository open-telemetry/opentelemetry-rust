//! OpenTelemetry shared core date types
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Key used for metric `LabelSet`s and trace `Span` attributes.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Key(Cow<'static, str>);

impl Key {
    /// Create a new `Key`.
    pub fn new<S: Into<Cow<'static, str>>>(value: S) -> Self {
        Key(value.into())
    }

    /// Create a `KeyValue` pair for `bool` values.
    pub fn bool(&self, value: bool) -> KeyValue {
        KeyValue {
            key: self.clone(),
            value: Value::Bool(value),
        }
    }

    /// Create a `KeyValue` pair for `i64` values.
    pub fn i64(&self, value: i64) -> KeyValue {
        KeyValue {
            key: self.clone(),
            value: Value::I64(value),
        }
    }

    /// Create a `KeyValue` pair for `u64` values.
    pub fn u64(&self, value: u64) -> KeyValue {
        KeyValue {
            key: self.clone(),
            value: Value::U64(value),
        }
    }

    /// Create a `KeyValue` pair for `f64` values.
    pub fn f64(&self, value: f64) -> KeyValue {
        KeyValue {
            key: self.clone(),
            value: Value::F64(value),
        }
    }

    /// Create a `KeyValue` pair for `Into<String>` values.
    pub fn string<S: Into<String>>(&self, value: S) -> KeyValue {
        KeyValue {
            key: self.clone(),
            value: Value::String(value.into()),
        }
    }

    /// Create a `KeyValue` pair for byte arrays.
    pub fn bytes(&self, value: Vec<u8>) -> KeyValue {
        KeyValue {
            key: self.clone(),
            value: Value::Bytes(value),
        }
    }

    /// Returns a reference to the key's `Cow` type for use in `LabelSet`s.
    pub fn inner(&self) -> &Cow<'static, str> {
        &self.0
    }

    /// Returns the inner `Cow` type.
    pub fn into_inner(self) -> Cow<'static, str> {
        self.0
    }
}

impl From<&'static str> for Key {
    /// Convert a `&str` to a `Key`.
    fn from(key_str: &'static str) -> Self {
        Key(Cow::from(key_str))
    }
}

impl Into<Cow<'static, str>> for Key {
    /// Converts `Key` instances into `Cow`
    fn into(self) -> Cow<'static, str> {
        self.0
    }
}

impl Into<String> for Key {
    /// Converts `Key` instances into `String`.
    fn into(self) -> String {
        self.0.to_string()
    }
}

/// Value types for use in `KeyValue` pairs.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// bool values
    Bool(bool),
    /// i64 values
    I64(i64),
    /// u64 values
    U64(u64),
    /// f64 values
    F64(f64),
    /// String values
    String(String),
    /// Byte array values
    Bytes(Vec<u8>),
}

impl From<&str> for Value {
    /// Convenience method for creating a `Value` form a `&str`.
    fn from(value_str: &str) -> Self {
        Value::String(value_str.to_string())
    }
}

impl From<i64> for Value {
    /// Convenience method for creating a `Value` form a `i64`.
    fn from(value: i64) -> Self {
        Value::I64(value)
    }
}

impl ToString for Value {
    /// Convert `Value` types to `String` for use by exporters that only use
    /// `String` values.
    fn to_string(&self) -> String {
        match self {
            Value::Bool(value) => value.to_string(),
            Value::I64(value) => value.to_string(),
            Value::U64(value) => value.to_string(),
            Value::F64(value) => value.to_string(),
            Value::String(value) => value.clone(),
            Value::Bytes(value) => {
                String::from_utf8(value.clone()).unwrap_or_else(|_| String::new())
            }
        }
    }
}

impl Into<Cow<'static, str>> for Value {
    /// Convert `Value` types into `Cow` for use in `LabelSet`s.
    fn into(self) -> Cow<'static, str> {
        self.to_string().into()
    }
}

/// `KeyValue` pairs are used by `LabelSet`s and `Span` attributes.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct KeyValue {
    /// Dimension or event key
    pub key: Key,
    /// Dimension or event value
    pub value: Value,
}

impl KeyValue {
    /// Create a new `KeyValue` pair.
    pub fn new<K, V>(key: K, value: V) -> Self
    where
        K: Into<Key>,
        V: Into<Value>,
    {
        KeyValue {
            key: key.into(),
            value: value.into(),
        }
    }
}

/// Units denote underlying data units tracked by `Meter`s.
#[derive(Default, Debug)]
pub struct Unit(String);

impl Unit {
    /// Create a new `Unit` from an `Into<String>`
    pub fn new<S: Into<String>>(value: S) -> Self {
        Unit(value.into())
    }

    /// View unit as &str
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
