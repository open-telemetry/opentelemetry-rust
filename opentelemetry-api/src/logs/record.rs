use crate::{
    trace::{OrderMap, SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId},
    Array, Key, StringValue, Value,
};
use std::{borrow::Cow, time::SystemTime};

#[derive(Debug, Clone, Default)]
#[non_exhaustive]
/// LogRecord represents all data carried by a log record, and
/// is provided to `LogExporter`s as input.
pub struct LogRecord {
    /// Record timestamp
    pub timestamp: Option<SystemTime>,

    /// Timestamp for when the record was observed by OpenTelemetry
    pub observed_timestamp: Option<SystemTime>,

    /// Trace context for logs associated with spans
    pub trace_context: Option<TraceContext>,

    /// The original severity string from the source
    pub severity_text: Option<Cow<'static, str>>,
    /// The corresponding severity value, normalized
    pub severity_number: Option<Severity>,

    /// Record body
    pub body: Option<AnyValue>,

    /// Additional attributes associated with this record
    pub attributes: Option<OrderMap<Key, AnyValue>>,
}

impl LogRecord {
    /// Create a [`LogRecordBuilder`] to create a new Log Record
    pub fn builder() -> LogRecordBuilder {
        LogRecordBuilder::new()
    }
}

/// TraceContext stores the trace data for logs that have an associated
/// span.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TraceContext {
    /// Trace id
    pub trace_id: TraceId,
    /// Span Id
    pub span_id: SpanId,
    /// Trace flags
    pub trace_flags: Option<TraceFlags>,
}

impl From<&SpanContext> for TraceContext {
    fn from(span_context: &SpanContext) -> Self {
        TraceContext {
            trace_id: span_context.trace_id(),
            span_id: span_context.span_id(),
            trace_flags: Some(span_context.trace_flags()),
        }
    }
}

/// Value types for representing arbitrary values in a log record.
#[derive(Debug, Clone)]
pub enum AnyValue {
    /// An integer value
    Int(i64),
    /// A double value
    Double(f64),
    /// A string value
    String(StringValue),
    /// A boolean value
    Boolean(bool),
    /// A byte array
    Bytes(Vec<u8>),
    /// An array of `Any` values
    ListAny(Vec<AnyValue>),
    /// A map of string keys to `Any` values, arbitrarily nested.
    Map(OrderMap<Key, AnyValue>),
}

macro_rules! impl_trivial_from {
    ($t:ty, $variant:path) => {
        impl From<$t> for AnyValue {
            fn from(val: $t) -> AnyValue {
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

impl_trivial_from!(String, AnyValue::String);
impl_trivial_from!(Cow<'static, str>, AnyValue::String);
impl_trivial_from!(&'static str, AnyValue::String);
impl_trivial_from!(StringValue, AnyValue::String);

impl_trivial_from!(bool, AnyValue::Boolean);

impl<T: Into<AnyValue>> FromIterator<T> for AnyValue {
    /// Creates an [`Any::ListAny`] value from a sequence of `Into<Any>` values.
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        AnyValue::ListAny(iter.into_iter().map(Into::into).collect())
    }
}

impl<K: Into<Key>, V: Into<AnyValue>> FromIterator<(K, V)> for AnyValue {
    /// Creates an [`Any::Map`] value from a sequence of key-value pairs
    /// that can be converted into a `Key` and `Any` respectively.
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        AnyValue::Map(OrderMap::from_iter(
            iter.into_iter().map(|(k, v)| (k.into(), v.into())),
        ))
    }
}

impl From<Value> for AnyValue {
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

/// A builder for [`LogRecord`] values.
#[derive(Debug, Clone)]
pub struct LogRecordBuilder {
    record: LogRecord,
}

impl LogRecordBuilder {
    /// Create a new LogRecordBuilder
    pub fn new() -> Self {
        Self {
            record: Default::default(),
        }
    }

    /// Assign timestamp
    pub fn with_timestamp(self, timestamp: SystemTime) -> Self {
        Self {
            record: LogRecord {
                timestamp: Some(timestamp),
                ..self.record
            },
        }
    }

    /// Assign observed timestamp
    pub fn with_observed_timestamp(self, timestamp: SystemTime) -> Self {
        Self {
            record: LogRecord {
                observed_timestamp: Some(timestamp),
                ..self.record
            },
        }
    }

    /// Assign the record's [`TraceContext`]
    pub fn with_span_context(self, span_context: &SpanContext) -> Self {
        Self {
            record: LogRecord {
                trace_context: Some(TraceContext {
                    span_id: span_context.span_id(),
                    trace_id: span_context.trace_id(),
                    trace_flags: Some(span_context.trace_flags()),
                }),
                ..self.record
            },
        }
    }

    /// Assign the record's [`TraceContext`] from a `TraceContextExt` trait
    pub fn with_context<T>(self, context: &T) -> Self
    where
        T: TraceContextExt,
    {
        if context.has_active_span() {
            self.with_span_context(context.span().span_context())
        } else {
            self
        }
    }

    /// Assign severity text
    pub fn with_severity_text<T>(self, severity: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        Self {
            record: LogRecord {
                severity_text: Some(severity.into()),
                ..self.record
            },
        }
    }

    /// Assign severity number
    pub fn with_severity_number(self, severity: Severity) -> Self {
        Self {
            record: LogRecord {
                severity_number: Some(severity),
                ..self.record
            },
        }
    }

    /// Assign body
    pub fn with_body(self, body: AnyValue) -> Self {
        Self {
            record: LogRecord {
                body: Some(body),
                ..self.record
            },
        }
    }

    /// Assign attributes, overriding previously set attributes
    pub fn with_attributes(self, attributes: OrderMap<Key, AnyValue>) -> Self {
        Self {
            record: LogRecord {
                attributes: Some(attributes),
                ..self.record
            },
        }
    }

    /// Set a single attribute for this record
    pub fn with_attribute<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<Key>,
        V: Into<AnyValue>,
    {
        if let Some(ref mut map) = self.record.attributes {
            map.insert(key.into(), value.into());
        } else {
            let mut map = OrderMap::with_capacity(1);
            map.insert(key.into(), value.into());
            self.record.attributes = Some(map);
        }

        self
    }

    /// Build the record, consuming the Builder
    pub fn build(self) -> LogRecord {
        self.record
    }
}

impl Default for LogRecordBuilder {
    fn default() -> Self {
        Self::new()
    }
}
