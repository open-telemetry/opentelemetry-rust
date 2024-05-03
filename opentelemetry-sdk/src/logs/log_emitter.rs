use super::{BatchLogProcessor, LogProcessor, LogRecord, SimpleLogProcessor, TraceContext};
use crate::{
    export::logs::{LogData, LogExporter},
    runtime::RuntimeChannel,
    Resource,
};
use opentelemetry::{
    global,
    logs::{LogError, LogResult},
    trace::TraceContextExt,
    Context, InstrumentationLibrary,
};

#[cfg(feature = "logs_level_enabled")]
use opentelemetry::logs::Severity;

use std::{
    borrow::Cow,
    sync::{atomic::Ordering, Arc},
};
use std::{sync::atomic::AtomicBool, time::SystemTime};

use once_cell::sync::Lazy;

// a no nop logger provider used as placeholder when the provider is shutdown
static NOOP_LOGGER_PROVIDER: Lazy<LoggerProvider> = Lazy::new(|| LoggerProvider {
    inner: Arc::new(LoggerProviderInner {
        processors: Vec::new(),
        resource: Resource::empty(),
    }),
    is_shutdown: Arc::new(AtomicBool::new(true)),
});

#[derive(Debug, Clone)]
/// Creator for `Logger` instances.
pub struct LoggerProvider {
    inner: Arc<LoggerProviderInner>,
    is_shutdown: Arc<AtomicBool>,
}

/// Default logger name if empty string is provided.
const DEFAULT_COMPONENT_NAME: &str = "rust.opentelemetry.io/sdk/logger";
// According to a Go-specific study mentioned on https://go.dev/blog/slog,
// up to 5 attributes is the most common case. We have chosen 8 as the default
// capacity for attributes to avoid reallocation in common scenarios.
const PREALLOCATED_ATTRIBUTE_CAPACITY: usize = 8;

impl opentelemetry::logs::LoggerProvider for LoggerProvider {
    type Logger = Logger;

    /// Create a new versioned `Logger` instance.
    fn versioned_logger(
        &self,
        name: impl Into<Cow<'static, str>>,
        version: Option<Cow<'static, str>>,
        schema_url: Option<Cow<'static, str>>,
        attributes: Option<Vec<opentelemetry::KeyValue>>,
    ) -> Logger {
        let name = name.into();

        let component_name = if name.is_empty() {
            Cow::Borrowed(DEFAULT_COMPONENT_NAME)
        } else {
            name
        };

        let mut builder = self.logger_builder(component_name);

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

    fn library_logger(&self, library: Arc<InstrumentationLibrary>) -> Self::Logger {
        // If the provider is shutdown, new logger will refer a no-op logger provider.
        if self.is_shutdown.load(Ordering::Relaxed) {
            return Logger::new(library, NOOP_LOGGER_PROVIDER.clone());
        }
        Logger::new(library, self.clone())
    }
}

impl LoggerProvider {
    /// Create a new `LoggerProvider` builder.
    pub fn builder() -> Builder {
        Builder::default()
    }

    pub(crate) fn log_processors(&self) -> &[Box<dyn LogProcessor>] {
        &self.inner.processors
    }

    pub(crate) fn resource(&self) -> &Resource {
        &self.inner.resource
    }

    /// Force flush all remaining logs in log processors and return results.
    pub fn force_flush(&self) -> Vec<LogResult<()>> {
        self.log_processors()
            .iter()
            .map(|processor| processor.force_flush())
            .collect()
    }

    /// Shuts down this `LoggerProvider`
    pub fn shutdown(&self) -> LogResult<()> {
        if self
            .is_shutdown
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            // propagate the shutdown signal to processors
            // it's up to the processor to properly block new logs after shutdown
            let mut errs = vec![];
            for processor in &self.inner.processors {
                if let Err(err) = processor.shutdown() {
                    errs.push(err);
                }
            }

            if errs.is_empty() {
                Ok(())
            } else {
                Err(LogError::Other(format!("{errs:?}").into()))
            }
        } else {
            Err(LogError::Other("logger provider already shut down".into()))
        }
    }
}

#[derive(Debug)]
struct LoggerProviderInner {
    processors: Vec<Box<dyn LogProcessor>>,
    resource: Resource,
}

impl Drop for LoggerProviderInner {
    fn drop(&mut self) {
        for processor in &mut self.processors {
            if let Err(err) = processor.shutdown() {
                global::handle_error(err);
            }
        }
    }
}

#[derive(Debug, Default)]
/// Builder for provider attributes.
pub struct Builder {
    processors: Vec<Box<dyn LogProcessor>>,
    resource: Option<Resource>,
}

impl Builder {
    /// The `LogExporter` that this provider should use.
    pub fn with_simple_exporter<T: LogExporter + 'static>(self, exporter: T) -> Self {
        let mut processors = self.processors;
        processors.push(Box::new(SimpleLogProcessor::new(Box::new(exporter))));

        Builder { processors, ..self }
    }

    /// The `LogExporter` setup using a default `BatchLogProcessor` that this provider should use.
    pub fn with_batch_exporter<T: LogExporter + 'static, R: RuntimeChannel>(
        self,
        exporter: T,
        runtime: R,
    ) -> Self {
        let batch = BatchLogProcessor::builder(exporter, runtime).build();
        self.with_log_processor(batch)
    }

    /// The `LogProcessor` that this provider should use.
    pub fn with_log_processor<T: LogProcessor + 'static>(self, processor: T) -> Self {
        let mut processors = self.processors;
        processors.push(Box::new(processor));

        Builder { processors, ..self }
    }

    /// The `Resource` to be associated with this Provider.
    pub fn with_resource(self, resource: Resource) -> Self {
        Builder {
            resource: Some(resource),
            ..self
        }
    }

    /// Create a new provider from this configuration.
    pub fn build(self) -> LoggerProvider {
        let resource = self.resource.unwrap_or_default();

        let logger_provider = LoggerProvider {
            inner: Arc::new(LoggerProviderInner {
                processors: self.processors,
                resource,
            }),
            is_shutdown: Arc::new(AtomicBool::new(false)),
        };

        // invoke set_resource on all the processors
        for processor in logger_provider.log_processors() {
            processor.set_resource(logger_provider.resource());
        }
        logger_provider
    }
}

#[derive(Debug)]
/// The object for emitting [`LogRecord`]s.
///
/// [`LogRecord`]: opentelemetry::logs::LogRecord
pub struct Logger {
    instrumentation_lib: Arc<InstrumentationLibrary>,
    provider: LoggerProvider,
}

impl Logger {
    pub(crate) fn new(
        instrumentation_lib: Arc<InstrumentationLibrary>,
        provider: LoggerProvider,
    ) -> Self {
        Logger {
            instrumentation_lib,
            provider,
        }
    }

    /// LoggerProvider associated with this logger.
    pub fn provider(&self) -> &LoggerProvider {
        &self.provider
    }

    /// Instrumentation library information of this logger.
    pub fn instrumentation_library(&self) -> &InstrumentationLibrary {
        &self.instrumentation_lib
    }
}

impl opentelemetry::logs::Logger for Logger {
    type LogRecord = LogRecord;

    fn create_log_record(&self) -> Self::LogRecord {
        // Reserve attributes memory for perf optimization. This may change in future.
        LogRecord {
            attributes: Some(Vec::with_capacity(PREALLOCATED_ATTRIBUTE_CAPACITY)),
            ..Default::default()
        }
    }

    /// Emit a `LogRecord`.
    fn emit(&self, record: Self::LogRecord) {
        let provider = self.provider();
        let processors = provider.log_processors();
        let trace_context = Context::map_current(|cx| {
            cx.has_active_span()
                .then(|| TraceContext::from(cx.span().span_context()))
        });
        let mut log_record = record;
        if let Some(ref trace_context) = trace_context {
            log_record.trace_context = Some(trace_context.clone());
        }
        if log_record.observed_timestamp.is_none() {
            log_record.observed_timestamp = Some(SystemTime::now());
        }

        let mut data = LogData {
            record: log_record,
            instrumentation: self.instrumentation_library().clone(),
        };

        for p in processors {
            p.emit(&mut data);
        }
    }

    #[cfg(feature = "logs_level_enabled")]
    fn event_enabled(&self, level: Severity, target: &str) -> bool {
        let provider = self.provider();

        let mut enabled = false;
        for processor in provider.log_processors() {
            enabled = enabled
                || processor.event_enabled(
                    level,
                    target,
                    self.instrumentation_library().name.as_ref(),
                );
        }
        enabled
    }
}

#[cfg(test)]
mod tests {
    use crate::resource::{
        SERVICE_NAME, TELEMETRY_SDK_LANGUAGE, TELEMETRY_SDK_NAME, TELEMETRY_SDK_VERSION,
    };
    use crate::Resource;

    use super::*;
    use opentelemetry::logs::{Logger, LoggerProvider as _};
    use opentelemetry::{Key, KeyValue, Value};
    use std::fmt::{Debug, Formatter};
    use std::sync::atomic::AtomicU64;
    use std::sync::Mutex;
    use std::thread;

    struct ShutdownTestLogProcessor {
        is_shutdown: Arc<Mutex<bool>>,
        counter: Arc<AtomicU64>,
    }

    impl Debug for ShutdownTestLogProcessor {
        fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
            todo!()
        }
    }

    impl ShutdownTestLogProcessor {
        pub(crate) fn new(counter: Arc<AtomicU64>) -> Self {
            ShutdownTestLogProcessor {
                is_shutdown: Arc::new(Mutex::new(false)),
                counter,
            }
        }
    }

    impl LogProcessor for ShutdownTestLogProcessor {
        fn emit(&self, _data: &mut LogData) {
            self.is_shutdown
                .lock()
                .map(|is_shutdown| {
                    if !*is_shutdown {
                        self.counter
                            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    }
                })
                .expect("lock poisoned");
        }

        fn force_flush(&self) -> LogResult<()> {
            Ok(())
        }

        fn shutdown(&self) -> LogResult<()> {
            self.is_shutdown
                .lock()
                .map(|mut is_shutdown| *is_shutdown = true)
                .expect("lock poisoned");
            Ok(())
        }

        #[cfg(feature = "logs_level_enabled")]
        fn event_enabled(&self, _level: Severity, _target: &str, _name: &str) -> bool {
            true
        }
    }
    #[test]
    fn test_logger_provider_default_resource() {
        let assert_resource = |provider: &super::LoggerProvider,
                               resource_key: &'static str,
                               expect: Option<&'static str>| {
            assert_eq!(
                provider
                    .resource()
                    .get(Key::from_static_str(resource_key))
                    .map(|v| v.to_string()),
                expect.map(|s| s.to_string())
            );
        };
        let assert_telemetry_resource = |provider: &super::LoggerProvider| {
            assert_eq!(
                provider.resource().get(TELEMETRY_SDK_LANGUAGE.into()),
                Some(Value::from("rust"))
            );
            assert_eq!(
                provider.resource().get(TELEMETRY_SDK_NAME.into()),
                Some(Value::from("opentelemetry"))
            );
            assert_eq!(
                provider.resource().get(TELEMETRY_SDK_VERSION.into()),
                Some(Value::from(env!("CARGO_PKG_VERSION")))
            );
        };

        // If users didn't provide a resource and there isn't a env var set. Use default one.
        temp_env::with_var_unset("OTEL_RESOURCE_ATTRIBUTES", || {
            let default_config_provider = super::LoggerProvider::builder().build();
            assert_resource(
                &default_config_provider,
                SERVICE_NAME,
                Some("unknown_service"),
            );
            assert_telemetry_resource(&default_config_provider);
        });

        // If user provided a resource, use that.
        let custom_config_provider = super::LoggerProvider::builder()
            .with_resource(Resource::new(vec![KeyValue::new(
                SERVICE_NAME,
                "test_service",
            )]))
            .build();
        assert_resource(&custom_config_provider, SERVICE_NAME, Some("test_service"));
        assert_eq!(custom_config_provider.resource().len(), 1);

        // If `OTEL_RESOURCE_ATTRIBUTES` is set, read them automatically
        temp_env::with_var(
            "OTEL_RESOURCE_ATTRIBUTES",
            Some("key1=value1, k2, k3=value2"),
            || {
                let env_resource_provider = super::LoggerProvider::builder().build();
                assert_resource(
                    &env_resource_provider,
                    SERVICE_NAME,
                    Some("unknown_service"),
                );
                assert_resource(&env_resource_provider, "key1", Some("value1"));
                assert_resource(&env_resource_provider, "k3", Some("value2"));
                assert_telemetry_resource(&env_resource_provider);
                assert_eq!(env_resource_provider.resource().len(), 6);
            },
        );

        // When `OTEL_RESOURCE_ATTRIBUTES` is set and also user provided config
        temp_env::with_var(
            "OTEL_RESOURCE_ATTRIBUTES",
            Some("my-custom-key=env-val,k2=value2"),
            || {
                let user_provided_resource_config_provider = super::LoggerProvider::builder()
                    .with_resource(Resource::default().merge(&mut Resource::new(vec![
                        KeyValue::new("my-custom-key", "my-custom-value"),
                        KeyValue::new("my-custom-key2", "my-custom-value2"),
                    ])))
                    .build();
                assert_resource(
                    &user_provided_resource_config_provider,
                    SERVICE_NAME,
                    Some("unknown_service"),
                );
                assert_resource(
                    &user_provided_resource_config_provider,
                    "my-custom-key",
                    Some("my-custom-value"),
                );
                assert_resource(
                    &user_provided_resource_config_provider,
                    "my-custom-key2",
                    Some("my-custom-value2"),
                );
                assert_resource(
                    &user_provided_resource_config_provider,
                    "k2",
                    Some("value2"),
                );
                assert_telemetry_resource(&user_provided_resource_config_provider);
                assert_eq!(user_provided_resource_config_provider.resource().len(), 7);
            },
        );

        // If user provided a resource, it takes priority during collision.
        let no_service_name = super::LoggerProvider::builder()
            .with_resource(Resource::empty())
            .build();
        assert_eq!(no_service_name.resource().len(), 0);
    }

    #[test]
    fn shutdown_test() {
        let counter = Arc::new(AtomicU64::new(0));
        let logger_provider = LoggerProvider::builder()
            .with_log_processor(ShutdownTestLogProcessor::new(counter.clone()))
            .build();

        let logger1 = logger_provider.logger("test-logger1");
        let logger2 = logger_provider.logger("test-logger2");
        logger1.emit(logger1.create_log_record());
        logger2.emit(logger1.create_log_record());

        let logger3 = logger_provider.logger("test-logger3");
        let handle = thread::spawn(move || {
            logger3.emit(logger3.create_log_record());
        });
        handle.join().expect("thread panicked");

        let _ = logger_provider.shutdown();
        logger1.emit(logger1.create_log_record());

        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[test]
    fn shutdown_idempotent_test() {
        let counter = Arc::new(AtomicU64::new(0));
        let logger_provider = LoggerProvider::builder()
            .with_log_processor(ShutdownTestLogProcessor::new(counter.clone()))
            .build();

        let shutdown_res = logger_provider.shutdown();
        assert!(shutdown_res.is_ok());

        // Subsequent shutdowns should return an error.
        let shutdown_res = logger_provider.shutdown();
        assert!(shutdown_res.is_err());

        // Subsequent shutdowns should return an error.
        let shutdown_res = logger_provider.shutdown();
        assert!(shutdown_res.is_err());
    }

    #[test]
    fn global_shutdown_test() {
        // cargo test global_shutdown_test --features=testing

        // Arrange
        let shutdown_called = Arc::new(Mutex::new(false));
        let flush_called = Arc::new(Mutex::new(false));
        let logger_provider = LoggerProvider::builder()
            .with_log_processor(LazyLogProcessor::new(
                shutdown_called.clone(),
                flush_called.clone(),
            ))
            .build();
        //set_logger_provider(logger_provider);
        let logger1 = logger_provider.logger("test-logger1");
        let logger2 = logger_provider.logger("test-logger2");

        // Acts
        logger1.emit(logger1.create_log_record());
        logger2.emit(logger1.create_log_record());

        // explicitly calling shutdown on logger_provider. This will
        // indeed do the shutdown, even if there are loggers still alive.
        let _ = logger_provider.shutdown();

        // Assert

        // shutdown is called.
        assert!(*shutdown_called.lock().unwrap());

        // flush is never called by the sdk.
        assert!(!*flush_called.lock().unwrap());
    }

    #[derive(Debug)]
    pub(crate) struct LazyLogProcessor {
        shutdown_called: Arc<Mutex<bool>>,
        flush_called: Arc<Mutex<bool>>,
    }

    impl LazyLogProcessor {
        pub(crate) fn new(
            shutdown_called: Arc<Mutex<bool>>,
            flush_called: Arc<Mutex<bool>>,
        ) -> Self {
            LazyLogProcessor {
                shutdown_called,
                flush_called,
            }
        }
    }

    impl LogProcessor for LazyLogProcessor {
        fn emit(&self, _data: &mut LogData) {
            // nothing to do.
        }

        fn force_flush(&self) -> LogResult<()> {
            *self.flush_called.lock().unwrap() = true;
            Ok(())
        }

        fn shutdown(&self) -> LogResult<()> {
            *self.shutdown_called.lock().unwrap() = true;
            Ok(())
        }

        #[cfg(feature = "logs_level_enabled")]
        fn event_enabled(&self, _level: Severity, _target: &str, _name: &str) -> bool {
            true
        }
    }
}
