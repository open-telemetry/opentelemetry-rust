use log::{Level, Metadata, Record};
use opentelemetry::{
    logs::{AnyValue, LogRecordBuilder, Logger, LoggerProvider, Severity},
    Key,
};
use std::borrow::Cow;

pub struct OpenTelemetryLogBridge<P, L>
where
    P: LoggerProvider<Logger = L> + Send + Sync,
    L: Logger + Send + Sync,
{
    logger: L,
    _phantom: std::marker::PhantomData<P>, // P is not used in this struct
}

impl<P, L> log::Log for OpenTelemetryLogBridge<P, L>
where
    P: LoggerProvider<Logger = L> + Send + Sync,
    L: Logger + Send + Sync,
{
    fn enabled(&self, _metadata: &Metadata) -> bool {
        #[cfg(feature = "logs_level_enabled")]
        return self
            .logger
            .event_enabled(severity_of_level(_metadata.level()), _metadata.target());
        #[cfg(not(feature = "logs_level_enabled"))]
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            self.logger.emit(
                LogRecordBuilder::new()
                    .with_severity_number(severity_of_level(record.level()))
                    .with_severity_text(record.level().as_str())
                    // Not populating ObservedTimestamp, instead relying on OpenTelemetry
                    // API to populate it with current time.
                    .with_body(AnyValue::from(record.args().to_string()))
                    .with_attributes(log_attributes(record.key_values()))
                    .build(),
            );
        }
    }

    fn flush(&self) {}
}

impl<P, L> OpenTelemetryLogBridge<P, L>
where
    P: LoggerProvider<Logger = L> + Send + Sync,
    L: Logger + Send + Sync,
{
    pub fn new(provider: &P) -> Self {
        OpenTelemetryLogBridge {
            logger: provider.versioned_logger(
                "opentelemetry-log-appender",
                Some(Cow::Borrowed(env!("CARGO_PKG_VERSION"))),
                None,
                None,
            ),
            _phantom: Default::default(),
        }
    }
}

const fn severity_of_level(level: Level) -> Severity {
    match level {
        Level::Error => Severity::Error,
        Level::Warn => Severity::Warn,
        Level::Info => Severity::Info,
        Level::Debug => Severity::Debug,
        Level::Trace => Severity::Trace,
    }
}

fn log_attributes(kvs: impl log::kv::Source) -> Vec<(Key, AnyValue)> {
    struct AttributeVisitor(Vec<(Key, AnyValue)>);

    impl<'kvs> log::kv::VisitSource<'kvs> for AttributeVisitor {
        fn visit_pair(
            &mut self,
            key: log::kv::Key<'kvs>,
            value: log::kv::Value<'kvs>,
        ) -> Result<(), log::kv::Error> {
            let key = Key::from(String::from(key.as_str()));

            if let Some(value) = any_value::serialize(value) {
                self.0.push((key, value));
            }

            Ok(())
        }
    }

    let mut visitor = AttributeVisitor(Vec::new());

    let _ = kvs.visit(&mut visitor);

    visitor.0
}

// This could make a nice addition to the SDK itself for serializing into `AnyValue`s
mod any_value {
    use std::{collections::HashMap, fmt};

    use opentelemetry::{logs::AnyValue, Key, StringValue};
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
    pub(crate) fn serialize(value: impl serde::Serialize) -> Option<AnyValue> {
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
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
            Ok(Some(AnyValue::Bytes(v.to_owned())))
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
            Ok(Some(AnyValue::ListAny(self.value)))
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
            Ok(Some(AnyValue::ListAny(self.value)))
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
            Ok(Some(AnyValue::ListAny(self.value)))
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
                let mut variant = HashMap::new();
                variant.insert(Key::from(self.variant), AnyValue::ListAny(self.value));
                variant
            })))
        }
    }

    impl SerializeMap for ValueSerializeMap {
        type Ok = Option<AnyValue>;

        type Error = ValueError;

        fn serialize_key<T: serde::Serialize + ?Sized>(
            &mut self,
            key: &T,
        ) -> Result<(), Self::Error> {
            let key = match key.serialize(ValueSerializer)? {
                Some(AnyValue::String(key)) => Key::from(String::from(key)),
                key => Key::from(format!("{:?}", key)),
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
            Ok(Some(AnyValue::Map(self.value)))
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
            Ok(Some(AnyValue::Map(self.value)))
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
                let mut variant = HashMap::new();
                variant.insert(Key::from(self.variant), AnyValue::Map(self.value));
                variant
            })))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::OpenTelemetryLogBridge;

    use opentelemetry::{logs::AnyValue, Key, StringValue};
    use opentelemetry_sdk::{logs::LoggerProvider, testing::logs::InMemoryLogsExporter};

    use log::Log;

    #[test]
    fn logbridge_with_default_metadata_is_enabled() {
        let exporter = InMemoryLogsExporter::default();

        let logger_provider = LoggerProvider::builder()
            .with_simple_exporter(exporter)
            .build();

        let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);

        // As a result of using `with_simple_exporter` while building the logger provider,
        // the processor used is a `SimpleLogProcessor` which has an implementation of `event_enabled`
        // that always returns true.
        #[cfg(feature = "logs_level_enabled")]
        assert_eq!(
            otel_log_appender.enabled(&log::Metadata::builder().build()),
            true
        );
        #[cfg(not(feature = "logs_level_enabled"))]
        assert_eq!(
            otel_log_appender.enabled(&log::Metadata::builder().build()),
            true
        );
    }

    #[test]
    fn logbridge_with_record_can_log() {
        let exporter = InMemoryLogsExporter::default();

        let logger_provider = LoggerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();

        let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);

        // log::trace!("TRACE")
        otel_log_appender.log(
            &log::RecordBuilder::new()
                .level(log::Level::Trace)
                .args(format_args!("TRACE"))
                .build(),
        );

        // log::trace!("DEBUG")
        otel_log_appender.log(
            &log::RecordBuilder::new()
                .level(log::Level::Debug)
                .args(format_args!("DEBUG"))
                .build(),
        );

        // log::trace!("INFO")
        otel_log_appender.log(
            &log::RecordBuilder::new()
                .level(log::Level::Info)
                .args(format_args!("INFO"))
                .build(),
        );

        // log::trace!("WARN")
        otel_log_appender.log(
            &log::RecordBuilder::new()
                .level(log::Level::Warn)
                .args(format_args!("WARN"))
                .build(),
        );

        // log::trace!("ERROR")
        otel_log_appender.log(
            &log::RecordBuilder::new()
                .level(log::Level::Error)
                .args(format_args!("ERROR"))
                .build(),
        );

        let logs = exporter.get_emitted_logs().unwrap();

        assert_eq!(logs.len(), 5);
        for log in logs {
            let body: String = match log.record.body.as_ref().unwrap() {
                super::AnyValue::String(s) => s.to_string(),
                _ => panic!("AnyValue::String expected"),
            };
            assert_eq!(body, log.record.severity_text.unwrap());
        }
    }

    #[test]
    fn logbridge_attributes() {
        #[derive(serde::Serialize)]
        struct Map {
            a: i32,
            b: i32,
            c: i32,
        }

        let exporter = InMemoryLogsExporter::default();

        let logger_provider = LoggerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();

        let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);

        /*log::info!(
            string_value = "a string",
            int_value = 42,
            double_value = 3.14,
            boolean_value = true,
            list_value:serde = [1, 2, 3],
            map_value:serde = Map { a: 1, b: 2, c: 3 };
            "body"
        );*/
        otel_log_appender.log(
            &log::RecordBuilder::new()
                .level(log::Level::Info)
                .args(format_args!("body"))
                .key_values(&[
                    ("string_value", log::kv::Value::from("a string")),
                    ("int_value", log::kv::Value::from(42)),
                    ("double_value", log::kv::Value::from(3.14)),
                    ("boolean_value", log::kv::Value::from(true)),
                    ("list_value", log::kv::Value::from_serde(&[1, 2, 3])),
                    (
                        "map_value",
                        log::kv::Value::from_serde(&Map { a: 1, b: 2, c: 3 }),
                    ),
                ])
                .build(),
        );

        let logs = exporter.get_emitted_logs().unwrap();
        let attributes = &logs[0].record.attributes.as_ref().unwrap();

        let get = |needle: &str| {
            attributes.iter().find_map(|(k, v)| {
                if k.as_str() == needle {
                    Some(v.clone())
                } else {
                    None
                }
            })
        };

        assert_eq!(
            AnyValue::String(StringValue::from("a string")),
            get("string_value").unwrap()
        );

        assert_eq!(AnyValue::Int(42), get("int_value").unwrap());

        assert_eq!(AnyValue::Double(3.14), get("double_value").unwrap());

        assert_eq!(AnyValue::Boolean(true), get("boolean_value").unwrap());

        assert_eq!(
            AnyValue::ListAny(vec![AnyValue::Int(1), AnyValue::Int(2), AnyValue::Int(3),]),
            get("list_value").unwrap()
        );

        assert_eq!(
            AnyValue::Map({
                let mut map = HashMap::new();

                map.insert(Key::from("a"), AnyValue::Int(1));
                map.insert(Key::from("b"), AnyValue::Int(2));
                map.insert(Key::from("c"), AnyValue::Int(3));

                map
            }),
            get("map_value").unwrap()
        );
    }

    #[test]
    fn test_flush() {
        let exporter = InMemoryLogsExporter::default();

        let logger_provider = LoggerProvider::builder()
            .with_simple_exporter(exporter)
            .build();

        let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);
        otel_log_appender.flush();
    }

    #[test]
    fn check_level_severities() {
        assert_eq!(
            super::severity_of_level(log::Level::Error),
            opentelemetry::logs::Severity::Error
        );
        assert_eq!(
            super::severity_of_level(log::Level::Warn),
            opentelemetry::logs::Severity::Warn
        );
        assert_eq!(
            super::severity_of_level(log::Level::Info),
            opentelemetry::logs::Severity::Info
        );
        assert_eq!(
            super::severity_of_level(log::Level::Debug),
            opentelemetry::logs::Severity::Debug
        );
        assert_eq!(
            super::severity_of_level(log::Level::Trace),
            opentelemetry::logs::Severity::Trace
        );
    }
}
