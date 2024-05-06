use opentelemetry::{
    logs::{AnyValue, Severity},
    trace::{SpanContext, SpanId, TraceFlags, TraceId},
    Key,
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
    pub attributes: Option<Vec<(Key, AnyValue)>>,
}

impl opentelemetry::logs::LogRecord for LogRecord {
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
        if let Some(ref mut attrs) = self.attributes {
            attrs.push((key.into(), value.into()));
        } else {
            self.attributes = Some(vec![(key.into(), value.into())]);
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::logs::{AnyValue, LogRecord as _, Severity};
    use opentelemetry::trace::{SpanContext, SpanId, TraceFlags, TraceId};
    use std::borrow::Cow;
    use std::time::SystemTime;

    // Helper function to create a TraceId from a u128 number
    fn trace_id_from_u128(num: u128) -> TraceId {
        TraceId::from_bytes(num.to_be_bytes())
    }

    // Helper function to create a SpanId from a u64 number
    fn span_id_from_u64(num: u64) -> SpanId {
        SpanId::from_bytes(num.to_be_bytes())
    }

    #[test]
    fn test_set_timestamp() {
        let mut log_record = LogRecord::default();
        let now = SystemTime::now();
        log_record.set_timestamp(now);
        assert_eq!(log_record.timestamp, Some(now));
    }

    #[test]
    fn test_set_observed_timestamp() {
        let mut log_record = LogRecord::default();
        let now = SystemTime::now();
        log_record.set_observed_timestamp(now);
        assert_eq!(log_record.observed_timestamp, Some(now));
    }

    #[test]
    fn test_set_span_context() {
        let mut log_record = LogRecord::default();
        let span_context = SpanContext::new(
            trace_id_from_u128(123),
            span_id_from_u64(456),
            TraceFlags::default(),
            true,
            Default::default(),
        );
        log_record.set_span_context(&span_context);
        assert_eq!(
            log_record.trace_context.clone().unwrap().trace_id,
            span_context.trace_id()
        );
        assert_eq!(
            log_record.trace_context.clone().unwrap().span_id,
            span_context.span_id()
        );
        assert_eq!(
            log_record.trace_context.unwrap().trace_flags,
            Some(span_context.trace_flags())
        );
    }

    #[test]
    fn test_set_severity_text() {
        let mut log_record = LogRecord::default();
        let severity_text: Cow<'static, str> = "ERROR".into(); // Explicitly typed
        log_record.set_severity_text(severity_text);
        assert_eq!(log_record.severity_text, Some(Cow::Borrowed("ERROR")));
    }

    #[test]
    fn test_set_severity_number() {
        let mut log_record = LogRecord::default();
        let severity_number = Severity::Error;
        log_record.set_severity_number(severity_number);
        assert_eq!(log_record.severity_number, Some(Severity::Error));
    }

    #[test]
    fn test_set_body() {
        let mut log_record = LogRecord::default();
        let body = AnyValue::String("Test body".into());
        log_record.set_body(body.clone());
        assert_eq!(log_record.body, Some(body));
    }

    #[test]
    fn test_set_attributes() {
        let mut log_record = LogRecord::default();
        let attributes = vec![(Key::new("key"), AnyValue::String("value".into()))];
        log_record.set_attributes(attributes.clone());
        assert_eq!(log_record.attributes, Some(attributes));
    }

    #[test]
    fn test_set_attribute() {
        let mut log_record = LogRecord::default();
        log_record.set_attribute("key", "value");
        assert_eq!(
            log_record.attributes,
            Some(vec![(Key::new("key"), AnyValue::String("value".into()))])
        );
    }
}
