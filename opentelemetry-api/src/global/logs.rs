use std::{
    borrow::Cow,
    fmt, mem,
    sync::{Arc, RwLock},
};

use once_cell::sync::Lazy;

use crate::{
    logs::{Logger, LoggerProvider, NoopLoggerProvider},
    KeyValue,
};

/// Allows a specific [`LoggerProvider`] to be used generically, by mirroring
/// the interface, and boxing the returned types.
///
/// [`LoggerProvider`]: crate::logs::LoggerProvider.
pub trait ObjectSafeLoggerProvider {
    /// Creates a versioned named [`Logger`] instance that is a trait object
    /// through the underlying [`LoggerProvider`].
    ///
    /// [`Logger`]: crate::logs::Logger
    /// [`LoggerProvider`]: crate::logs::LoggerProvider
    fn versioned_logger_boxed(
        &self,
        name: Cow<'static, str>,
        version: Option<Cow<'static, str>>,
        schema_url: Option<Cow<'static, str>>,
        attributes: Option<Vec<KeyValue>>,
        include_trace_context: bool,
    ) -> Box<dyn Logger + Send + Sync + 'static>;
}

impl<L, P> ObjectSafeLoggerProvider for P
where
    L: Logger + Send + Sync + 'static,
    P: LoggerProvider<Logger = L>,
{
    fn versioned_logger_boxed(
        &self,
        name: Cow<'static, str>,
        version: Option<Cow<'static, str>>,
        schema_url: Option<Cow<'static, str>>,
        attributes: Option<Vec<KeyValue>>,
        include_trace_context: bool,
    ) -> Box<dyn Logger + Send + Sync + 'static> {
        Box::new(self.versioned_logger(
            name,
            version,
            schema_url,
            attributes,
            include_trace_context,
        ))
    }
}

pub struct BoxedLogger(Box<dyn Logger + Send + Sync + 'static>);

impl fmt::Debug for BoxedLogger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BoxedLogger")
    }
}

impl Logger for BoxedLogger {
    fn emit(&self, record: crate::logs::LogRecord) {
        self.0.emit(record)
    }
}

#[derive(Clone)]
/// Represents the globally configured [`LoggerProvider`] instance.
pub struct GlobalLoggerProvider {
    provider: Arc<dyn ObjectSafeLoggerProvider + Send + Sync>,
}

impl fmt::Debug for GlobalLoggerProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GlobalLoggerProvider")
    }
}

impl GlobalLoggerProvider {
    fn new<
        L: Logger + Send + Sync + 'static,
        P: LoggerProvider<Logger = L> + Send + Sync + 'static,
    >(
        provider: P,
    ) -> Self {
        GlobalLoggerProvider {
            provider: Arc::new(provider),
        }
    }
}

impl LoggerProvider for GlobalLoggerProvider {
    type Logger = BoxedLogger;

    fn versioned_logger(
        &self,
        name: impl Into<Cow<'static, str>>,
        version: Option<Cow<'static, str>>,
        schema_url: Option<Cow<'static, str>>,
        attributes: Option<Vec<KeyValue>>,
        include_trace_context: bool,
    ) -> Self::Logger {
        BoxedLogger(self.provider.versioned_logger_boxed(
            name.into(),
            version,
            schema_url,
            attributes,
            include_trace_context,
        ))
    }
}

static GLOBAL_LOGGER_PROVIDER: Lazy<RwLock<GlobalLoggerProvider>> =
    Lazy::new(|| RwLock::new(GlobalLoggerProvider::new(NoopLoggerProvider::new())));

/// Returns an instance of the currently configured global [`LoggerProvider`]
/// through [`GlobalLoggerProvider`].
///
/// [`LoggerProvider`]: crate::logs::LoggerProvider
pub fn logger_provider() -> GlobalLoggerProvider {
    GLOBAL_LOGGER_PROVIDER
        .read()
        .expect("GLOBAL_LOGGER_PROVIDER RwLock poisoned")
        .clone()
}

/// Creates a named instance of [`Logger`] via the configured
/// [`GlobalLoggerProvider`].
///
/// If `name` is an empty string, the provider will use a default name.
///
/// [`Logger`]: crate::logs::Logger
pub fn logger(name: Cow<'static, str>) -> BoxedLogger {
    logger_provider().logger(name)
}

/// Sets the given [`LoggerProvider`] instance as the current global provider,
/// returning the [`LoggerProvider`] instance that was previously set as global
/// provider.
pub fn set_logger_provider<L, P>(new_provider: P) -> GlobalLoggerProvider
where
    L: Logger + Send + Sync + 'static,
    P: LoggerProvider<Logger = L> + Send + Sync + 'static,
{
    let mut provider = GLOBAL_LOGGER_PROVIDER
        .write()
        .expect("GLOBAL_LOGGER_PROVIDER RwLock poisoned");
    mem::replace(&mut *provider, GlobalLoggerProvider::new(new_provider))
}

/// Shut down the current global [`LoggerProvider`].
pub fn shutdown_logger_provider() {
    let _ = set_logger_provider(NoopLoggerProvider::new());
}
