//! # Trace Provider SDK
//!
//! ## Tracer Creation
//!
//! New `Tracer` instances are always created through a `TracerProvider`.
//!
//! All configuration objects and extension points (span processors,
//! propagators) are provided by the `TracerProvider`. `Tracer` instances do
//! not duplicate this data to avoid that different `Tracer` instances
//! of the `TracerProvider` have different versions of these data.
use crate::exporter::trace::SpanExporter;
use crate::{api, sdk};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Default tracer name if empty string is provided.
const DEFAULT_COMPONENT_NAME: &str = "rust.opentelemetry.io/sdk/tracer";

/// TracerProvider inner type
#[derive(Debug)]
struct ProviderInner {
    named_tracers: RwLock<HashMap<sdk::InstrumentationLibrary, sdk::Tracer>>,
    processors: Vec<Box<dyn api::SpanProcessor>>,
    config: sdk::Config,
}

impl Drop for ProviderInner {
    fn drop(&mut self) {
        for processor in &self.processors {
            processor.shutdown();
        }
    }
}

/// Creator and registry of named `Tracer` instances.
#[derive(Clone, Debug)]
pub struct TracerProvider {
    inner: Arc<ProviderInner>,
}

impl Default for TracerProvider {
    fn default() -> Self {
        TracerProvider::builder().build()
    }
}

impl TracerProvider {
    /// Create a new `TracerProvider` builder.
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Span processors associated with this provider
    pub fn span_processors(&self) -> &Vec<Box<dyn api::SpanProcessor>> {
        &self.inner.processors
    }

    /// Config associated with this tracer
    pub fn config(&self) -> &sdk::Config {
        &self.inner.config
    }
}

impl api::TracerProvider for TracerProvider {
    /// This implementation of `api::TraceProvider` produces `sdk::Tracer` instances.
    type Tracer = sdk::Tracer;

    /// Find or create `Tracer` instance by name.
    fn get_tracer(&self, name: &'static str, version: Option<&'static str>) -> Self::Tracer {
        // Use default value if name is invalid empty string
        let component_name = if name.is_empty() {
            DEFAULT_COMPONENT_NAME
        } else {
            name
        };

        let instrumentation_lib = sdk::InstrumentationLibrary::new(component_name, version.unwrap_or(""));

        // Return named tracer if already initialized
        if let Some(tracer) = self
            .inner
            .named_tracers
            .read()
            .expect("RwLock poisoned")
            .get(&instrumentation_lib)
        {
            return tracer.clone();
        };

        // Else construct new named tracer
        let mut tracers = self.inner.named_tracers.write().expect("RwLock poisoned");
        let new_tracer = sdk::Tracer::new(instrumentation_lib, self.clone());
        tracers.insert(instrumentation_lib, new_tracer.clone());

        new_tracer
    }
}

/// Builder for provider attributes.
#[derive(Default, Debug)]
pub struct Builder {
    processors: Vec<Box<dyn api::SpanProcessor>>,
    config: sdk::Config,
}

impl Builder {
    /// The `SpanExporter` that this provider should use.
    pub fn with_simple_exporter<T: SpanExporter + 'static>(self, exporter: T) -> Self {
        let mut processors = self.processors;
        processors.push(Box::new(sdk::SimpleSpanProcessor::new(Box::new(exporter))));

        Builder { processors, ..self }
    }

    /// The `BatchProcessor` that this provider should use.
    pub fn with_batch_exporter(self, processor: sdk::BatchSpanProcessor) -> Self {
        let mut processors = self.processors;
        processors.push(Box::new(processor));

        Builder { processors, ..self }
    }

    /// The `SpanProcessor` that this provider should use.
    pub fn with_span_processor<T: api::SpanProcessor + 'static>(self, processor: T) -> Self {
        let mut processors = self.processors;
        processors.push(Box::new(processor));

        Builder { processors, ..self }
    }

    /// The sdk `Config` that this provider will use.
    pub fn with_config(self, config: sdk::Config) -> Self {
        Builder { config, ..self }
    }

    /// Create a new provider from this configuration.
    pub fn build(self) -> TracerProvider {
        TracerProvider {
            inner: Arc::new(ProviderInner {
                named_tracers: Default::default(),
                processors: self.processors,
                config: self.config,
            }),
        }
    }
}
