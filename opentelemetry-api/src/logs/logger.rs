use std::borrow::Cow;

use crate::{logs::LogRecord, KeyValue};

/// The interface for emitting [`LogRecord`]s.
pub trait Logger {
    /// Emit a [`LogRecord`]. If there is currently active trace context,
    ///  the logger will set the record's
    /// [`TraceContext`] to the active trace context, using the current thread's
    /// [`Context`].
    ///
    /// [`Context`]: crate::Context
    /// [`TraceContext`]: crate::logs::TraceContext
    fn emit(&self, record: LogRecord);
}

/// Interfaces that can create [`Logger`] instances.
pub trait LoggerProvider {
    /// The [`Logger`] type that this provider will return.
    type Logger: Logger;

    /// Returns a new versioned logger with a given name.
    ///
    /// The `name` should be the application name or the name of the library
    /// providing instrumentation. If the name is empty, then an
    /// implementation-defined default name may be used instead.
    fn versioned_logger(
        &self,
        name: impl Into<Cow<'static, str>>,
        version: Option<Cow<'static, str>>,
        schema_url: Option<Cow<'static, str>>,
        attributes: Option<Vec<KeyValue>>,
    ) -> Self::Logger;

    /// Returns a new logger with the given name.
    ///
    /// The `name` should be the application name or the name of the library
    /// providing instrumentation. If the name is empty, then an
    /// implementation-defined default name may be used instead.
    fn logger(&self, name: impl Into<Cow<'static, str>>) -> Self::Logger {
        self.versioned_logger(name, None, None, None)
    }
}
