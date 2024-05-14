/// provide serde support for proto traceIds and spanIds.
/// Those are hex encoded strings in the jsons but they are byte arrays in the proto.
/// See https://opentelemetry.io/docs/specs/otlp/#json-protobuf-encoding for more details
#[cfg(all(feature = "with-serde", feature = "gen-tonic-messages"))]
pub(crate) mod serializers {
    use crate::tonic::common::v1::any_value::{self, Value};
    use crate::tonic::common::v1::AnyValue;
    use serde::de::{self, MapAccess, Visitor};
    use serde::ser::SerializeStruct;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::fmt;

    // hex string <-> bytes conversion

    pub fn serialize_to_hex_string<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_string = hex::encode(bytes);
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
                hex::decode(value).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(BytesVisitor)
    }

    // AnyValue <-> KeyValue conversion
    pub fn serialize_to_value<S>(value: &Option<AnyValue>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        // Serialize any_value::Value using its own implementation
        // If value is None, it will be serialized as such
        match value {
            Some(any_value) => match &any_value.value {
                Some(Value::IntValue(i)) => serialize_i64_to_string(i, serializer),
                Some(value) => value.serialize(serializer),
                None => serializer.serialize_none(),
            },
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize_from_value<'de, D>(deserializer: D) -> Result<Option<AnyValue>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ValueVisitor;

    impl<'de> de::Visitor<'de> for ValueVisitor {
        type Value = AnyValue;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a JSON object for AnyValue")
        }

        fn visit_map<V>(self, mut map: V) -> Result<AnyValue, V::Error>
        where
            V: de::MapAccess<'de>,
        {
            let mut value: Option<any_value::Value> = None;

            while let Some(key) = map.next_key::<String>()? {
                let key_str = key.as_str();
                match key_str {
                    "stringValue" => {
                        let s = map.next_value()?;
                        value = Some(any_value::Value::StringValue(s));
                    },
                    "boolValue" => {
                        let b = map.next_value()?;
                        value = Some(any_value::Value::BoolValue(b));
                    },
                    "intValue" => {
                        let value_str = map.next_value::<String>()?;
                        let int_value = value_str.parse::<i64>()
                            .map_err(de::Error::custom)?;
                        value = Some(any_value::Value::IntValue(int_value));
                    },
                    "doubleValue" => {
                        let d = map.next_value()?;
                        value = Some(any_value::Value::DoubleValue(d));
                    },
                    "arrayValue" => {
                        let a = map.next_value()?;
                        value = Some(any_value::Value::ArrayValue(a));
                    },
                    "kvlistValue" => {
                        let kv = map.next_value()?;
                        value = Some(any_value::Value::KvlistValue(kv));
                    },
                    "bytesValue" => {
                        let bytes = map.next_value()?;
                        value = Some(any_value::Value::BytesValue(bytes));
                    },
                    _ => {
                        //skip unknown keys, and handle error later.
                        continue
                    }
                }
            }

            if let Some(v) = value {
                Ok(AnyValue { value: Some(v) })
            } else {
                Err(de::Error::custom("Invalid data for AnyValue, no known keys found"))
            }
        }
    }

    let value = deserializer.deserialize_map(ValueVisitor)?;
    Ok(Some(value))
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
        let s: String = Deserialize::deserialize(deserializer)?;
        s.parse::<u64>().map_err(de::Error::custom)
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
        let s: String = Deserialize::deserialize(deserializer)?;
        s.parse::<i64>().map_err(de::Error::custom)
    }
}

#[cfg(feature = "gen-tonic-messages")]
#[path = "proto/tonic"]
/// Generated files using [`tonic`](https://docs.rs/crate/tonic) and [`prost`](https://docs.rs/crate/prost)
pub mod tonic {
    /// Service stub and clients
    #[path = ""]
    pub mod collector {
        #[cfg(feature = "logs")]
        #[path = ""]
        pub mod logs {
            #[path = "opentelemetry.proto.collector.logs.v1.rs"]
            pub mod v1;
        }

        #[cfg(feature = "metrics")]
        #[path = ""]
        pub mod metrics {
            #[path = "opentelemetry.proto.collector.metrics.v1.rs"]
            pub mod v1;
        }

        #[cfg(feature = "trace")]
        #[path = ""]
        pub mod trace {
            #[path = "opentelemetry.proto.collector.trace.v1.rs"]
            pub mod v1;
        }
    }

    /// Common types used across all signals
    #[path = ""]
    pub mod common {
        #[path = "opentelemetry.proto.common.v1.rs"]
        pub mod v1;
    }

    /// Generated types used in logging.
    #[cfg(feature = "logs")]
    #[path = ""]
    pub mod logs {
        #[path = "opentelemetry.proto.logs.v1.rs"]
        pub mod v1;
    }

    /// Generated types used in metrics.
    #[cfg(feature = "metrics")]
    #[path = ""]
    pub mod metrics {
        #[path = "opentelemetry.proto.metrics.v1.rs"]
        pub mod v1;
    }

    /// Generated types used in resources.
    #[path = ""]
    pub mod resource {
        #[path = "opentelemetry.proto.resource.v1.rs"]
        pub mod v1;
    }

    /// Generated types used in traces.
    #[cfg(feature = "trace")]
    #[path = ""]
    pub mod trace {
        #[path = "opentelemetry.proto.trace.v1.rs"]
        pub mod v1;
    }

    /// Generated types used in zpages.
    #[cfg(feature = "zpages")]
    #[path = ""]
    pub mod tracez {
        #[path = "opentelemetry.proto.tracez.v1.rs"]
        pub mod v1;
    }

    pub use crate::transform::common::tonic::Attributes;
}
