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

    fn library_logger(
        &self,
        _library: Arc<InstrumentationLibrary>,
        _include_trace_context: bool,
    ) -> Self::Logger {
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

/// A no-op implementation of a [`Logger`]
#[derive(Clone, Debug)]
pub struct NoopLogger(());

impl Logger for NoopLogger {
    fn emit(&self, _record: LogRecord) {}
}
