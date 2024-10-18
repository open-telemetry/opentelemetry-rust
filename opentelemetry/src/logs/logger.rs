use std::{borrow::Cow, sync::Arc};

use crate::{logs::LogRecord, InstrumentationLibrary};

#[cfg(feature = "logs_level_enabled")]
use super::Severity;

/// The interface for emitting [`LogRecord`]s.

pub trait Logger {
    /// Specifies the `LogRecord` type associated with this logger.
    type LogRecord: LogRecord;

    /// Creates a new log record builder.
    fn create_log_record(&self) -> Self::LogRecord;

    /// Emit a [`LogRecord`]. If there is active current thread's [`Context`],
    ///  the logger will set the record's `TraceContext` to the active trace context,
    ///
    /// [`Context`]: crate::Context
    fn emit(&self, record: Self::LogRecord);

    #[cfg(feature = "logs_level_enabled")]
    /// Check if the given log level is enabled.
    fn event_enabled(&self, level: Severity, target: &str) -> bool;
}

/// Interfaces that can create [`Logger`] instances.
pub trait LoggerProvider {
    /// The [`Logger`] type that this provider will return.
    type Logger: Logger;

    /// Returns a new logger with the given instrumentation library.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use opentelemetry::InstrumentationLibrary;
    /// use opentelemetry::logs::LoggerProvider;
    /// use opentelemetry_sdk::logs::LoggerProvider as SdkLoggerProvider;
    ///
    /// let provider = SdkLoggerProvider::builder().build();
    ///
    /// // logger used in applications/binaries
    /// let logger = provider.logger("my_app");
    ///
    /// // logger used in libraries/crates that optionally includes version and schema url
    /// let library = Arc::new(
    ///     InstrumentationLibrary::builder(env!("CARGO_PKG_NAME"))
    ///         .with_version(env!("CARGO_PKG_VERSION"))
    ///         .with_schema_url("https://opentelemetry.io/schema/1.0.0")
    ///         .build(),
    /// );
    /// let logger = provider.library_logger(library);
    /// ```
    fn library_logger(&self, library: Arc<InstrumentationLibrary>) -> Self::Logger;

    /// Returns a new logger with the given name.
    ///
    /// The `name` should be the application name or the name of the library
    /// providing instrumentation. If the name is empty, then an
    /// implementation-defined default name may be used instead.
    fn logger(&self, name: impl Into<Cow<'static, str>>) -> Self::Logger {
        let library = Arc::new(InstrumentationLibrary::builder(name).build());
        self.library_logger(library)
    }
}
