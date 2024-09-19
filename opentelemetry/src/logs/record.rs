use crate::{Array, Key, StringValue, Value};
use std::{borrow::Cow, collections::HashMap, time::SystemTime};

/// SDK implemented trait for managing log records
pub trait LogRecord<'a> {
    /// Sets the `event_name` of a record
    fn set_event_name(&mut self, name: &'static str);

    /// Sets the `target` of a record.
    /// Currently, both `opentelemetry-appender-tracing` and `opentelemetry-appender-log` create a single logger
    /// with a scope that doesn't accurately reflect the component emitting the logs.
    /// Exporters MAY use this field to override the `instrumentation_scope.name`.
    fn set_target<T>(&mut self, _target: T)
    where
        T: Into<Cow<'static, str>>;

    /// Sets the time when the event occurred measured by the origin clock, i.e. the time at the source.
    fn set_timestamp(&mut self, timestamp: SystemTime);

    /// Sets the observed event timestamp.
    fn set_observed_timestamp(&mut self, timestamp: SystemTime);

    /// Sets severity as text.
    fn set_severity_text(&mut self, text: &'static str);

    /// Sets severity as a numeric value.
    fn set_severity_number(&mut self, number: Severity);

    /// Sets the message body of the log.
    fn set_body(&mut self, body: AnyValue<'a>);

    /// Adds multiple attributes.
    fn add_attributes<I, K, V>(&mut self, attributes: I)
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<Key>,
        V: Into<AnyValue<'a>>;

    /// Adds a single attribute.
    fn add_attribute<K, V>(&mut self, key: K, value: V)
    where
        K: Into<Key>,
        V: Into<AnyValue<'a>>;
}

/// Value types for representing arbitrary values in a log record.
#[derive(Debug, Clone, PartialEq)]
pub enum AnyValue<'a> {
    /// An integer value
    Int(i64),
    /// A double value
    Double(f64),
    /// A string value
    String(Cow<'a, str>),
    /// A boolean value
    Boolean(bool),
    /// A byte array
    Bytes(Box<Vec<u8>>),
    /// An array of `Any` values
    ListAny(Box<Vec<AnyValue<'a>>>),
    /// A map of string keys to `Any` values, arbitrarily nested.
    Map(Box<HashMap<Key, AnyValue<'a>>>),
}

macro_rules! impl_trivial_from {
    ($t:ty, $variant:path) => {
        impl<'a> From<$t> for AnyValue<'a> {
            fn from(val: $t) -> AnyValue<'a> {
                $variant(val.into())
            }
        }
    };
}

impl_trivial_from!(i8, AnyValue::Int);
impl_trivial_from!(i16, AnyValue::Int);
impl_trivial_from!(i32, AnyValue::Int);
impl_trivial_from!(i64, AnyValue::Int);

impl_trivial_from!(u8, AnyValue::Int);
impl_trivial_from!(u16, AnyValue::Int);
impl_trivial_from!(u32, AnyValue::Int);

impl_trivial_from!(f64, AnyValue::Double);
impl_trivial_from!(f32, AnyValue::Double);

impl_trivial_from!(bool, AnyValue::Boolean);

impl<'a> From<Cow<'a, str>> for AnyValue<'a> {
    fn from(val: Cow<'a, str>) -> AnyValue<'a> {
        AnyValue::String(val)
    }
}

impl<'a> From<&'a str> for AnyValue<'a> {
    fn from(val: &'a str) -> AnyValue<'a> {
        AnyValue::String(Cow::Borrowed(val))
    }
}

impl From<String> for AnyValue<'static> {
    fn from(val: String) -> AnyValue<'static> {
        AnyValue::String(Cow::Owned(val))
    }
}

impl<'a, T: Into<AnyValue<'a>>> FromIterator<T> for AnyValue<'a> {
    /// Creates an [`AnyValue::ListAny`] value from a sequence of `Into<AnyValue>` values.
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        AnyValue::ListAny(Box::new(iter.into_iter().map(Into::into).collect()))
    }
}

impl<'a, K: Into<Key>, V: Into<AnyValue<'a>>> FromIterator<(K, V)> for AnyValue<'a> {
    /// Creates an [`AnyValue::Map`] value from a sequence of key-value pairs
    /// that can be converted into a `Key` and `AnyValue` respectively.
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        AnyValue::Map(Box::new(HashMap::from_iter(
            iter.into_iter().map(|(k, v)| (k.into(), v.into())),
        )))
    }
}

impl<'a> From<StringValue> for AnyValue<'a> {
    fn from(value: StringValue) -> Self {
        AnyValue::String(Cow::Owned(value.to_string()))
    }
}

impl<'a> From<Value> for AnyValue<'a> {
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(b) => b.into(),
            Value::I64(i) => i.into(),
            Value::F64(f) => f.into(),
            Value::String(s) => s.into(),
            Value::Array(a) => match a {
                Array::Bool(b) => AnyValue::from_iter(b),
                Array::F64(f) => AnyValue::from_iter(f),
                Array::I64(i) => AnyValue::from_iter(i),
                Array::String(s) => AnyValue::from_iter(s),
            },
        }
    }
}

impl<'a> AnyValue<'a> {
    /// Converts the `AnyValue` into an owned version in place, updating the existing instance.
    pub fn make_owned(&mut self) {
        match self {
            AnyValue::String(ref mut s) => {
                if let Cow::Borrowed(borrowed_str) = s {
                    // Replace the borrowed string with an owned string
                    *s = Cow::Owned(borrowed_str.to_string());
                }
            }
            AnyValue::ListAny(ref mut list) => {
                // Recursively convert each item in the list to owned
                for item in list.iter_mut() {
                    item.make_owned();
                }
            }
            AnyValue::Map(ref mut map) => {
                // Recursively convert each value in the map to owned
                for value in map.values_mut() {
                    value.make_owned();
                }
            }
            // Other variants are inherently owned and do not need to be modified
            _ => {}
        }
    }

    /// Converts the `AnyValue` into an owned version.
    pub fn into_owned(self) -> AnyValue<'static> {
        match self {
            AnyValue::Int(v) => AnyValue::Int(v),
            AnyValue::Double(v) => AnyValue::Double(v),
            AnyValue::String(s) => AnyValue::String(Cow::Owned(s.into_owned())),
            AnyValue::Boolean(v) => AnyValue::Boolean(v),
            AnyValue::Bytes(b) => AnyValue::Bytes(b), // Assuming this is already owned
            AnyValue::ListAny(v) => {
                AnyValue::ListAny(Box::new(v.into_iter().map(AnyValue::into_owned).collect()))
            }
            AnyValue::Map(m) => AnyValue::Map(Box::new(
                m.into_iter().map(|(k, v)| (k, v.into_owned())).collect(),
            )),
        }
    }
}

/// A normalized severity value.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub enum Severity {
    /// TRACE
    Trace = 1,
    /// TRACE2
    Trace2 = 2,
    /// TRACE3
    Trace3 = 3,
    /// TRACE4
    Trace4 = 4,
    /// DEBUG
    Debug = 5,
    /// DEBUG2
    Debug2 = 6,
    /// DEBUG3
    Debug3 = 7,
    /// DEBUG4
    Debug4 = 8,
    /// INFO
    Info = 9,
    /// INFO2
    Info2 = 10,
    /// INFO3
    Info3 = 11,
    /// INFO4
    Info4 = 12,
    /// WARN
    Warn = 13,
    /// WARN2
    Warn2 = 14,
    /// WARN3
    Warn3 = 15,
    /// WARN4
    Warn4 = 16,
    /// ERROR
    Error = 17,
    /// ERROR2
    Error2 = 18,
    /// ERROR3
    Error3 = 19,
    /// ERROR4
    Error4 = 20,
    /// FATAL
    Fatal = 21,
    /// FATAL2
    Fatal2 = 22,
    /// FATAL3
    Fatal3 = 23,
    /// FATAL4
    Fatal4 = 24,
}

impl Severity {
    /// Return the string representing the short name for the `Severity`
    /// value as specified by the OpenTelemetry logs data model.
    pub const fn name(&self) -> &'static str {
        match &self {
            Severity::Trace => "TRACE",
            Severity::Trace2 => "TRACE2",
            Severity::Trace3 => "TRACE3",
            Severity::Trace4 => "TRACE4",

            Severity::Debug => "DEBUG",
            Severity::Debug2 => "DEBUG2",
            Severity::Debug3 => "DEBUG3",
            Severity::Debug4 => "DEBUG4",

            Severity::Info => "INFO",
            Severity::Info2 => "INFO2",
            Severity::Info3 => "INFO3",
            Severity::Info4 => "INFO4",

            Severity::Warn => "WARN",
            Severity::Warn2 => "WARN2",
            Severity::Warn3 => "WARN3",
            Severity::Warn4 => "WARN4",

            Severity::Error => "ERROR",
            Severity::Error2 => "ERROR2",
            Severity::Error3 => "ERROR3",
            Severity::Error4 => "ERROR4",

            Severity::Fatal => "FATAL",
            Severity::Fatal2 => "FATAL2",
            Severity::Fatal3 => "FATAL3",
            Severity::Fatal4 => "FATAL4",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum StringType<'a> {
    Static(&'static str),
    Dynamic(Cow<'a, str>),
}

impl<'a> StringType<'a> {
    // Creates a StringType from a static string slice
    fn from_static(s: &'static str) -> Self {
        StringType::Static(s)
    }

    // Creates a StringType from a borrowed string slice using Cow
    fn from_borrowed(s: &'a str) -> Self {
        StringType::Dynamic(Cow::Borrowed(s))
    }

    // Creates a StringType from an owned String using Cow
    fn from_owned(s: String) -> Self {
        StringType::Dynamic(Cow::Owned(s))
    }

    // Converts the StringType to a fully owned String
    fn into_owned(self) -> String {
        match self {
            StringType::Static(s) => s.to_string(),
            StringType::Dynamic(cow) => cow.into_owned(),
        }
    }

    // Converts the borrowed Cow variant to an owned String if it's not already owned
    fn to_owned(&mut self) {
        if let StringType::Dynamic(cow) = self {
            *cow = Cow::Owned(cow.to_string());
        }
    }
}
