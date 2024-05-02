use opentelemetry::{
    logs::{AnyValue, LogRecord, LogRecordBuilder, Severity},
    trace::{SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId},
    Key,
};
use std::{borrow::Cow, time::SystemTime};

#[derive(Debug, Clone, Default)]
#[non_exhaustive]
/// LogRecord represents all data carried by a log record, and
/// is provided to `LogExporter`s as input.
pub struct SdkLogRecord {
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
    pub attributes: Option<Vec<(Key, AnyValue)>>,
}

impl LogRecord for SdkLogRecord {
    fn set_timestamp(&mut self, timestamp: SystemTime) {
        self.timestamp = Some(timestamp);
    }

    fn set_observed_timestamp(&mut self, timestamp: SystemTime) {
        self.observed_timestamp = Some(timestamp);
    }

    fn set_span_context(&mut self, span_context: &SpanContext) {
        self.trace_context = Some(TraceContext {
            span_id: span_context.span_id(),
            trace_id: span_context.trace_id(),
            trace_flags: Some(span_context.trace_flags()),
        });
    }

    fn set_severity_text(&mut self, severity_text: Cow<'static, str>) {
        self.severity_text = Some(severity_text);
    }

    fn set_severity_number(&mut self, severity_number: Severity) {
        self.severity_number = Some(severity_number);
    }

    fn set_body(&mut self, body: AnyValue) {
        self.body = Some(body);
    }

    fn set_attributes(&mut self, attributes: Vec<(Key, AnyValue)>) {
        self.attributes = Some(attributes);
    }

    fn set_attribute<K, V>(&mut self, key: K, value: V)
    where
        K: Into<Key>,
        V: Into<AnyValue>,
    {
        if self.attributes.is_none() {
            self.attributes = Some(Vec::new());
        }
        if let Some(ref mut attributes) = self.attributes {
            attributes.push((key.into(), value.into()));
        }
    }
}

/// Implementation for LogRecordBuilder
#[derive(Debug, Default)]
pub struct SdkLogRecordBuilder {
    record: SdkLogRecord,
}
impl LogRecordBuilder for SdkLogRecordBuilder {
    type LogRecord = SdkLogRecord;

    fn with_timestamp(mut self, timestamp: SystemTime) -> Self {
        self.record.set_timestamp(timestamp);
        self
    }

    fn with_observed_timestamp(mut self, timestamp: SystemTime) -> Self {
        self.record.observed_timestamp = Some(timestamp);
        self
    }

    fn with_span_context(mut self, context: &SpanContext) -> Self {
        self.record.trace_context = Some(TraceContext {
            span_id: context.span_id(),
            trace_id: context.trace_id(),
            trace_flags: Some(context.trace_flags()),
        });
        self
    }

    fn with_severity_text(mut self, text: Cow<'static, str>) -> Self {
        self.record.severity_text = Some(text);
        self
    }

    fn with_severity_number(mut self, number: Severity) -> Self {
        self.record.severity_number = Some(number);
        self
    }

    fn with_body(mut self, body: AnyValue) -> Self {
        self.record.body = Some(body);
        self
    }

    fn with_attributes(mut self, attributes: Vec<(Key, AnyValue)>) -> Self {
        self.record.attributes = Some(attributes);
        self
    }

    fn with_attribute<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<Key>,
        V: Into<AnyValue>,
    {
        if self.record.attributes.is_none() {
            self.record.attributes = Some(Vec::new());
        }
        if let Some(ref mut attributes) = self.record.attributes {
            attributes.push((key.into(), value.into()));
        }
        self
    }

    fn with_context<T>(mut self, context: &T) -> Self
    where
        T: TraceContextExt,
    {
        self.record.set_context(context);
        self
    }

    fn build(&self) -> Self::LogRecord {
        self.record.clone()
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
