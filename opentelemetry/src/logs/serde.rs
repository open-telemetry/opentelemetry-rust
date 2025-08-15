use std::{collections::HashMap, fmt};

use crate::{logs::AnyValue, Key, StringValue};
use serde::ser::{
    Error, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant, Serializer, StdError,
};

/// Serialize an arbitrary `serde::Serialize` into an `AnyValue`.
///
/// This method performs the following translations when converting between `serde`'s data model and OpenTelemetry's:
///
/// - Integers that don't fit in a `i64` are converted into strings.
/// - Unit types and nones are discarded (effectively treated as undefined).
/// - Struct and tuple variants are converted into an internally tagged map.
/// - Unit variants are converted into strings.
pub fn serialize(value: impl serde::Serialize) -> Option<AnyValue> {
    value.serialize(ValueSerializer).ok()?
}

struct ValueSerializer;

struct ValueSerializeSeq {
    value: Vec<AnyValue>,
}

struct ValueSerializeTuple {
    value: Vec<AnyValue>,
}

struct ValueSerializeTupleStruct {
    value: Vec<AnyValue>,
}

struct ValueSerializeMap {
    key: Option<Key>,
    value: HashMap<Key, AnyValue>,
}

struct ValueSerializeStruct {
    value: HashMap<Key, AnyValue>,
}

struct ValueSerializeTupleVariant {
    variant: &'static str,
    value: Vec<AnyValue>,
}

struct ValueSerializeStructVariant {
    variant: &'static str,
    value: HashMap<Key, AnyValue>,
}

#[derive(Debug)]
struct ValueError(String);

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Error for ValueError {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        ValueError(msg.to_string())
    }
}

impl StdError for ValueError {}

impl Serializer for ValueSerializer {
    type Ok = Option<AnyValue>;

    type Error = ValueError;

    type SerializeSeq = ValueSerializeSeq;

    type SerializeTuple = ValueSerializeTuple;

    type SerializeTupleStruct = ValueSerializeTupleStruct;

    type SerializeTupleVariant = ValueSerializeTupleVariant;

    type SerializeMap = ValueSerializeMap;

    type SerializeStruct = ValueSerializeStruct;

    type SerializeStructVariant = ValueSerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Some(AnyValue::Boolean(v)))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Some(AnyValue::Int(v)))
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        if let Ok(v) = v.try_into() {
            self.serialize_i64(v)
        } else {
            self.collect_str(&v)
        }
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        if let Ok(v) = v.try_into() {
            self.serialize_i64(v)
        } else {
            self.collect_str(&v)
        }
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        if let Ok(v) = v.try_into() {
            self.serialize_i64(v)
        } else {
            self.collect_str(&v)
        }
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Some(AnyValue::Double(v)))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.collect_str(&v)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Some(AnyValue::String(StringValue::from(v.to_owned()))))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(Some(AnyValue::Bytes(Box::new(v.to_owned()))))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(None)
    }

    fn serialize_some<T: serde::Serialize + ?Sized>(
        self,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(None)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        name.serialize(self)
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        variant.serialize(self)
    }

    fn serialize_newtype_struct<T: serde::Serialize + ?Sized>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: serde::Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        let mut map = self.serialize_map(Some(1))?;
        map.serialize_entry(variant, value)?;
        map.end()
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(ValueSerializeSeq { value: Vec::new() })
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(ValueSerializeTuple { value: Vec::new() })
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(ValueSerializeTupleStruct { value: Vec::new() })
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(ValueSerializeTupleVariant {
            variant,
            value: Vec::new(),
        })
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(ValueSerializeMap {
            key: None,
            value: HashMap::new(),
        })
    }

    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(ValueSerializeStruct {
            value: HashMap::new(),
        })
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(ValueSerializeStructVariant {
            variant,
            value: HashMap::new(),
        })
    }
}

impl SerializeSeq for ValueSerializeSeq {
    type Ok = Option<AnyValue>;

    type Error = ValueError;

    fn serialize_element<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        if let Some(value) = value.serialize(ValueSerializer)? {
            self.value.push(value);
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Some(AnyValue::ListAny(Box::new(self.value))))
    }
}

impl SerializeTuple for ValueSerializeTuple {
    type Ok = Option<AnyValue>;

    type Error = ValueError;

    fn serialize_element<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        if let Some(value) = value.serialize(ValueSerializer)? {
            self.value.push(value);
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Some(AnyValue::ListAny(Box::new(self.value))))
    }
}

impl SerializeTupleStruct for ValueSerializeTupleStruct {
    type Ok = Option<AnyValue>;

    type Error = ValueError;

    fn serialize_field<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        if let Some(value) = value.serialize(ValueSerializer)? {
            self.value.push(value);
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Some(AnyValue::ListAny(Box::new(self.value))))
    }
}

impl SerializeTupleVariant for ValueSerializeTupleVariant {
    type Ok = Option<AnyValue>;

    type Error = ValueError;

    fn serialize_field<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        if let Some(value) = value.serialize(ValueSerializer)? {
            self.value.push(value);
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Some(AnyValue::Map({
            let mut variant = Box::<HashMap<Key, AnyValue>>::default();
            variant.insert(
                Key::from(self.variant),
                AnyValue::ListAny(Box::new(self.value)),
            );
            variant
        })))
    }
}

impl SerializeMap for ValueSerializeMap {
    type Ok = Option<AnyValue>;

    type Error = ValueError;

    fn serialize_key<T: serde::Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Self::Error> {
        let key = match key.serialize(ValueSerializer)? {
            Some(AnyValue::String(key)) => Key::from(String::from(key)),
            key => Key::from(format!("{key:?}")),
        };

        self.key = Some(key);

        Ok(())
    }

    fn serialize_value<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        let key = self
            .key
            .take()
            .ok_or_else(|| Self::Error::custom("missing key"))?;

        if let Some(value) = value.serialize(ValueSerializer)? {
            self.value.insert(key, value);
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Some(AnyValue::Map(Box::new(self.value))))
    }
}

impl SerializeStruct for ValueSerializeStruct {
    type Ok = Option<AnyValue>;

    type Error = ValueError;

    fn serialize_field<T: serde::Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        let key = Key::from(key);

        if let Some(value) = value.serialize(ValueSerializer)? {
            self.value.insert(key, value);
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Some(AnyValue::Map(Box::new(self.value))))
    }
}

impl SerializeStructVariant for ValueSerializeStructVariant {
    type Ok = Option<AnyValue>;

    type Error = ValueError;

    fn serialize_field<T: serde::Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        let key = Key::from(key);

        if let Some(value) = value.serialize(ValueSerializer)? {
            self.value.insert(key, value);
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Some(AnyValue::Map({
            let mut variant = Box::<HashMap<Key, AnyValue>>::default();
            variant.insert(Key::from(self.variant), AnyValue::Map(Box::new(self.value)));
            variant
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::serialize;

    use crate::{logs::AnyValue, Key, StringValue};

    use std::collections::HashMap;

    #[test]
    fn test_serialize() {
        assert_eq!(None, serialize(None::<i32>));
        assert_eq!(None, serialize(()));

        assert_eq!(AnyValue::Int(1), serialize(1u8).unwrap());
        assert_eq!(AnyValue::Int(1), serialize(1u16).unwrap());
        assert_eq!(AnyValue::Int(1), serialize(1u32).unwrap());
        assert_eq!(AnyValue::Int(1), serialize(1u64).unwrap());
        assert_eq!(AnyValue::Int(1), serialize(1u128).unwrap());
        assert_eq!(AnyValue::Int(1), serialize(1i8).unwrap());
        assert_eq!(AnyValue::Int(1), serialize(1i16).unwrap());
        assert_eq!(AnyValue::Int(1), serialize(1i32).unwrap());
        assert_eq!(AnyValue::Int(1), serialize(1i64).unwrap());
        assert_eq!(AnyValue::Int(1), serialize(1i128).unwrap());

        assert_eq!(AnyValue::Double(1.0), serialize(1.0f32).unwrap());
        assert_eq!(AnyValue::Double(1.0), serialize(1.0f64).unwrap());

        assert_eq!(
            AnyValue::String(StringValue::from("a")),
            serialize("a").unwrap()
        );

        assert_eq!(
            AnyValue::String(StringValue::from("a")),
            serialize('a').unwrap()
        );

        assert_eq!(AnyValue::Boolean(true), serialize(true).unwrap());

        assert_eq!(
            AnyValue::Map({
                let mut map = Box::<HashMap<Key, AnyValue>>::default();

                map.insert(Key::from("a"), AnyValue::Int(1));
                map.insert(Key::from("b"), AnyValue::Int(1));
                map.insert(Key::from("c"), AnyValue::Int(1));

                map
            }),
            serialize({
                let mut map = HashMap::<&str, i32>::default();

                map.insert("a", 1);
                map.insert("b", 1);
                map.insert("c", 1);

                map
            })
            .unwrap(),
        );

        assert_eq!(
            AnyValue::ListAny(Box::new(vec![
                AnyValue::Int(1),
                AnyValue::Int(1),
                AnyValue::Int(1)
            ])),
            serialize(vec![1, 1, 1]).unwrap(),
        );

        assert_eq!(
            AnyValue::ListAny(Box::new(vec![
                AnyValue::Int(1),
                AnyValue::Int(1),
                AnyValue::Int(1)
            ])),
            serialize((1, 1, 1)).unwrap(),
        );

        #[derive(serde::Serialize)]
        struct Newtype(i32);

        assert_eq!(AnyValue::Int(1), serialize(Newtype(1)).unwrap());

        #[derive(serde::Serialize)]
        enum Enum {
            Unit,
            Newtype(i32),
            Struct { a: i32, b: i32, c: i32 },
            Tuple(i32, i32, i32),
        }

        assert_eq!(
            AnyValue::String(StringValue::from("Unit")),
            serialize(Enum::Unit).unwrap()
        );

        assert_eq!(
            AnyValue::Map({
                let mut map = HashMap::new();

                map.insert(Key::from("Newtype"), AnyValue::Int(42));

                Box::new(map)
            }),
            serialize(Enum::Newtype(42)).unwrap()
        );

        assert_eq!(
            AnyValue::Map({
                let mut map = HashMap::new();

                map.insert(
                    Key::from("Struct"),
                    AnyValue::Map(Box::new({
                        let mut map = HashMap::new();
                        map.insert(Key::from("a"), AnyValue::Int(1));
                        map.insert(Key::from("b"), AnyValue::Int(1));
                        map.insert(Key::from("c"), AnyValue::Int(1));
                        map
                    })),
                );

                Box::new(map)
            }),
            serialize(Enum::Struct { a: 1, b: 1, c: 1 }).unwrap(),
        );

        assert_eq!(
            AnyValue::Map({
                let mut map = HashMap::new();

                map.insert(
                    Key::from("Tuple"),
                    AnyValue::ListAny(Box::new(vec![
                        AnyValue::Int(1),
                        AnyValue::Int(1),
                        AnyValue::Int(1),
                    ])),
                );

                Box::new(map)
            }),
            serialize(Enum::Tuple(1, 1, 1)).unwrap(),
        );

        struct Bytes<B>(B);

        impl<B: AsRef<[u8]>> serde::Serialize for Bytes<B> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_bytes(self.0.as_ref())
            }
        }

        assert_eq!(
            AnyValue::Bytes(Box::new(vec![1, 2, 3])),
            serialize(Bytes(vec![1, 2, 3])).unwrap(),
        );

        struct Map {
            a: i32,
            b: i32,
            c: i32,
        }

        impl serde::Serialize for Map {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                use serde::ser::SerializeMap;

                let mut map = serializer.serialize_map(Some(3))?;

                map.serialize_entry(&"a", &self.a)?;
                map.serialize_entry(&"b", &self.b)?;
                map.serialize_entry(&"c", &self.c)?;

                map.end()
            }
        }

        assert_eq!(
            AnyValue::Map({
                let mut map = Box::<HashMap<Key, AnyValue>>::default();

                map.insert(Key::from("a"), AnyValue::Int(1));
                map.insert(Key::from("b"), AnyValue::Int(1));
                map.insert(Key::from("c"), AnyValue::Int(1));

                map
            }),
            serialize(&Map { a: 1, b: 1, c: 1 }).unwrap(),
        );
    }
}
