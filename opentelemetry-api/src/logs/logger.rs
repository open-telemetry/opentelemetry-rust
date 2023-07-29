use std::{borrow::Cow, sync::Arc};

use crate::{logs::LogRecord, InstrumentationLibrary, KeyValue};

#[cfg(feature = "logs_level_enabled")]
use super::Severity;

/// The interface for emitting [`LogRecord`]s.

pub trait Logger {
    /// Emit a [`LogRecord`]. If there is active current thread's [`Context`],
    ///  the logger will set the record's [`TraceContext`] to the active trace context,
    ///
    /// [`Context`]: crate::Context
    /// [`TraceContext`]: crate::logs::TraceContext
    fn emit(&self, record: LogRecord);

    #[cfg(feature = "logs_level_enabled")]
    /// Check if the given log level is enabled.
    fn event_enabled(&self, level: Severity, target: &str) -> bool;
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
    ) -> Self::Logger {
        self.library_logger(Arc::new(InstrumentationLibrary::new(
            name, version, schema_url, attributes,
        )))
    }

    /// Returns a new versioned logger with the given instrumentation library.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry_api::{global, InstrumentationLibrary, logs::LoggerProvider};
    ///
    /// let provider = global::logger_provider();
    ///
    /// // logger used in applications/binaries
    /// let logger = provider.logger("my_app");
    /// // logger used in libraries/crates that optionally includes version and schema url
    /// let library = std::sync::Arc::new(InstrumentationLibrary::new(
    ///     env!("CARGO_PKG_NAME"),
    ///     Some(env!("CARGO_PKG_VERSION")),
    ///     Some("https://opentelemetry.io/schema/1.0.0"),
    ///     None,
    /// ));
    /// let logger = provider.library_logger(library);
    /// ```
    fn library_logger(&self, library: Arc<InstrumentationLibrary>) -> Self::Logger;

    /// Returns a new logger with the given name.
    ///
    /// The `name` should be the application name or the name of the library
    /// providing instrumentation. If the name is empty, then an
    /// implementation-defined default name may be used instead.
    fn logger(&self, name: impl Into<Cow<'static, str>>) -> Self::Logger {
        self.versioned_logger(name, None, None, None)
    }
}
