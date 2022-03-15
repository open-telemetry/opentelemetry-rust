use opentelemetry_api::trace::{SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId};
use std::{borrow::Cow, collections::BTreeMap, time::SystemTime};

#[derive(Debug, Clone, Default)]
#[non_exhaustive]
/// LogRecord represents all data carrier by a log record, and
/// is provided to `LogExporter`s as input.
pub struct LogRecord {
    /// Record timestamp
    pub timestamp: Option<SystemTime>,

    /// Trace context for logs associated with spans
    pub trace_context: Option<TraceContext>,

    /// The original severity string from the source
    pub severity_text: Option<Cow<'static, str>>,
    /// The corresponding severity value, normalized
    pub severity_number: Option<Severity>,

    /// Record name
    pub name: Option<Cow<'static, str>>,
    /// Record body
    pub body: Option<Any>,

    /// Resource attributes for the entity that produced this record
    pub resource: Option<BTreeMap<Cow<'static, str>, Any>>,
    /// Additional attributes associated with this record
    pub attributes: Option<BTreeMap<Cow<'static, str>, Any>>,
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

/// Value types for representing arbitrary values in a log record.
#[derive(Debug, Clone)]
pub enum Any {
    /// An integer value
    Int(i64),
    /// A double value
    Double(f64),
    /// A string value
    String(String),
    /// A boolean value
    Boolean(bool),
    /// A byte array
    Bytes(Vec<u8>),
    /// An array of `Any` values
    ListAny(Vec<Any>),
    /// A map of string keys to `Any` values, arbitrarily nested.
    Map(BTreeMap<Cow<'static, str>, Any>),
}

macro_rules! impl_trivial_from {
    ($t:ty, $variant:path) => {
        impl From<$t> for Any {
            fn from(val: $t) -> Any {
                $variant(val.into())
            }
        }
    };
}

impl_trivial_from!(i8, Any::Int);
impl_trivial_from!(i16, Any::Int);
impl_trivial_from!(i32, Any::Int);
impl_trivial_from!(i64, Any::Int);

impl_trivial_from!(u8, Any::Int);
impl_trivial_from!(u16, Any::Int);
impl_trivial_from!(u32, Any::Int);

impl_trivial_from!(f64, Any::Double);
impl_trivial_from!(f32, Any::Double);

impl_trivial_from!(String, Any::String);
impl_trivial_from!(Cow<'static, str>, Any::String);
impl_trivial_from!(&str, Any::String);

impl_trivial_from!(bool, Any::Boolean);

impl<T: Into<Any>> From<Vec<T>> for Any {
    /// Converts a list of `Into<Any>` values into a [`Any::ListAny`]
    /// value.
    fn from(val: Vec<T>) -> Any {
        Any::ListAny(val.into_iter().map(Into::into).collect())
    }
}

/// A normalized severity value.
#[derive(Debug, Copy, Clone)]
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

    /// Assign name
    pub fn with_name<T>(self, name: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        Self {
            record: LogRecord {
                name: Some(name.into()),
                ..self.record
            },
        }
    }

    /// Assign body
    pub fn with_body(self, body: Any) -> Self {
        Self {
            record: LogRecord {
                body: Some(body),
                ..self.record
            },
        }
    }

    /// Assign resource
    pub fn with_resource(self, resource: BTreeMap<Cow<'static, str>, Any>) -> Self {
        Self {
            record: LogRecord {
                resource: Some(resource),
                ..self.record
            },
        }
    }

    /// Assign attributes, overriding previously set attributes
    pub fn with_attributes(self, attributes: BTreeMap<Cow<'static, str>, Any>) -> Self {
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
        K: Into<Cow<'static, str>>,
        V: Into<Any>,
    {
        if let Some(ref mut map) = self.record.attributes {
            map.insert(key.into(), value.into());
        } else {
            self.record.attributes = Some(BTreeMap::from([(key.into(), value.into())]));
        }

        self
    }

    /// Build the record, consuming the Builder
    pub fn build(self) -> LogRecord {
        self.record
    }
}
