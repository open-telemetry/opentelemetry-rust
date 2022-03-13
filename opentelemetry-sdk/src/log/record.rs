use std::{borrow::Cow, collections::BTreeMap, time::SystemTime};

use opentelemetry_api::trace::{SpanId, TraceFlags, TraceId};

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
    Trace = 1,
    Trace2 = 2,
    Trace3 = 3,
    Trace4 = 4,
    Debug = 5,
    Debug2 = 6,
    Debug3 = 7,
    Debug4 = 8,
    Info = 9,
    Info2 = 10,
    Info3 = 11,
    Info4 = 12,
    Warn = 13,
    Warn2 = 14,
    Warn3 = 15,
    Warn4 = 16,
    Error = 17,
    Error2 = 18,
    Error3 = 19,
    Error4 = 20,
    Fatal = 21,
    Fatal2 = 22,
    Fatal3 = 23,
    Fatal4 = 24,
}
