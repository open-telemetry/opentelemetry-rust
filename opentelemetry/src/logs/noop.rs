use std::{borrow::Cow, sync::Arc};

use crate::{
    logs::{LogRecord, Logger, LoggerProvider},
    InstrumentationLibrary, KeyValue,
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

/// No-operation implementation of LogRecord.
pub struct NoopLogRecord;

impl LogRecord for NoopLogRecord {
    // Implement the LogRecord trait methods with empty bodies.
    fn set_timestamp(&mut self, _timestamp: SystemTime) {}
    fn set_observed_timestamp(&mut self, _timestamp: SystemTime) {}
    fn set_span_context(&mut self, _context: &SpanContext) {}
    fn set_severity_text(&mut self, _text: &str) {}
    fn set_severity_number(&mut self, _number: Severity) {}
    fn set_body(&mut self, _body: AnyValue) {}
    fn set_attributes(&mut self, _attributes: Vec<(Key, AnyValue)>) {}
}


/// A no-op implementation of a [`Logger`]
#[derive(Clone, Debug)]
pub struct NoopLogger(());

impl Logger for NoopLogger {
    fn emit(&self, _record: LogRecord) {}
    #[cfg(feature = "logs_level_enabled")]
    fn event_enabled(&self, _level: super::Severity, _target: &str) -> bool {
        false
    }
}
