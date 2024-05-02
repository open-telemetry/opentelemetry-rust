//! Bridge `log` into OpenTelemetry.
//!
//! This library implements a log appender for the [`log`] crate using the [Logs Bridge API].
//!
//! # Getting Started
//!
//! The bridge requires configuration on both the `log` and OpenTelemetry sides.
//!
//! For OpenTelemetry, configure a [`LoggerProvider`] with the desired exporter:
//!
//! ```
//! # #[tokio::main] async fn main() {
//! # use opentelemetry_sdk::logs::{BatchLogProcessor, LoggerProvider};
//! # use opentelemetry_sdk::runtime;
//! let exporter = opentelemetry_stdout::LogExporterBuilder::default().build();
//!
//! let logger_provider = LoggerProvider::builder()
//!     .with_log_processor(BatchLogProcessor::builder(exporter, runtime::Tokio).build())
//!     .build();
//! # }
//! ```
//!
//! For `log`, set the global logger to an [`OpenTelemetryLogBridge`] instance using the `LoggerProvider`:
//!
//! ```
//! # #[tokio::main] async fn main() {
//! # use opentelemetry_sdk::logs::{BatchLogProcessor, LoggerProvider};
//! # use opentelemetry_sdk::runtime;
//! # use opentelemetry_appender_log::OpenTelemetryLogBridge;
//! # let exporter = opentelemetry_stdout::LogExporterBuilder::default().build();
//! # let logger_provider = LoggerProvider::builder()
//! #     .with_log_processor(BatchLogProcessor::builder(exporter, runtime::Tokio).build())
//! #     .build();
//! let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);
//!
//! log::set_boxed_logger(Box::new(otel_log_appender)).unwrap();
//! # }
//! ```
//!
//! # Mapping Log Records
//!
//! This section outlines how log records produced by `log` are mapped into OpenTelemetry log records.
//! Each subsection deals with a different property on `opentelemetry::logs::LogRecord`.
//!
//! ## Body
//!
//! The body is the stringified message ([`log::Record::args`]).
//!
//! ## Severity
//!
//! The severity number and text are mapped from the [`log::Level`] ([`log::Record::level`]):
//!
//! | `log::Level` | Severity Text | Severity Number |
//! | ------------ | ------------- | --------------- |
//! | `Error`      | Error         | 17              |
//! | `Warn`       | Warn          | 13              |
//! | `Info`       | Info          | 9               |
//! | `Debug`      | Debug         | 5               |
//! | `Trace`      | Trace         | 1               |
//!
//! # Attributes
//!
//! Any key-values ([`log::Record::key_values`]) are converted into attributes:
//!
//! | Type            | Result                | Notes                                                                                                                   |
//! | --------------- | --------------------- | ----------------------------------------------------------------------------------------------------------------------- |
//! | `i8`-`i128`     | [`AnyValue::Int`]     | If the value is too big then it will be stringified using [`std::fmt::Display`]                                         |
//! | `u8`-`u128`     | [`AnyValue::Int`]     | If the value is too big then it will be stringified using [`std::fmt::Display`]                                         |
//! | `f32`-`f64`     | [`AnyValue::Double`]  |                                                                                                                         |
//! | `bool`          | [`AnyValue::Boolean`] |                                                                                                                         |
//! | `str`           | [`AnyValue::String`]  |                                                                                                                         |
//! | Bytes           | [`AnyValue::Bytes`]   | Requires the `with-serde` feature, otherwise it will be stringified using [`std::fmt::Debug`]                           |
//! | `()`            | -                     | Unit values are discared                                                                                                |
//! | `Some`          | Any                   | `Some` variants use their inner value                                                                                   |
//! | `None`          | -                     | `None` variants are discared                                                                                            |
//! | Unit struct     | [`AnyValue::String`]  | Uses the name of the struct                                                                                             |
//! | Unit variant    | [`AnyValue::String`]  | Uses the name of the variant                                                                                            |
//! | Newtype struct  | Any                   | Uses the inner value of the newtype                                                                                     |
//! | Newtype variant | [`AnyValue::Map`]     | An internally-tagged map. Requires the `with-serde` feature, otherwise it will be stringified using [`std::fmt::Debug`] |
//! | Sequence        | [`AnyValue::ListAny`] | Requires the `with-serde` feature, otherwise it will be stringified using [`std::fmt::Debug`]                           |
//! | Tuple           | [`AnyValue::ListAny`] | Requires the `with-serde` feature, otherwise it will be stringified using [`std::fmt::Debug`]                           |
//! | Tuple struct    | [`AnyValue::ListAny`] | Requires the `with-serde` feature, otherwise it will be stringified using [`std::fmt::Debug`]                           |
//! | Tuple variant   | [`AnyValue::Map`]     | An internally-tagged map. Requires the `with-serde` feature, otherwise it will be stringified using [`std::fmt::Debug`] |
//! | Map             | [`AnyValue::Map`]     | Requires the `with-serde` feature, otherwise it will be stringified using [`std::fmt::Debug`]                           |
//! | Struct          | [`AnyValue::Map`]     | Requires the `with-serde` feature, otherwise it will be stringified using [`std::fmt::Debug`]                           |
//! | Struct variant  | [`AnyValue::Map`]     | An internally-tagged map. Requires the `with-serde` feature, otherwise it will be stringified using [`std::fmt::Debug`] |
//!
//! # Feature Flags
//!
//! This library provides the following Cargo features:
//!
//! - `logs_level_enabled`: Allow users to control the log level.
//! - `with-serde`: Support complex values as attributes without stringifying them.
//!
//! [Logs Bridge API]: https://opentelemetry.io/docs/specs/otel/logs/bridge-api/

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
            let log_record = self
                .logger
                .create_log_record()
                .with_severity_number(severity_of_level(record.level()))
                .with_severity_text(record.level().as_str().into())
                .with_body(AnyValue::from(record.args().to_string()))
                .with_attributes(log_attributes(record.key_values()))
                .build();

            self.logger.emit(log_record);
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
            logger: provider
                .logger_builder("opentelemetry-log-appender")
                .with_version(Cow::Borrowed(env!("CARGO_PKG_VERSION")))
                .build(),
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

#[cfg(not(feature = "with-serde"))]
mod any_value {
    use opentelemetry::{logs::AnyValue, StringValue};

    pub(crate) fn serialize(value: log::kv::Value) -> Option<AnyValue> {
        struct ValueVisitor(Option<AnyValue>);

        impl<'kvs> log::kv::VisitValue<'kvs> for ValueVisitor {
            fn visit_any(&mut self, value: log::kv::Value) -> Result<(), log::kv::Error> {
                self.0 = Some(AnyValue::String(StringValue::from(value.to_string())));

                Ok(())
            }

            fn visit_bool(&mut self, value: bool) -> Result<(), log::kv::Error> {
                self.0 = Some(AnyValue::Boolean(value));

                Ok(())
            }

            fn visit_str(&mut self, value: &str) -> Result<(), log::kv::Error> {
                self.0 = Some(AnyValue::String(StringValue::from(value.to_owned())));

                Ok(())
            }

            fn visit_i64(&mut self, value: i64) -> Result<(), log::kv::Error> {
                self.0 = Some(AnyValue::Int(value));

                Ok(())
            }

            fn visit_u64(&mut self, value: u64) -> Result<(), log::kv::Error> {
                if let Ok(value) = value.try_into() {
                    self.visit_i64(value)
                } else {
                    self.visit_any(log::kv::Value::from(value))
                }
            }

            fn visit_i128(&mut self, value: i128) -> Result<(), log::kv::Error> {
                if let Ok(value) = value.try_into() {
                    self.visit_i64(value)
                } else {
                    self.visit_any(log::kv::Value::from(value))
                }
            }

            fn visit_u128(&mut self, value: u128) -> Result<(), log::kv::Error> {
                if let Ok(value) = value.try_into() {
                    self.visit_i64(value)
                } else {
                    self.visit_any(log::kv::Value::from(value))
                }
            }

            fn visit_f64(&mut self, value: f64) -> Result<(), log::kv::Error> {
                self.0 = Some(AnyValue::Double(value));

                Ok(())
            }
        }

        let mut visitor = ValueVisitor(None);
        value.visit(&mut visitor).unwrap();
        visitor.0
    }
}

// This could make a nice addition to the SDK itself for serializing into `AnyValue`s
#[cfg(feature = "with-serde")]
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
    use super::OpenTelemetryLogBridge;

    use opentelemetry::{logs::AnyValue, StringValue};
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
        assert!(otel_log_appender.enabled(&log::Metadata::builder().build()));
        #[cfg(not(feature = "logs_level_enabled"))]
        assert!(otel_log_appender.enabled(&log::Metadata::builder().build()));
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
        struct Struct {
            a: i32,
            b: i32,
            c: i32,
        }

        #[derive(serde::Serialize)]
        struct Newtype(i32);

        #[derive(serde::Serialize)]
        enum Enum {
            Unit,
            Newtype(i32),
            Struct { a: i32, b: i32, c: i32 },
            Tuple(i32, i32, i32),
        }

        struct Bytes<B>(B);

        impl<B: AsRef<[u8]>> serde::Serialize for Bytes<B> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_bytes(self.0.as_ref())
            }
        }

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

        let exporter = InMemoryLogsExporter::default();

        let logger_provider = LoggerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();

        let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);

        otel_log_appender.log(
            &log::RecordBuilder::new()
                .level(log::Level::Info)
                .args(format_args!("body"))
                .key_values(&[
                    ("str_value", log::kv::Value::from("a string")),
                    ("u8_value", log::kv::Value::from(1u8)),
                    ("u16_value", log::kv::Value::from(2u16)),
                    ("u32_value", log::kv::Value::from(42u32)),
                    ("u64_value", log::kv::Value::from(2147483660u64)),
                    ("u128_small_value", log::kv::Value::from(2147483660u128)),
                    (
                        "u128_big_value",
                        log::kv::Value::from(9223372036854775820u128),
                    ),
                    ("i8_value", log::kv::Value::from(1i8)),
                    ("i16_value", log::kv::Value::from(2i16)),
                    ("i32_value", log::kv::Value::from(42i32)),
                    ("i64_value", log::kv::Value::from(2147483660i64)),
                    ("i128_small_value", log::kv::Value::from(2147483660i128)),
                    (
                        "i128_big_value",
                        log::kv::Value::from(9223372036854775820i128),
                    ),
                    ("f64_value", log::kv::Value::from(4.2f64)),
                    ("bool_value", log::kv::Value::from(true)),
                    ("bytes_value", log::kv::Value::from_serde(&Bytes([1, 1, 1]))),
                    ("unit_value", log::kv::Value::from_serde(&())),
                    ("some_value", log::kv::Value::from_serde(&Some(42))),
                    ("none_value", log::kv::Value::from_serde(&None::<i32>)),
                    (
                        "slice_value",
                        log::kv::Value::from_serde(&(&[1, 1, 1] as &[i32])),
                    ),
                    (
                        "map_value",
                        log::kv::Value::from_serde(&Map { a: 1, b: 1, c: 1 }),
                    ),
                    (
                        "struct_value",
                        log::kv::Value::from_serde(&Struct { a: 1, b: 1, c: 1 }),
                    ),
                    ("tuple_value", log::kv::Value::from_serde(&(1, 1, 1))),
                    ("newtype_value", log::kv::Value::from_serde(&Newtype(42))),
                    (
                        "unit_variant_value",
                        log::kv::Value::from_serde(&Enum::Unit),
                    ),
                    (
                        "newtype_variant_value",
                        log::kv::Value::from_serde(&Enum::Newtype(42)),
                    ),
                    (
                        "struct_variant_value",
                        log::kv::Value::from_serde(&Enum::Struct { a: 1, b: 1, c: 1 }),
                    ),
                    (
                        "tuple_variant_value",
                        log::kv::Value::from_serde(&Enum::Tuple(1, 1, 1)),
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
            get("str_value").unwrap()
        );

        assert_eq!(AnyValue::Int(1), get("i8_value").unwrap());
        assert_eq!(AnyValue::Int(2), get("i16_value").unwrap());
        assert_eq!(AnyValue::Int(42), get("i32_value").unwrap());
        assert_eq!(AnyValue::Int(2147483660), get("i64_value").unwrap());
        assert_eq!(AnyValue::Int(2147483660), get("i128_small_value").unwrap());
        assert_eq!(
            AnyValue::String(StringValue::from("9223372036854775820")),
            get("i128_big_value").unwrap()
        );

        assert_eq!(AnyValue::Double(4.2), get("f64_value").unwrap());

        assert_eq!(AnyValue::Boolean(true), get("bool_value").unwrap());

        #[cfg(not(feature = "with-serde"))]
        {
            assert_eq!(
                AnyValue::String(StringValue::from("[1, 1, 1]")),
                get("slice_value").unwrap()
            );

            assert_eq!(
                AnyValue::String(StringValue::from("{\"a\": 1, \"b\": 1, \"c\": 1}")),
                get("map_value").unwrap()
            );

            assert_eq!(
                AnyValue::String(StringValue::from("Struct { a: 1, b: 1, c: 1 }")),
                get("struct_value").unwrap()
            );

            assert_eq!(
                AnyValue::String(StringValue::from("(1, 1, 1)")),
                get("tuple_value").unwrap()
            );

            assert_eq!(
                AnyValue::String(StringValue::from("Newtype(42)")),
                get("newtype_value").unwrap()
            );

            assert_eq!(
                AnyValue::String(StringValue::from("Unit")),
                get("unit_variant_value").unwrap()
            );

            assert_eq!(
                AnyValue::String(StringValue::from("Newtype(42)")),
                get("newtype_variant_value").unwrap()
            );

            assert_eq!(
                AnyValue::String(StringValue::from("Struct { a: 1, b: 1, c: 1 }")),
                get("struct_variant_value").unwrap()
            );

            assert_eq!(
                AnyValue::String(StringValue::from("Tuple(1, 1, 1)")),
                get("tuple_variant_value").unwrap()
            );
        }
        #[cfg(feature = "with-serde")]
        {
            use opentelemetry::Key;
            use std::collections::HashMap;

            assert_eq!(None, get("unit_value"));
            assert_eq!(None, get("none_value"));
            assert_eq!(AnyValue::Int(42), get("some_value").unwrap());

            assert_eq!(
                AnyValue::ListAny(vec![AnyValue::Int(1), AnyValue::Int(1), AnyValue::Int(1)]),
                get("slice_value").unwrap()
            );

            assert_eq!(
                AnyValue::Map({
                    let mut map = HashMap::new();

                    map.insert(Key::from("a"), AnyValue::Int(1));
                    map.insert(Key::from("b"), AnyValue::Int(1));
                    map.insert(Key::from("c"), AnyValue::Int(1));

                    map
                }),
                get("map_value").unwrap()
            );

            assert_eq!(
                AnyValue::Map({
                    let mut map = HashMap::new();

                    map.insert(Key::from("a"), AnyValue::Int(1));
                    map.insert(Key::from("b"), AnyValue::Int(1));
                    map.insert(Key::from("c"), AnyValue::Int(1));

                    map
                }),
                get("struct_value").unwrap()
            );

            assert_eq!(
                AnyValue::ListAny(vec![AnyValue::Int(1), AnyValue::Int(1), AnyValue::Int(1)]),
                get("tuple_value").unwrap()
            );

            assert_eq!(
                AnyValue::String(StringValue::from("Unit")),
                get("unit_variant_value").unwrap()
            );

            assert_eq!(
                AnyValue::Map({
                    let mut map = HashMap::new();

                    map.insert(Key::from("Newtype"), AnyValue::Int(42));

                    map
                }),
                get("newtype_variant_value").unwrap()
            );

            assert_eq!(
                AnyValue::Map({
                    let mut map = HashMap::new();

                    map.insert(
                        Key::from("Struct"),
                        AnyValue::Map({
                            let mut map = HashMap::new();

                            map.insert(Key::from("a"), AnyValue::Int(1));
                            map.insert(Key::from("b"), AnyValue::Int(1));
                            map.insert(Key::from("c"), AnyValue::Int(1));

                            map
                        }),
                    );

                    map
                }),
                get("struct_variant_value").unwrap()
            );

            assert_eq!(
                AnyValue::Map({
                    let mut map = HashMap::new();

                    map.insert(
                        Key::from("Tuple"),
                        AnyValue::ListAny(vec![
                            AnyValue::Int(1),
                            AnyValue::Int(1),
                            AnyValue::Int(1),
                        ]),
                    );

                    map
                }),
                get("tuple_variant_value").unwrap()
            );
        }
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
