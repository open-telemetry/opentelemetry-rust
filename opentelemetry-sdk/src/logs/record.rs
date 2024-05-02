use opentelemetry::{
    trace::{SpanContext, SpanId, TraceFlags, TraceId},
    Key,
    logs::{LogRecord, AnyValue, Severity},
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
        fn set_timestamp(&mut self, timestamp: SystemTime) -> &mut Self{
            self.timestamp = Some(timestamp);
            self
        }
    
        fn set_observed_timestamp(&mut self, timestamp: SystemTime) -> &mut Self{
            self.observed_timestamp = Some(timestamp);
            self
        }
    
        fn set_span_context(&mut self, span_context: &SpanContext) -> &mut Self{
            self.trace_context =  Some(TraceContext {
                span_id: span_context.span_id(),
                trace_id: span_context.trace_id(),
                trace_flags: Some(span_context.trace_flags()),
            });
            self
        }
    
        fn set_severity_text(&mut self, severity_text: Option<Cow<'static, str>>) -> &mut Self{
            self.severity_text = severity_text.into();
            self
        }
    
        fn set_severity_number(&mut self, severity_number: Severity) -> &mut Self{
            self.severity_number = Some(severity_number);
            self
        }
    
        fn set_body(&mut self, body: Option<AnyValue>) -> &mut Self{
            self.body = body;
            self
        }
    
        fn set_attributes(&mut self, attributes: Vec<(Key, AnyValue)>) -> &mut Self{
            self.attributes = Some(attributes);
            self
        }
    
        fn set_attribute<K,V>(&mut self, key: K, value: V) -> &mut Self
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
            self
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
