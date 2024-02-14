use std::{borrow::Cow, sync::Arc};

use crate::{logs::LogRecord, InstrumentationLibrary, InstrumentationLibraryBuilder, KeyValue};

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

    /// Deprecated, use [`LoggerProvider::logger_builder()`]
    ///
    /// Returns a new versioned logger with a given name.
    ///
    /// The `name` should be the application name or the name of the library
    /// providing instrumentation. If the name is empty, then an
    /// implementation-defined default name may be used instead.
    /// Create a new versioned `Logger` instance.
    #[deprecated(since = "0.23.0", note = "Please use logger_builder() instead")]
    fn versioned_logger(
        &self,
        name: impl Into<Cow<'static, str>>,
        version: Option<Cow<'static, str>>,
        schema_url: Option<Cow<'static, str>>,
        attributes: Option<Vec<KeyValue>>,
    ) -> Self::Logger {
        let mut builder = self.logger_builder(name);
        if let Some(v) = version {
            builder = builder.with_version(v);
        }
        if let Some(s) = schema_url {
            builder = builder.with_schema_url(s);
        }
        if let Some(a) = attributes {
            builder = builder.with_attributes(a);
        }
        builder.build()
    }

    /// Returns a new builder for creating a [`Logger`] instance
    ///
    /// The `name` should be the application name or the name of the library
    /// providing instrumentation. If the name is empty, then an
    /// implementation-defined default name may be used instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::{global, logs::LoggerProvider};
    ///
    /// let provider = global::logger_provider();
    ///
    /// // logger used in applications/binaries
    /// let logger = provider.logger_builder("my_app").build();
    ///
    /// // logger used in libraries/crates that optionally includes version and schema url
    /// let logger = provider.logger_builder("my_library")
    ///     .with_version(env!("CARGO_PKG_VERSION"))
    ///     .with_schema_url("https://opentelemetry.io/schema/1.0.0")
    ///     .build();
    /// ```
    fn logger_builder(&self, name: impl Into<Cow<'static, str>>) -> LoggerBuilder<'_, Self> {
        LoggerBuilder {
            provider: self,
            library_builder: InstrumentationLibrary::builder(name),
        }
    }

    /// Returns a new versioned logger with the given instrumentation library.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::{global, InstrumentationLibrary, logs::LoggerProvider};
    ///
    /// let provider = global::logger_provider();
    ///
    /// // logger used in applications/binaries
    /// let logger = provider.logger("my_app");
    ///
    /// // logger used in libraries/crates that optionally includes version and schema url
    /// let library = std::sync::Arc::new(
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
        self.logger_builder(name).build()
    }
}

#[derive(Debug)]
pub struct LoggerBuilder<'a, T: LoggerProvider + ?Sized> {
    provider: &'a T,
    library_builder: InstrumentationLibraryBuilder,
}

impl<'a, T: LoggerProvider + ?Sized> LoggerBuilder<'a, T> {
    pub fn with_version(mut self, version: impl Into<Cow<'static, str>>) -> Self {
        self.library_builder = self.library_builder.with_version(version);
        self
    }

    pub fn with_schema_url(mut self, schema_url: impl Into<Cow<'static, str>>) -> Self {
        self.library_builder = self.library_builder.with_schema_url(schema_url);
        self
    }

    pub fn with_attributes(mut self, attributes: impl Into<Vec<KeyValue>>) -> Self {
        self.library_builder = self.library_builder.with_attributes(attributes);
        self
    }

    pub fn build(self) -> T::Logger {
        self.provider
            .library_logger(Arc::new(self.library_builder.build()))
    }
}
