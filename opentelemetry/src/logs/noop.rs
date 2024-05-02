use std::{borrow::Cow, sync::Arc, time::SystemTime};

use crate::{
    logs::{AnyValue, LogRecord, LogRecordBuilder, Logger, LoggerProvider, Severity},
    trace::SpanContext,
    InstrumentationLibrary, Key, KeyValue,
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
    fn set_timestamp(&mut self, _timestamp: SystemTime) {}
    fn set_observed_timestamp(&mut self, _timestamp: SystemTime) {}
    fn set_span_context(&mut self, _context: &SpanContext) {}
    fn set_severity_text(&mut self, _text: Cow<'static, str>) {}
    fn set_severity_number(&mut self, _number: Severity) {}
    fn set_body(&mut self, _body: AnyValue) {}
    fn set_attributes(&mut self, _attributes: Vec<(Key, AnyValue)>) {}
    fn set_attribute<K, V>(&mut self, _key: K, _value: V)
    where
        K: Into<Key>,
        V: Into<AnyValue>,
    {
    }
    fn set_context<T>(&mut self, _context: &T)
    where
        T: crate::trace::TraceContextExt,
    {
    }
}

#[derive(Debug, Clone, Default)]
pub struct NoopLogRecordBuilder;

impl LogRecordBuilder for NoopLogRecordBuilder {
    type LogRecord = NoopLogRecord;

    fn with_timestamp(self, _timestamp: SystemTime) -> Self {
        self
    }

    fn with_observed_timestamp(self, _timestamp: SystemTime) -> Self {
        self
    }

    fn with_span_context(self, _context: &SpanContext) -> Self {
        self
    }

    fn with_severity_text(self, _text: Cow<'static, str>) -> Self {
        self
    }

    fn with_severity_number(self, _number: Severity) -> Self {
        self
    }

    fn with_body(self, _body: AnyValue) -> Self {
        self
    }

    fn with_attributes(self, _attributes: Vec<(Key, AnyValue)>) -> Self {
        self
    }

    fn with_attribute<K, V>(self, _key: K, _value: V) -> Self
    where
        K: Into<Key>,
        V: Into<AnyValue>,
    {
        self
    }

    fn with_context<T>(self, _context: &T) -> Self
    where
        T: crate::trace::TraceContextExt,
    {
        self
    }

    fn build(&self) -> Self::LogRecord {
        NoopLogRecord
    }
}

/// A no-op implementation of a [`Logger`]
#[derive(Clone, Debug)]
pub struct NoopLogger(());

impl Logger for NoopLogger {
    type LogRecord = NoopLogRecord;
    type LogRecordBuilder = NoopLogRecordBuilder;

    fn create_log_record(&self) -> Self::LogRecordBuilder {
        NoopLogRecordBuilder {}
    }
    fn emit(&self, _record: Self::LogRecord) {}
    #[cfg(feature = "logs_level_enabled")]
    fn event_enabled(&self, _level: super::Severity, _target: &str) -> bool {
        false
    }
}
