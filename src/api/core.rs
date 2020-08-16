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
    pub fn bool<T: Into<bool>>(&self, value: T) -> KeyValue {
        KeyValue {
            key: self.clone(),
            value: Value::Bool(value.into()),
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

    /// Create a `KeyValue` pair for `String` values.
    pub fn string<T: Into<String>>(&self, value: T) -> KeyValue {
        KeyValue {
            key: self.clone(),
            value: Value::String(value.into()),
        }
    }

    /// Create a `KeyValue` pair for byte arrays.
    pub fn bytes<T: Into<Vec<u8>>>(&self, value: T) -> KeyValue {
        KeyValue {
            key: self.clone(),
            value: Value::Bytes(value.into()),
        }
    }

    /// Create a `KeyValue` pair for arrays.
    pub fn array<T: Into<Vec<Value>>>(&self, value: T) -> KeyValue {
        KeyValue {
            key: self.clone(),
            value: Value::Array(value.into()),
        }
    }

    /// Returns a reference to the underlying key name
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<&'static str> for Key {
    /// Convert a `&str` to a `Key`.
    fn from(key_str: &'static str) -> Self {
        Key(Cow::from(key_str))
    }
}

impl From<String> for Key {
    /// Convert a `String` to a `Key`.
    fn from(string: String) -> Self {
        Key(Cow::from(string))
    }
}

impl From<Key> for String {
    /// Converts `Key` instances into `String`.
    fn from(key: Key) -> Self {
        key.0.into_owned()
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
    /// Array of homogeneous values
    Array(Vec<Value>),
}

macro_rules! from_values {
   (
        $(
            ($t:ty, $val:expr);
        )+
    ) => {
        $(
            impl From<$t> for Value {
                fn from(t: $t) -> Self {
                    $val(t)
                }
            }
        )+
    }
}

from_values!(
    (bool, Value::Bool);
    (i64, Value::I64);
    (u64, Value::U64);
    (f64, Value::F64);
    (String, Value::String);
    (Vec<u8>, Value::Bytes);
    (Vec<Value>, Value::Array);
);

impl From<&str> for Value {
    /// Convenience method for creating a `Value` from a `&str`.
    fn from(value_str: &str) -> Self {
        Value::String(value_str.to_string())
    }
}

impl From<Value> for String {
    /// Convert `Value` types to `String` for use by exporters that only use
    /// `String` values.
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(value) => value.to_string(),
            Value::I64(value) => value.to_string(),
            Value::U64(value) => value.to_string(),
            Value::F64(value) => value.to_string(),
            Value::String(value) => value,
            Value::Bytes(value) => String::from_utf8(value).unwrap_or_else(|_| String::new()),
            Value::Array(value) => format_value_array_as_string(&value),
        }
    }
}

impl From<&Value> for String {
    /// Convert `&Value` types to `String` for use by exporters that only use
    /// `String` values.
    fn from(value: &Value) -> Self {
        match value {
            Value::Bool(value) => value.to_string(),
            Value::I64(value) => value.to_string(),
            Value::U64(value) => value.to_string(),
            Value::F64(value) => value.to_string(),
            Value::String(value) => value.clone(),
            Value::Bytes(value) => {
                String::from_utf8(value.clone()).unwrap_or_else(|_| String::new())
            }
            Value::Array(value) => format_value_array_as_string(value),
        }
    }
}

fn format_value_array_as_string(v: &[Value]) -> String {
    format!(
        "[{}]",
        v.iter()
            .map(|elem| match elem {
                v @ Value::String(_) | v @ Value::Bytes(_) => format!(r#""{}""#, String::from(v)),
                v => String::from(v),
            })
            .collect::<Vec<_>>()
            .join(",")
    )
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
#[derive(Clone, Default, Debug, PartialEq, Hash)]
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

impl AsRef<str> for Unit {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
