use std::{borrow::Cow, sync::Arc, time::SystemTime};

use crate::{
    logs::{LogRecord, Logger, LoggerProvider, AnyValue, Severity},
    InstrumentationLibrary, KeyValue, Key,
    trace::SpanContext
};

/// A no-op implementation of a [`LoggerProvider`].
#[derive(Clone, Debug, Default)]
pub struct NoopLoggerProvider(());

impl NoopLoggerProvider {
    /// Create a new no-op logger provider.
    pub fn new() -> Self {
        NoopLoggerProvider(())
    }
}

impl LoggerProvider for NoopLoggerProvider {
    type Logger = NoopLogger;

    fn library_logger(&self, _library: Arc<InstrumentationLibrary>) -> Self::Logger {
        NoopLogger(())
    }

    fn versioned_logger(
        &self,
        _name: impl Into<Cow<'static, str>>,
        _version: Option<Cow<'static, str>>,
        _schema_url: Option<Cow<'static, str>>,
        _attributes: Option<Vec<KeyValue>>,
    ) -> Self::Logger {
        NoopLogger(())
    }
}

#[derive(Debug, Clone, Default)]
/// A no-operation log record that implements the LogRecord trait.
pub struct NoopLogRecord;

impl LogRecord for NoopLogRecord {
    // Implement the LogRecord trait methods with empty bodies.
    fn set_timestamp(&mut self, _timestamp: SystemTime) -> &mut Self {self}
    fn set_observed_timestamp(&mut self, _timestamp: SystemTime) -> &mut Self {self}
    fn set_span_context(&mut self, _context: &SpanContext) ->  &mut Self {self}
    fn set_severity_text(&mut self, _text: Option<Cow<'static, str>>) -> &mut Self {self}
    fn set_severity_number(&mut self, _number: Severity) -> &mut Self {self}
    fn set_body(&mut self, _body: Option<AnyValue>) -> &mut Self {self}
    fn set_attributes(&mut self, _attributes: Vec<(Key, AnyValue)>) -> &mut Self {self}
    fn set_attribute<K,V>(&mut self, _key: K, _value: V) -> &mut Self
        where
            K: Into<Key>,
            V: Into<AnyValue> {self}
    fn set_context<T>(&mut self, _context: &T) -> &mut Self where T: crate::trace::TraceContextExt  {self}
}

/// A no-op implementation of a [`Logger`]
#[derive(Clone, Debug)]
pub struct NoopLogger(());

impl Logger for NoopLogger {
    type LogRecord = NoopLogRecord;
    fn create_log_record(&self) -> Self::LogRecord {
        NoopLogRecord
    }
    fn emit(&self, _record: Self::LogRecord) {}
    #[cfg(feature = "logs_level_enabled")]
    fn event_enabled(&self, _level: super::Severity, _target: &str) -> bool {
        false
    }
}
