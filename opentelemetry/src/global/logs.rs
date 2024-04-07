use std::{
    borrow::Cow,
    fmt, mem,
    sync::{Arc, RwLock},
};

use once_cell::sync::Lazy;

use crate::logs::LogResult;
use crate::{
    logs::{Logger, LoggerProvider, NoopLoggerProvider},
    InstrumentationLibrary,
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
    fn boxed_logger(
        &self,
        library: Arc<InstrumentationLibrary>,
    ) -> Box<dyn Logger + Send + Sync + 'static>;

    /// shutdown the logger provider, logs emitted after this will not be processed.
    fn shutdown(&self) -> Vec<LogResult<()>>;
}

impl<L, P> ObjectSafeLoggerProvider for P
where
    L: Logger + Send + Sync + 'static,
    P: LoggerProvider<Logger = L>,
{
    fn boxed_logger(
        &self,
        library: Arc<InstrumentationLibrary>,
    ) -> Box<dyn Logger + Send + Sync + 'static> {
        Box::new(self.library_logger(library))
    }

    fn shutdown(&self) -> Vec<LogResult<()>> {
        self.shutdown()
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

    #[cfg(feature = "logs_level_enabled")]
    fn event_enabled(&self, level: crate::logs::Severity, target: &str) -> bool {
        self.0.event_enabled(level, target)
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

    fn library_logger(&self, library: Arc<InstrumentationLibrary>) -> Self::Logger {
        BoxedLogger(self.provider.boxed_logger(library))
    }

    fn shutdown(&self) -> Vec<LogResult<()>> {
        self.provider.shutdown()
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
pub fn logger(name: impl Into<Cow<'static, str>>) -> BoxedLogger {
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
    let logger_provider = set_logger_provider(NoopLoggerProvider::new());
    let _ = ObjectSafeLoggerProvider::shutdown(&logger_provider);
}
