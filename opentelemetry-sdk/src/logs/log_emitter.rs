use super::{BatchLogProcessor, Config, LogProcessor, LogRuntime, SimpleLogProcessor};
use crate::export::logs::{LogData, LogExporter};
use opentelemetry_api::{
    global::{self},
    logs::{LogRecord, LogResult},
    trace::TraceContextExt,
    Context, InstrumentationLibrary,
};
use std::{
    borrow::Cow,
    sync::{Arc, Weak},
};

#[derive(Debug, Clone)]
/// Creator for `Logger` instances.
pub struct LoggerProvider {
    inner: Arc<LoggerProviderInner>,
}

/// Default logger name if empty string is provided.
const DEFAULT_COMPONENT_NAME: &str = "rust.opentelemetry.io/sdk/logger";

impl opentelemetry_api::logs::LoggerProvider for LoggerProvider {
    type Logger = Logger;

    /// Create a new versioned `Logger` instance.
    fn versioned_logger(
        &self,
        name: impl Into<Cow<'static, str>>,
        version: Option<Cow<'static, str>>,
        schema_url: Option<Cow<'static, str>>,
        attributes: Option<Vec<opentelemetry_api::KeyValue>>,
        include_trace_context: bool,
    ) -> Logger {
        let name = name.into();

        let component_name = if name.is_empty() {
            Cow::Borrowed(DEFAULT_COMPONENT_NAME)
        } else {
            name
        };

        Logger::new(
            InstrumentationLibrary::new(component_name, version, schema_url, attributes),
            Arc::downgrade(&self.inner),
            include_trace_context,
        )
    }
}

impl LoggerProvider {
    /// Build a new logger provider.
    pub(crate) fn new(inner: Arc<LoggerProviderInner>) -> Self {
        LoggerProvider { inner }
    }

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
            .expect("canont shutdown LoggerProvider when child Loggers are still active")
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
pub(crate) struct LoggerProviderInner {
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
    pub fn with_batch_exporter<T: LogExporter + 'static, R: LogRuntime>(
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
/// [`LogRecord`]: opentelemetry_api::logs::LogRecord
pub struct Logger {
    include_trace_context: bool,
    instrumentation_lib: InstrumentationLibrary,
    provider: Weak<LoggerProviderInner>,
}

impl Logger {
    pub(crate) fn new(
        instrumentation_lib: InstrumentationLibrary,
        provider: Weak<LoggerProviderInner>,
        include_trace_context: bool,
    ) -> Self {
        Logger {
            include_trace_context,
            instrumentation_lib,
            provider,
        }
    }

    /// LoggerProvider associated with this logger.
    pub fn provider(&self) -> Option<LoggerProvider> {
        self.provider.upgrade().map(LoggerProvider::new)
    }

    /// Instrumentation library information of this logger.
    pub fn instrumentation_library(&self) -> &InstrumentationLibrary {
        &self.instrumentation_lib
    }
}

impl opentelemetry_api::logs::Logger for Logger {
    /// Emit a `LogRecord`.
    fn emit(&self, record: LogRecord) {
        let provider = match self.provider() {
            Some(provider) => provider,
            None => return,
        };

        let config = provider.config();
        for processor in provider.log_processors() {
            let mut record = record.clone();
            if self.include_trace_context {
                let ctx = Context::current();
                if ctx.has_active_span() {
                    let span = ctx.span();
                    record.trace_context = Some(span.span_context().into());
                }
            }
            let data = LogData {
                record,
                resource: config.resource.clone(),
                instrumentation: self.instrumentation_lib.clone(),
            };
            processor.emit(data);
        }
    }
}
