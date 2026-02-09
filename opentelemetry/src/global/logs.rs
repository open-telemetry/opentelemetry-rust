use std::any::Any;
use std::borrow::Cow;
use std::fmt::Debug;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::SystemTime;

use crate::logs::{AnyValue, LogRecord, Logger, LoggerProvider, NoopLoggerProvider, Severity};
use crate::{otel_error, otel_info, InstrumentationScope, Key, SpanId, TraceFlags, TraceId};

/// Allows a specific [`crate::logs::LoggerProvider`] to be used generically by [`BoxedLoggerProvider`]
/// instances by mirroring the interface and boxing the return types.
pub trait DynCompatibleLoggerProvider {
    /// Returns a new logger with the given instrumentation scope.
    fn logger_with_scope(&self, scope: InstrumentationScope) -> BoxedLogger;

    /// Returns a new logger with the given name.
    ///
    /// The `name` should be the application name or the name of the library
    /// providing instrumentation.
    fn logger(&self, name: Cow<'static, str>) -> BoxedLogger {
        let scope = InstrumentationScope::builder(name).build();
        self.logger_with_scope(scope)
    }
}

impl<L: 'static + LoggerProvider> DynCompatibleLoggerProvider for L {
    fn logger_with_scope(&self, scope: InstrumentationScope) -> BoxedLogger {
        BoxedLogger(Box::new(self.logger_with_scope(scope)))
    }
}

/// Allows a specific [`crate::logs::Logger`] to be used generically by [`BoxedLogger`]
/// instances by mirroring the interface and boxing the return types.
pub trait DynCompatibleLogger {
    /// Creates a new log record builder.
    fn create_log_record(&self) -> BoxedLogRecord;

    /// Emit a [`LogRecord`]. If there is active current thread's [`Context`],
    /// the logger will set the record's `TraceContext` to the active trace context.
    ///
    /// If the underlying type inside the [`BoxedLogRecord`] does not [`downcast`] into the `LogRecord` type of the boxed logger implementation, this function will panic.
    ///
    /// [`Context`]: crate::Context
    /// [`downcast`]: Box::downcast
    fn emit(&self, record: BoxedLogRecord);

    /// Check if the given log is enabled.
    fn event_enabled(&self, level: Severity, target: &str, name: Option<&str>) -> bool;
}

impl<L: 'static + Logger> DynCompatibleLogger for L {
    fn create_log_record(&self) -> BoxedLogRecord {
        BoxedLogRecord(Box::new(self.create_log_record()))
    }

    fn emit(&self, record: BoxedLogRecord) {
        match record.0.into_any().downcast::<L::LogRecord>() {
            Ok(record) => self.emit(*record),
            Err(_) => panic!("Incompatible LogRecord type passed to logger"),
        }
    }

    fn event_enabled(&self, level: Severity, target: &str, name: Option<&str>) -> bool {
        self.event_enabled(level, target, name)
    }
}

/// Allows a specific [`crate::logs::LogRecord`] to be used generically by [`BoxedLogRecord`]
/// instances by mirroring the interface, boxing the return types, and providing downcasting.
pub trait DynCompatibleLogRecord {
    /// Utility method to allow downcasting.
    fn into_any(self: Box<Self>) -> Box<dyn Any>;

    /// Sets the `event_name` of a record
    fn set_event_name(&mut self, name: &'static str);

    /// Sets the `target` of a record.
    ///
    /// Exporters MAY use this field to override the `instrumentation_scope.name`.
    fn set_target(&mut self, _target: Cow<'static, str>);

    /// Sets the time when the event occurred measured by the origin clock, i.e. the time at the source.
    fn set_timestamp(&mut self, timestamp: SystemTime);

    /// Sets the observed event timestamp.
    fn set_observed_timestamp(&mut self, timestamp: SystemTime);

    /// Sets severity as text.
    fn set_severity_text(&mut self, text: &'static str);

    /// Sets severity as a numeric value.
    fn set_severity_number(&mut self, number: Severity);

    /// Sets the message body of the log.
    fn set_body(&mut self, body: AnyValue);

    /// Adds multiple attributes.
    fn add_attributes(&mut self, attributes: Vec<(Key, AnyValue)>);

    /// Adds a single attribute.
    fn add_attribute(&mut self, key: Key, value: AnyValue);

    /// Sets the trace context of the log.
    fn set_trace_context(
        &mut self,
        trace_id: TraceId,
        span_id: SpanId,
        trace_flags: Option<TraceFlags>,
    );
}

impl<L: 'static + LogRecord + Any> DynCompatibleLogRecord for L {
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn set_event_name(&mut self, name: &'static str) {
        self.set_event_name(name);
    }

    fn set_target(&mut self, target: Cow<'static, str>) {
        self.set_target(target);
    }

    fn set_timestamp(&mut self, timestamp: SystemTime) {
        self.set_timestamp(timestamp);
    }

    fn set_observed_timestamp(&mut self, timestamp: SystemTime) {
        self.set_observed_timestamp(timestamp);
    }

    fn set_severity_text(&mut self, text: &'static str) {
        self.set_severity_text(text);
    }

    fn set_severity_number(&mut self, number: Severity) {
        self.set_severity_number(number);
    }

    fn set_body(&mut self, body: AnyValue) {
        self.set_body(body);
    }

    fn add_attributes(&mut self, attributes: Vec<(Key, AnyValue)>) {
        self.add_attributes(attributes);
    }

    fn add_attribute(&mut self, key: Key, value: AnyValue) {
        self.add_attribute(key, value);
    }

    fn set_trace_context(
        &mut self,
        trace_id: TraceId,
        span_id: SpanId,
        trace_flags: Option<TraceFlags>,
    ) {
        self.set_trace_context(trace_id, span_id, trace_flags);
    }
}

/// Wraps the [`GlobalLoggerProvider`]'s [`Logger`] so it can be used generically by
/// applications without knowing the underlying type.
pub struct BoxedLogger(Box<dyn DynCompatibleLogger>);

/// Wraps the [`BoxedLogger`]'s [`LogRecord`] so it can be used generically by
/// applications without knowing the underlying type.
pub struct BoxedLogRecord(Box<dyn DynCompatibleLogRecord>);

impl Debug for BoxedLogger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BoxedLogger").finish()
    }
}

impl Debug for BoxedLogRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BoxedLogRecord").finish()
    }
}

impl Logger for BoxedLogger {
    type LogRecord = BoxedLogRecord;

    fn create_log_record(&self) -> Self::LogRecord {
        self.0.create_log_record()
    }

    fn emit(&self, record: Self::LogRecord) {
        self.0.emit(record);
    }

    fn event_enabled(&self, level: Severity, target: &str, name: Option<&str>) -> bool {
        self.0.event_enabled(level, target, name)
    }
}

impl LogRecord for BoxedLogRecord {
    fn set_event_name(&mut self, name: &'static str) {
        self.0.set_event_name(name);
    }

    fn set_target<T>(&mut self, target: T)
    where
        T: Into<Cow<'static, str>>,
    {
        self.0.set_target(target.into());
    }

    fn set_timestamp(&mut self, timestamp: SystemTime) {
        self.0.set_timestamp(timestamp)
    }

    fn set_observed_timestamp(&mut self, timestamp: SystemTime) {
        self.0.set_observed_timestamp(timestamp)
    }

    fn set_severity_text(&mut self, text: &'static str) {
        self.0.set_severity_text(text)
    }

    fn set_severity_number(&mut self, number: Severity) {
        self.0.set_severity_number(number)
    }

    fn set_body(&mut self, body: AnyValue) {
        self.0.set_body(body)
    }

    fn add_attributes<I, K, V>(&mut self, attributes: I)
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<Key>,
        V: Into<AnyValue>,
    {
        self.0.add_attributes(
            attributes
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        );
    }

    fn add_attribute<K, V>(&mut self, key: K, value: V)
    where
        K: Into<Key>,
        V: Into<AnyValue>,
    {
        self.0.add_attribute(key.into(), value.into());
    }
}

/// TODO
#[derive(Clone)]
pub struct GlobalLoggerProvider {
    provider: Arc<dyn DynCompatibleLoggerProvider + Send + Sync>,
}

impl Debug for GlobalLoggerProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlobalLoggerProvider").finish()
    }
}

impl GlobalLoggerProvider {
    /// Create a new GlobalLoggerProvider instance from a [`LoggerProvider`].
    fn new<L>(provider: L) -> Self
    where
        L: 'static + LoggerProvider + Send + Sync,
    {
        GlobalLoggerProvider {
            provider: Arc::new(provider),
        }
    }
}

/// The global `Logger` provider singleton.
static GLOBAL_LOGGER_PROVIDER: OnceLock<RwLock<GlobalLoggerProvider>> = OnceLock::new();

#[inline]
fn global_logger_provider() -> &'static RwLock<GlobalLoggerProvider> {
    GLOBAL_LOGGER_PROVIDER
        .get_or_init(|| RwLock::new(GlobalLoggerProvider::new(NoopLoggerProvider::new())))
}

/// Returns an instance of the currently configured global [`LoggerProvider`] through
/// [`GlobalLoggerProvider`].
pub fn logger_provider() -> GlobalLoggerProvider {
    // Try to get the global logger provider. If the RwLock is poisoned, we'll log an error and return a NoopLoggerProvider.
    let global_provider = global_logger_provider().read();
    if let Ok(provider) = global_provider {
        provider.clone()
    } else {
        otel_error!(name: "LoggerProvider.GlobalGetFailed", message = "Getting global loger provider failed. Loggers created using global::logger() or global::logger_with_scope() will not function. Report this issue in OpenTelemetry repo.");
        GlobalLoggerProvider::new(NoopLoggerProvider::new())
    }
}

impl LoggerProvider for GlobalLoggerProvider {
    type Logger = BoxedLogger;

    fn logger_with_scope(&self, scope: InstrumentationScope) -> Self::Logger {
        self.provider.logger_with_scope(scope)
    }
}

/// Creates a named instance of a [`Logger`] via the configured [`GlobalTracerProvider`].
///
/// If the name is an empty string, the provider will use a default name.
///
/// This is a more convenient way of expressing `global::logger_provider().logger(name)`.
pub fn logger(name: impl Into<Cow<'static, str>>) -> BoxedLogger {
    LoggerProvider::logger(&logger_provider(), name)
}

/// Creates a [`Logger`] with the given instrumentation scope
/// via the configured [`GlobalTracerProvider`].
///
/// This is a simpler alternative to `global::logger_provider().logger_with_scope(...)`
pub fn logger_with_scope(scope: InstrumentationScope) -> BoxedLogger {
    LoggerProvider::logger_with_scope(&logger_provider(), scope)
}

/// Sets the given [`LoggerProvider`] instance as the current global provider.
///
/// Libraries SHOULD NOT call this function. It is intended for applications/executables.
pub fn set_logger_provider<L>(new_provider: L)
where
    L: LoggerProvider + Send + Sync + 'static,
{
    let mut global_provider = global_logger_provider().write();
    if let Ok(ref mut provider) = global_provider {
        **provider = GlobalLoggerProvider::new(new_provider);
        otel_info!(name: "LoggerProvider.GlobalSet", message = "Global logger provider is set. Logs can now be created using global::logger() or global::logger().");
    } else {
        otel_error!(name: "LoggerProvider.GlobalSetFailed", message = "Setting global logger provider failed. Traces created using global::logger() or global::logger_with_scope() will not function. Report this issue in OpenTelemetry repo.");
    }
}
