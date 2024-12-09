use super::{BatchLogProcessor, LogProcessor, LogRecord, SimpleLogProcessor, TraceContext};
use crate::{export::logs::LogExporter, runtime::RuntimeChannel, Resource};
use crate::{logs::LogError, logs::LogResult};
use opentelemetry::{otel_debug, otel_info, trace::TraceContextExt, Context, InstrumentationScope};

#[cfg(feature = "spec_unstable_logs_enabled")]
use opentelemetry::logs::Severity;

use std::time::SystemTime;
use std::{
    borrow::Cow,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, OnceLock,
    },
};

// a no nop logger provider used as placeholder when the provider is shutdown
// TODO - replace it with LazyLock once it is stable
static NOOP_LOGGER_PROVIDER: OnceLock<LoggerProvider> = OnceLock::new();

#[inline]
fn noop_logger_provider() -> &'static LoggerProvider {
    NOOP_LOGGER_PROVIDER.get_or_init(|| LoggerProvider {
        inner: Arc::new(LoggerProviderInner {
            processors: Vec::new(),
            resource: Resource::empty(),
            is_shutdown: AtomicBool::new(true),
        }),
    })
}

#[derive(Debug, Clone)]
/// Handles the creation and coordination of [`Logger`]s.
///
/// All `Logger`s created by a `LoggerProvider` will share the same
/// [`Resource`] and have their created log records processed by the
/// configured log processors. This is a clonable handle to the `LoggerProvider`
/// itself, and cloning it will create a new reference, not a new instance of a
/// `LoggerProvider`. Dropping the last reference will trigger the shutdown of
/// the provider, ensuring that all remaining logs are flushed and no further
/// logs are processed. Shutdown can also be triggered manually by calling
/// the [`shutdown`](LoggerProvider::shutdown) method.
///
/// [`Logger`]: opentelemetry::logs::Logger
/// [`Resource`]: crate::Resource
pub struct LoggerProvider {
    inner: Arc<LoggerProviderInner>,
}

impl opentelemetry::logs::LoggerProvider for LoggerProvider {
    type Logger = Logger;

    fn logger(&self, name: impl Into<Cow<'static, str>>) -> Self::Logger {
        let scope = InstrumentationScope::builder(name).build();
        self.logger_with_scope(scope)
    }

    fn logger_with_scope(&self, scope: InstrumentationScope) -> Self::Logger {
        // If the provider is shutdown, new logger will refer a no-op logger provider.
        if self.inner.is_shutdown.load(Ordering::Relaxed) {
            otel_debug!(
                name: "LoggerProvider.NoOpLoggerReturned",
                logger_name = scope.name(),
            );
            return Logger::new(scope, noop_logger_provider().clone());
        }
        if scope.name().is_empty() {
            otel_info!(name: "LoggerNameEmpty",  message = "Logger name is empty; consider providing a meaningful name. Logger will function normally and the provided name will be used as-is.");
        };
        otel_debug!(
            name: "LoggerProvider.NewLoggerReturned",
            logger_name = scope.name(),
        );
        Logger::new(scope, self.clone())
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
        otel_debug!(
            name: "LoggerProvider.ShutdownInvokedByUser",
        );
        if self
            .inner
            .is_shutdown
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            // propagate the shutdown signal to processors
            let errs = self.inner.shutdown();
            if errs.is_empty() {
                Ok(())
            } else {
                Err(LogError::Other(format!("{errs:?}").into()))
            }
        } else {
            Err(LogError::AlreadyShutdown("LoggerProvider".to_string()))
        }
    }
}

#[derive(Debug)]
struct LoggerProviderInner {
    processors: Vec<Box<dyn LogProcessor>>,
    resource: Resource,
    is_shutdown: AtomicBool,
}

impl LoggerProviderInner {
    /// Shuts down the `LoggerProviderInner` and returns any errors.
    pub(crate) fn shutdown(&self) -> Vec<LogError> {
        let mut errs = vec![];
        for processor in &self.processors {
            if let Err(err) = processor.shutdown() {
                // Log at debug level because:
                //  - The error is also returned to the user for handling (if applicable)
                //  - Or the error occurs during `LoggerProviderInner::Drop` as part of telemetry shutdown,
                //    which is non-actionable by the user
                match err {
                    // specific handling for mutex poisioning
                    LogError::MutexPoisoned(_) => {
                        otel_debug!(
                            name: "LoggerProvider.Drop.ShutdownMutexPoisoned",
                        );
                    }
                    _ => {
                        otel_debug!(
                            name: "LoggerProvider.Drop.ShutdownError",
                            error = format!("{err}")
                        );
                    }
                }
                errs.push(err);
            }
        }
        errs
    }
}

impl Drop for LoggerProviderInner {
    fn drop(&mut self) {
        if !self.is_shutdown.load(Ordering::Relaxed) {
            otel_info!(
                name: "LoggerProvider.Drop",
                message = "Last reference of LoggerProvider dropped, initiating shutdown."
            );
            let _ = self.shutdown(); // errors are handled within shutdown
        } else {
            otel_debug!(
                name: "LoggerProvider.Drop.AlreadyShutdown",
                message = "LoggerProvider was already shut down; drop will not attempt shutdown again."
            );
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
        processors.push(Box::new(SimpleLogProcessor::new(exporter)));

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
                is_shutdown: AtomicBool::new(false),
            }),
        };

        // invoke set_resource on all the processors
        for processor in logger_provider.log_processors() {
            processor.set_resource(logger_provider.resource());
        }

        otel_debug!(
            name: "LoggerProvider.Built",
        );
        logger_provider
    }
}

#[derive(Debug)]
/// The object for emitting [`LogRecord`]s.
///
/// [`LogRecord`]: opentelemetry::logs::LogRecord
pub struct Logger {
    scope: InstrumentationScope,
    provider: LoggerProvider,
}

impl Logger {
    pub(crate) fn new(scope: InstrumentationScope, provider: LoggerProvider) -> Self {
        Logger { scope, provider }
    }

    #[cfg(test)]
    pub(crate) fn instrumentation_scope(&self) -> &InstrumentationScope {
        &self.scope
    }
}

impl opentelemetry::logs::Logger for Logger {
    type LogRecord = LogRecord;

    fn create_log_record(&self) -> Self::LogRecord {
        LogRecord::default()
    }

    /// Emit a `LogRecord`.
    fn emit(&self, mut record: Self::LogRecord) {
        let provider = &self.provider;
        let processors = provider.log_processors();

        //let mut log_record = record;
        if record.trace_context.is_none() {
            let trace_context = Context::map_current(|cx| {
                cx.has_active_span()
                    .then(|| TraceContext::from(cx.span().span_context()))
            });

            if let Some(ref trace_context) = trace_context {
                record.trace_context = Some(trace_context.clone());
            }
        }
        if record.observed_timestamp.is_none() {
            record.observed_timestamp = Some(SystemTime::now());
        }

        for p in processors {
            p.emit(&mut record, &self.scope);
        }
    }

    #[cfg(feature = "spec_unstable_logs_enabled")]
    fn event_enabled(&self, level: Severity, target: &str) -> bool {
        let provider = &self.provider;

        let mut enabled = false;
        for processor in provider.log_processors() {
            enabled = enabled || processor.event_enabled(level, target, self.scope.name().as_ref());
        }
        enabled
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        resource::{
            SERVICE_NAME, TELEMETRY_SDK_LANGUAGE, TELEMETRY_SDK_NAME, TELEMETRY_SDK_VERSION,
        },
        testing::logs::InMemoryLogExporter,
        trace::TracerProvider,
        Resource,
    };

    use super::*;
    use opentelemetry::logs::{AnyValue, LogRecord as _, Logger as _, LoggerProvider as _};
    use opentelemetry::trace::{SpanId, TraceId, Tracer as _, TracerProvider as _};
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
        fn emit(&self, _data: &mut LogRecord, _scope: &InstrumentationScope) {
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
    fn trace_context_test() {
        let exporter = InMemoryLogExporter::default();

        let logger_provider = LoggerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();

        let logger = logger_provider.logger("test-logger");

        let tracer_provider = TracerProvider::builder().build();

        let tracer = tracer_provider.tracer("test-tracer");

        tracer.in_span("test-span", |cx| {
            let ambient_ctxt = cx.span().span_context().clone();
            let explicit_ctxt = TraceContext {
                trace_id: TraceId::from_u128(13),
                span_id: SpanId::from_u64(14),
                trace_flags: None,
            };

            let mut ambient_ctxt_record = logger.create_log_record();
            ambient_ctxt_record.set_body(AnyValue::String("ambient".into()));

            let mut explicit_ctxt_record = logger.create_log_record();
            explicit_ctxt_record.set_body(AnyValue::String("explicit".into()));
            explicit_ctxt_record.set_trace_context(
                explicit_ctxt.trace_id,
                explicit_ctxt.span_id,
                explicit_ctxt.trace_flags,
            );

            logger.emit(ambient_ctxt_record);
            logger.emit(explicit_ctxt_record);

            let emitted = exporter.get_emitted_logs().unwrap();

            assert_eq!(
                Some(AnyValue::String("ambient".into())),
                emitted[0].record.body
            );
            assert_eq!(
                ambient_ctxt.trace_id(),
                emitted[0].record.trace_context.as_ref().unwrap().trace_id
            );
            assert_eq!(
                ambient_ctxt.span_id(),
                emitted[0].record.trace_context.as_ref().unwrap().span_id
            );

            assert_eq!(
                Some(AnyValue::String("explicit".into())),
                emitted[1].record.body
            );
            assert_eq!(
                explicit_ctxt.trace_id,
                emitted[1].record.trace_context.as_ref().unwrap().trace_id
            );
            assert_eq!(
                explicit_ctxt.span_id,
                emitted[1].record.trace_context.as_ref().unwrap().span_id
            );
        });
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

    #[test]
    fn drop_test_with_multiple_providers() {
        let shutdown_called = Arc::new(Mutex::new(false));
        let flush_called = Arc::new(Mutex::new(false));
        {
            // Create a shared LoggerProviderInner and use it across multiple providers
            let shared_inner = Arc::new(LoggerProviderInner {
                processors: vec![Box::new(LazyLogProcessor::new(
                    shutdown_called.clone(),
                    flush_called.clone(),
                ))],
                resource: Resource::empty(),
                is_shutdown: AtomicBool::new(false),
            });

            {
                let logger_provider1 = LoggerProvider {
                    inner: shared_inner.clone(),
                };
                let logger_provider2 = LoggerProvider {
                    inner: shared_inner.clone(),
                };

                let logger1 = logger_provider1.logger("test-logger1");
                let logger2 = logger_provider2.logger("test-logger2");

                logger1.emit(logger1.create_log_record());
                logger2.emit(logger1.create_log_record());

                // LoggerProviderInner should not be dropped yet, since both providers and `shared_inner`
                // are still holding a reference.
            }
            // At this point, both `logger_provider1` and `logger_provider2` are dropped,
            // but `shared_inner` still holds a reference, so `LoggerProviderInner` is NOT dropped yet.
        }
        // Verify shutdown was called during the drop of the shared LoggerProviderInner
        assert!(*shutdown_called.lock().unwrap());
        // Verify flush was not called during drop
        assert!(!*flush_called.lock().unwrap());
    }

    #[test]
    fn drop_after_shutdown_test_with_multiple_providers() {
        let shutdown_called = Arc::new(Mutex::new(0)); // Count the number of times shutdown is called
        let flush_called = Arc::new(Mutex::new(false));

        // Create a shared LoggerProviderInner and use it across multiple providers
        let shared_inner = Arc::new(LoggerProviderInner {
            processors: vec![Box::new(CountingShutdownProcessor::new(
                shutdown_called.clone(),
                flush_called.clone(),
            ))],
            resource: Resource::empty(),
            is_shutdown: AtomicBool::new(false),
        });

        // Create a scope to test behavior when providers are dropped
        {
            let logger_provider1 = LoggerProvider {
                inner: shared_inner.clone(),
            };
            let logger_provider2 = LoggerProvider {
                inner: shared_inner.clone(),
            };

            // Explicitly shut down the logger provider
            let shutdown_result = logger_provider1.shutdown();
            assert!(shutdown_result.is_ok());

            // Verify that shutdown was called exactly once
            assert_eq!(*shutdown_called.lock().unwrap(), 1);

            // LoggerProvider2 should observe the shutdown state but not trigger another shutdown
            let shutdown_result2 = logger_provider2.shutdown();
            assert!(shutdown_result2.is_err());

            // Both logger providers will be dropped at the end of this scope
        }

        // Verify that shutdown was only called once, even after drop
        assert_eq!(*shutdown_called.lock().unwrap(), 1);
    }

    #[test]
    fn test_empty_logger_name() {
        let exporter = InMemoryLogExporter::default();
        let logger_provider = LoggerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();
        let logger = logger_provider.logger("");
        let mut record = logger.create_log_record();
        record.set_body("Testing empty logger name".into());
        logger.emit(record);

        // Create a logger using a scope with an empty name
        let scope = InstrumentationScope::builder("").build();
        let scoped_logger = logger_provider.logger_with_scope(scope);
        let mut scoped_record = scoped_logger.create_log_record();
        scoped_record.set_body("Testing empty logger scope name".into());
        scoped_logger.emit(scoped_record);

        // Assert: Verify that the emitted logs are processed correctly
        let emitted_logs = exporter.get_emitted_logs().unwrap();
        assert_eq!(emitted_logs.len(), 2);
        // Assert the first log
        assert_eq!(
            emitted_logs[0].clone().record.body,
            Some(AnyValue::String("Testing empty logger name".into()))
        );
        assert_eq!(logger.scope.name(), "");

        // Assert the second log created through the scope
        assert_eq!(
            emitted_logs[1].clone().record.body,
            Some(AnyValue::String("Testing empty logger scope name".into()))
        );
        assert_eq!(scoped_logger.scope.name(), "");
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
        fn emit(&self, _data: &mut LogRecord, _scope: &InstrumentationScope) {
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
    }

    #[derive(Debug)]
    struct CountingShutdownProcessor {
        shutdown_count: Arc<Mutex<i32>>,
        flush_called: Arc<Mutex<bool>>,
    }

    impl CountingShutdownProcessor {
        fn new(shutdown_count: Arc<Mutex<i32>>, flush_called: Arc<Mutex<bool>>) -> Self {
            CountingShutdownProcessor {
                shutdown_count,
                flush_called,
            }
        }
    }

    impl LogProcessor for CountingShutdownProcessor {
        fn emit(&self, _data: &mut LogRecord, _scope: &InstrumentationScope) {
            // nothing to do
        }

        fn force_flush(&self) -> LogResult<()> {
            *self.flush_called.lock().unwrap() = true;
            Ok(())
        }

        fn shutdown(&self) -> LogResult<()> {
            let mut count = self.shutdown_count.lock().unwrap();
            *count += 1;
            Ok(())
        }
    }
}
