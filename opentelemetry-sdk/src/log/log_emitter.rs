use super::{BatchLogProcessor, Config, LogProcessor, LogRecord, LogRuntime, SimpleLogProcessor};
use crate::{
    export::log::{LogExporter, ResourceLog},
    resource::{EnvResourceDetector, SdkProvidedResourceDetector},
    Resource,
};
use opentelemetry_api::{log::LogResult, InstrumentationLibrary};
use std::{
    borrow::Cow,
    sync::{Arc, Weak},
    time::Duration,
};

#[derive(Debug)]
/// Creator for `LogEmitter` instances.
pub struct LogEmitterProvider {
    inner: Arc<LogEmitterProviderInner>,
}

/// Default log emitter name if empty string is provided.
const DEFAULT_COMPONENT_NAME: &str = "rust.opentelemetry.io/sdk/logemitter";

impl LogEmitterProvider {
    /// Build a new log emitter provider.
    pub(crate) fn new(inner: Arc<LogEmitterProviderInner>) -> Self {
        LogEmitterProvider { inner }
    }

    /// Create a new `LogEmitterProvider` builder.
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Create a new `LogEmitter`.
    pub fn log_emitter(&self, name: impl Into<Cow<'static, str>>) -> LogEmitter {
        self.versioned_log_emitter(name, Some(env!("CARGO_PKG_VERSION")))
    }

    /// Create a new version `LogEmitter` instance.
    pub fn versioned_log_emitter(
        &self,
        name: impl Into<Cow<'static, str>>,
        version: Option<&'static str>,
    ) -> LogEmitter {
        let name = name.into();

        let component_name = if name.is_empty() {
            Cow::Borrowed(DEFAULT_COMPONENT_NAME)
        } else {
            name
        };

        LogEmitter::new(
            InstrumentationLibrary::new(component_name, version.map(Into::into), None),
            Arc::downgrade(&self.inner),
        )
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
}

#[derive(Debug)]
pub(crate) struct LogEmitterProviderInner {
    processors: Vec<Box<dyn LogProcessor>>,
    config: Config,
}

#[derive(Debug)]
/// Builder for provider attributes.
pub struct Builder {
    processors: Vec<Box<dyn LogProcessor>>,
    config: Config,
    sdk_provided_resource: Resource,
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            processors: Default::default(),
            config: Default::default(),
            sdk_provided_resource: Resource::from_detectors(
                Duration::from_secs(0),
                vec![
                    Box::new(SdkProvidedResourceDetector),
                    Box::new(EnvResourceDetector::new()),
                ],
            ),
        }
    }
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

    /// Return the clone of sdk provided resource.
    ///
    /// See <https://github.com/open-telemetry/opentelemetry-specification/blob/v1.8.0/specification/resource/sdk.md#sdk-provided-resource-attributes>
    /// for details.
    pub fn sdk_provided_resource(&self) -> Resource {
        self.sdk_provided_resource.clone()
    }

    /// Create a new provider from this configuration.
    pub fn build(self) -> LogEmitterProvider {
        let mut config = self.config;
        config.resource = match config.resource {
            None => Some(Arc::new(self.sdk_provided_resource)),
            Some(resource) => {
                if resource.is_empty() {
                    None
                } else {
                    Some(Arc::new(self.sdk_provided_resource.merge(resource)))
                }
            }
        };

        LogEmitterProvider {
            inner: Arc::new(LogEmitterProviderInner {
                processors: self.processors,
                config,
            }),
        }
    }
}

#[derive(Debug)]
/// The object for emitting [`LogRecord`]s.
///
/// [`LogRecord`]: crate::LogRecord
pub struct LogEmitter {
    instrumentation_lib: InstrumentationLibrary,
    provider: Weak<LogEmitterProviderInner>,
}

impl LogEmitter {
    pub(crate) fn new(
        instrumentation_lib: InstrumentationLibrary,
        provider: Weak<LogEmitterProviderInner>,
    ) -> Self {
        LogEmitter {
            instrumentation_lib,
            provider,
        }
    }

    /// LogEmitterProvider associated with this tracer.
    pub fn provider(&self) -> Option<LogEmitterProvider> {
        self.provider.upgrade().map(LogEmitterProvider::new)
    }

    /// Instrumentation library information of this tracer.
    pub fn instrumentation_library(&self) -> &InstrumentationLibrary {
        &self.instrumentation_lib
    }

    /// Emit a `LogRecord`.
    pub fn emit(&self, record: LogRecord) {
        let provider = match self.provider() {
            Some(provider) => provider,
            None => return,
        };

        let config = provider.config();
        for processor in provider.log_processors() {
            let data = ResourceLog {
                record: record.clone(),
                resource: config.resource.clone(),
                instrumentation: self.instrumentation_lib.clone(),
            };
            processor.emit(data);
        }
    }
}
