use super::{BatchLogProcessor, Config, LogProcessor, SimpleLogProcessor};
use crate::{
    export::logs::{LogData, LogExporter},
    runtime::RuntimeChannel,
};
use opentelemetry::{
    global::{self},
    logs::{LogRecord, LogResult, TraceContext},
    trace::TraceContextExt,
    Context, InstrumentationLibrary,
};

#[cfg(feature = "logs_level_enabled")]
use opentelemetry::logs::Severity;

use std::{borrow::Cow, sync::Arc};

#[derive(Debug, Clone)]
/// Creator for `Logger` instances.
pub struct LoggerProvider {
    inner: Arc<LoggerProviderInner>,
}

/// Default logger name if empty string is provided.
const DEFAULT_COMPONENT_NAME: &str = "rust.opentelemetry.io/sdk/logger";

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

        self.library_logger(Arc::new(InstrumentationLibrary::new(
            component_name,
            version,
            schema_url,
            attributes,
        )))
    }

    fn library_logger(&self, library: Arc<InstrumentationLibrary>) -> Self::Logger {
        Logger::new(library, self.clone())
    }
}

impl LoggerProvider {
    /// Create a new `LoggerProvider` builder.
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Config associated with this provider.
    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    /// Log processors associated with this provider.
    pub fn log_processors(&self) -> &Vec<Box<dyn LogProcessor>> {
        &self.inner.processors
    }

    /// Force flush all remaining logs in log processors and return results.
    pub fn force_flush(&self) -> Vec<LogResult<()>> {
        self.log_processors()
            .iter()
            .map(|processor| processor.force_flush())
            .collect()
    }

    /// Shuts down this `LoggerProvider`, panicking on failure.
    pub fn shutdown(&mut self) -> Vec<LogResult<()>> {
        self.try_shutdown()
            .expect("cannot shutdown LoggerProvider when child Loggers are still active")
    }

    /// Attempts to shutdown this `LoggerProvider`, succeeding only when
    /// all cloned `LoggerProvider` values have been dropped.
    pub fn try_shutdown(&mut self) -> Option<Vec<LogResult<()>>> {
        Arc::get_mut(&mut self.inner).map(|inner| {
            inner
                .processors
                .iter_mut()
                .map(|processor| processor.shutdown())
                .collect()
        })
    }
}

#[derive(Debug)]
struct LoggerProviderInner {
    processors: Vec<Box<dyn LogProcessor>>,
    config: Config,
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
    config: Config,
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

    /// The `Config` that this provider should use.
    pub fn with_config(self, config: Config) -> Self {
        Builder { config, ..self }
    }

    /// Create a new provider from this configuration.
    pub fn build(self) -> LoggerProvider {
        LoggerProvider {
            inner: Arc::new(LoggerProviderInner {
                processors: self.processors,
                config: self.config,
            }),
        }
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
    /// Emit a `LogRecord`.
    fn emit(&self, record: LogRecord) {
        let provider = self.provider();
        let config = provider.config();
        let processors = provider.log_processors();
        let trace_context = Context::map_current(|cx| {
            cx.has_active_span()
                .then(|| TraceContext::from(cx.span().span_context()))
        });
        for p in processors {
            let mut record = record.clone();
            if let Some(ref trace_context) = trace_context {
                record.trace_context = Some(trace_context.clone())
            }
            let data = LogData {
                record,
                resource: config.resource.clone(),
                instrumentation: self.instrumentation_library().clone(),
            };
            p.emit(data);
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
    use opentelemetry::global::{logger, set_logger_provider, shutdown_logger_provider};
    use opentelemetry::logs::Logger;
    use opentelemetry::{Key, KeyValue, Value};
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn test_logger_provider_default_resource() {
        let assert_resource = |provider: &super::LoggerProvider,
                               resource_key: &'static str,
                               expect: Option<&'static str>| {
            assert_eq!(
                provider
                    .config()
                    .resource
                    .get(Key::from_static_str(resource_key))
                    .map(|v| v.to_string()),
                expect.map(|s| s.to_string())
            );
        };
        let assert_telemetry_resource = |provider: &super::LoggerProvider| {
            assert_eq!(
                provider
                    .config()
                    .resource
                    .get(TELEMETRY_SDK_LANGUAGE.into()),
                Some(Value::from("rust"))
            );
            assert_eq!(
                provider.config().resource.get(TELEMETRY_SDK_NAME.into()),
                Some(Value::from("opentelemetry"))
            );
            assert_eq!(
                provider.config().resource.get(TELEMETRY_SDK_VERSION.into()),
                Some(Value::from(env!("CARGO_PKG_VERSION")))
            );
        };

        // If users didn't provide a resource and there isn't a env var set. Use default one.
        let default_config_provider = super::LoggerProvider::builder().build();
        assert_resource(
            &default_config_provider,
            SERVICE_NAME,
            Some("unknown_service"),
        );
        assert_telemetry_resource(&default_config_provider);

        // If user provided a resource, use that.
        let custom_config_provider = super::LoggerProvider::builder()
            .with_config(Config {
                resource: Cow::Owned(Resource::new(vec![KeyValue::new(
                    SERVICE_NAME,
                    "test_service",
                )])),
                ..Default::default()
            })
            .build();
        assert_resource(&custom_config_provider, SERVICE_NAME, Some("test_service"));

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
                assert_eq!(env_resource_provider.config().resource.len(), 6);
            },
        );

        // When `OTEL_RESOURCE_ATTRIBUTES` is set and also user provided config
        temp_env::with_var(
            "OTEL_RESOURCE_ATTRIBUTES",
            Some("my-custom-key=env-val,k2=value2"),
            || {
                let user_provided_resource_config_provider = super::LoggerProvider::builder()
                    .with_config(Config {
                        resource: Cow::Owned(Resource::default().merge(&mut Resource::new(vec![
                            KeyValue::new("my-custom-key", "my-custom-value"),
                        ]))),
                        ..Default::default()
                    })
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
                    "k2",
                    Some("value2"),
                );
                assert_telemetry_resource(&user_provided_resource_config_provider);
                assert_eq!(
                    user_provided_resource_config_provider
                        .config()
                        .resource
                        .len(),
                    6
                );
            },
        );

        // If user provided a resource, it takes priority during collision.
        let no_service_name = super::LoggerProvider::builder()
            .with_config(Config {
                resource: Cow::Owned(Resource::empty()),
                ..Default::default()
            })
            .build();

        assert_resource(&no_service_name, SERVICE_NAME, None);
    }

    #[test]
    fn shutdown_test() {
        // cargo test shutdown_test --features=logs

        // Arrange
        let shutdown_called = Arc::new(Mutex::new(false));
        let flush_called = Arc::new(Mutex::new(false));
        let signal_to_end = Arc::new(Mutex::new(false));
        let signal_to_thread_started = Arc::new(Mutex::new(false));
        let logger_provider = LoggerProvider::builder()
            .with_log_processor(LazyLogProcessor::new(
                shutdown_called.clone(),
                flush_called.clone(),
            ))
            .build();
        set_logger_provider(logger_provider);

        // Act
        let logger1 = logger("test-logger1");
        let logger2 = logger("test-logger2");
        logger1.emit(LogRecord::default());
        logger2.emit(LogRecord::default());

        let signal_to_end_clone = signal_to_end.clone();
        let signal_to_thread_started_clone = signal_to_thread_started.clone();

        let handle = thread::spawn(move || {
            let logger3 = logger("test-logger3");
            loop {
                // signal the main thread that this thread has started.
                *signal_to_thread_started_clone.lock().unwrap() = true;
                logger3.emit(LogRecord::default());
                if *signal_to_end_clone.lock().unwrap() {
                    break;
                }
            }
        });

        // wait for the spawned thread to start before calling shutdown This is
        // very important - if shutdown is called before the spawned thread
        // obtains its logger, then the logger will be no-op one, and the test
        // will pass, but it will not be testing the intended scenario.
        while !*signal_to_thread_started.lock().unwrap() {
            thread::sleep(std::time::Duration::from_millis(10));
        }

        // Intentionally *not* calling shutdown/flush on the provider, but
        // instead relying on shutdown_logger_provider which causes the global
        // provider to be dropped, leading to the sdk logger provider's drop to
        // be called, which is expected to call shutdown on processors.
        shutdown_logger_provider();

        // Assert

        // shutdown_logger_provider is necessary but not sufficient, as loggers
        // hold on to the the provider (via inner provider clones).
        assert!(!*shutdown_called.lock().unwrap());

        // flush is never called by the sdk.
        assert!(!*flush_called.lock().unwrap());

        // Drop one of the logger. Not enough!
        drop(logger1);
        assert!(!*shutdown_called.lock().unwrap());

        // drop logger2, which is the only remaining logger in this thread.
        // Still not enough!
        drop(logger2);
        assert!(!*shutdown_called.lock().unwrap());

        // now signal the spawned thread to end, which causes it to drop its
        // logger. Since that is the last logger, the provider (inner provider)
        // is finally dropped, triggering shutdown
        *signal_to_end.lock().unwrap() = true;
        handle.join().unwrap();
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
        fn emit(&self, _data: LogData) {
            // nothing to do.
        }

        fn force_flush(&self) -> LogResult<()> {
            *self.flush_called.lock().unwrap() = true;
            Ok(())
        }

        fn shutdown(&mut self) -> LogResult<()> {
            *self.shutdown_called.lock().unwrap() = true;
            Ok(())
        }

        #[cfg(feature = "logs_level_enabled")]
        fn event_enabled(&self, _level: Severity, _target: &str, _name: &str) -> bool {
            true
        }
    }
}
