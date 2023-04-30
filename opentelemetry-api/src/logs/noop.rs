use crate::logs::{LogRecord, Logger, LoggerProvider};

/// A no-op implementation of a [`LoggerProvider`].
#[derive(Clone, Debug, Default)]
pub struct NoopLoggerProvider(());

impl NoopLoggerProvider {
    pub fn new() -> Self {
        NoopLoggerProvider(())
    }
}

impl LoggerProvider for NoopLoggerProvider {
    type Logger = NoopLogger;

    fn versioned_logger(
        &self,
        _name: std::borrow::Cow<'static, str>,
        _version: Option<std::borrow::Cow<'static, str>>,
        _schema_url: Option<std::borrow::Cow<'static, str>>,
        _attributes: Option<Vec<crate::KeyValue>>,
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
