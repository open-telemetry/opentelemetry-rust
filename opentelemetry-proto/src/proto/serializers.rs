//! Serde helpers for the OTLP/JSON encoding of the enclosing generated type tree.
//! See https://opentelemetry.io/docs/specs/otlp/#json-protobuf-encoding for more details

use super::common::v1::any_value::{self, Value};
use super::common::v1::{AnyValue, ArrayValue, KeyValueList};
use serde::de::{self, MapAccess, Visitor};
use serde::ser::{SerializeMap, SerializeStruct};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

pub fn is_default<T>(value: &T) -> bool
where
    T: Default + PartialEq,
{
    value == &T::default()
}

// hex string <-> bytes conversion

pub fn serialize_to_hex_string<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let hex_string = const_hex::encode(bytes);
    serializer.serialize_str(&hex_string)
}

pub fn deserialize_from_hex_string<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    struct BytesVisitor;

    impl<'de> Visitor<'de> for BytesVisitor {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string representing hex-encoded bytes")
        }

        fn visit_str<E>(self, value: &str) -> Result<Vec<u8>, E>
        where
            E: de::Error,
        {
            const_hex::decode(value).map_err(E::custom)
        }
    }

    deserializer.deserialize_str(BytesVisitor)
}

// AnyValue <-> KeyValue conversion
pub fn serialize_to_value<S>(value: &Option<Value>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match &value {
        Some(Value::IntValue(i)) => {
            // Attempt to serialize the intValue field
            let mut map = serializer.serialize_map(Some(1))?;
            map.serialize_entry("intValue", &i.to_string());
            map.end()
        }
        Some(Value::BytesValue(b)) => {
            let mut map = serializer.serialize_map(Some(1))?;
            map.serialize_entry("bytesValue", &base64::encode(b));
            map.end()
        }
        Some(value) => value.serialize(serializer),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize_from_value<'de, D>(deserializer: D) -> Result<Option<Value>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ValueVisitor;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        Int(i64),
        String(String),
    }

    impl StringOrInt {
        fn get_int<'de, V>(&self) -> Result<i64, V::Error>
        where
            V: de::MapAccess<'de>,
        {
            match self {
                Self::Int(val) => Ok(*val),
                Self::String(val) => Ok(val.parse::<i64>().map_err(de::Error::custom)?),
            }
        }
    }

    impl<'de> de::Visitor<'de> for ValueVisitor {
        type Value = Option<Value>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a JSON object for AnyValue")
        }

        fn visit_map<V>(self, mut map: V) -> Result<Option<Value>, V::Error>
        where
            V: de::MapAccess<'de>,
        {
            let mut value: Option<any_value::Value> = None;

            while let Some(key) = map.next_key::<String>()? {
                let key_str = key.as_str();
                match key_str {
                    "stringValue" => {
                        if let Some(s) = map.next_value::<Option<String>>()? {
                            value = Some(any_value::Value::StringValue(s));
                        }
                    }
                    "boolValue" => {
                        if let Some(b) = map.next_value::<Option<bool>>()? {
                            value = Some(any_value::Value::BoolValue(b));
                        }
                    }
                    "intValue" => {
                        if let Some(int_value) = map.next_value::<Option<StringOrInt>>()? {
                            value = Some(any_value::Value::IntValue(int_value.get_int::<V>()?));
                        }
                    }
                    "doubleValue" => {
                        if let Some(d) = map.next_value::<Option<f64>>()? {
                            value = Some(any_value::Value::DoubleValue(d));
                        }
                    }
                    "arrayValue" => {
                        if let Some(a) = map.next_value::<Option<ArrayValue>>()? {
                            value = Some(any_value::Value::ArrayValue(a));
                        }
                    }
                    "kvlistValue" => {
                        if let Some(kv) = map.next_value::<Option<KeyValueList>>()? {
                            value = Some(any_value::Value::KvlistValue(kv));
                        }
                    }
                    "bytesValue" => {
                        if let Some(base64) = map.next_value::<Option<String>>()? {
                            let decoded = base64::decode(base64.as_bytes())
                                .map_err(|e| de::Error::custom(e))?;
                            value = Some(any_value::Value::BytesValue(decoded));
                        }
                    }
                    _ => {
                        //skip unknown keys, and handle error later.
                        continue;
                    }
                }
            }

            // An AnyValue with no selected field is valid and considered empty.
            // Unknown fields are ignored according to the OTLP/JSON specification,
            // so an empty or unknown-only object has the same representation.
            Ok(value)
        }
    }

    let value = deserializer.deserialize_map(ValueVisitor)?;
    Ok(value)
}

pub fn serialize_u64_to_string<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = value.to_string();
    serializer.serialize_str(&s)
}

pub fn deserialize_string_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    struct U64Visitor;

    impl<'de> de::Visitor<'de> for U64Visitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a u64 integer or a string containing a u64 integer")
        }

        fn visit_u64<E>(self, value: u64) -> Result<u64, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<u64, E>
        where
            E: de::Error,
        {
            u64::try_from(value)
                .map_err(|_| E::custom(format!("i64 value {} is out of range for u64", value)))
        }

        fn visit_str<E>(self, value: &str) -> Result<u64, E>
        where
            E: de::Error,
        {
            value.parse::<u64>().map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_any(U64Visitor)
}

pub fn serialize_vec_u64_to_string<S>(value: &[u64], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.collect_seq(value.iter().map(u64::to_string))
}

pub fn deserialize_vec_string_to_vec_u64<'de, D>(deserializer: D) -> Result<Vec<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Element(u64);

    impl<'de> Deserialize<'de> for Element {
        fn deserialize<D2>(deserializer: D2) -> Result<Self, D2::Error>
        where
            D2: Deserializer<'de>,
        {
            deserialize_string_to_u64(deserializer).map(Element)
        }
    }

    let elements: Vec<Element> = Deserialize::deserialize(deserializer)?;
    Ok(elements.into_iter().map(|Element(value)| value).collect())
}

pub fn serialize_i64_to_string<S>(value: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = value.to_string();
    serializer.serialize_str(&s)
}

pub fn deserialize_string_to_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    struct I64Visitor;

    impl<'de> de::Visitor<'de> for I64Visitor {
        type Value = i64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an i64 integer or a string containing an i64 integer")
        }

        fn visit_i64<E>(self, value: i64) -> Result<i64, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_u64<E>(self, value: u64) -> Result<i64, E>
        where
            E: de::Error,
        {
            i64::try_from(value)
                .map_err(|_| E::custom(format!("u64 value {} is out of range for i64", value)))
        }

        fn visit_str<E>(self, value: &str) -> Result<i64, E>
        where
            E: de::Error,
        {
            value.parse::<i64>().map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_any(I64Visitor)
}

// Special serializer and deserializer for NaN, Infinity, and -Infinity
pub fn serialize_f64_special<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if value.is_nan() {
        serializer.serialize_str("NaN")
    } else if value.is_infinite() {
        if value.is_sign_positive() {
            serializer.serialize_str("Infinity")
        } else {
            serializer.serialize_str("-Infinity")
        }
    } else {
        serializer.serialize_f64(*value)
    }
}

pub fn deserialize_f64_special<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    struct F64Visitor;

    impl<'de> de::Visitor<'de> for F64Visitor {
        type Value = f64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a float or a string representing NaN, Infinity, or -Infinity")
        }

        fn visit_f64<E>(self, value: f64) -> Result<f64, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_u64<E>(self, value: u64) -> Result<f64, E>
        where
            E: de::Error,
        {
            Ok(value as f64)
        }

        fn visit_i64<E>(self, value: i64) -> Result<f64, E>
        where
            E: de::Error,
        {
            Ok(value as f64)
        }

        fn visit_str<E>(self, value: &str) -> Result<f64, E>
        where
            E: de::Error,
        {
            match value {
                "NaN" => Ok(f64::NAN),
                "Infinity" => Ok(f64::INFINITY),
                "-Infinity" => Ok(f64::NEG_INFINITY),
                _ => value.parse::<f64>().map_err(|_| {
                    E::custom(format!(
                        "invalid string for f64: expected a number, NaN, Infinity, or -Infinity but got '{}'",
                        value
                    ))
                }),
            }
        }
    }

    deserializer.deserialize_any(F64Visitor)
}
