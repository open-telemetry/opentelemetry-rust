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
use crate::{sdk, sdk::trace::SpanProcessor};
use std::sync::Arc;

/// Default tracer name if empty string is provided.
const DEFAULT_COMPONENT_NAME: &str = "rust.opentelemetry.io/sdk/tracer";

/// TracerProvider inner type
#[derive(Debug)]
pub(crate) struct TracerProviderInner {
    processors: Vec<Box<dyn SpanProcessor>>,
    config: sdk::trace::Config,
}

impl Drop for TracerProviderInner {
    fn drop(&mut self) {
        for processor in &mut self.processors {
            let _result = processor.shutdown();
        }
    }
}

/// Creator and registry of named `Tracer` instances.
#[derive(Clone, Debug)]
pub struct TracerProvider {
    inner: Arc<TracerProviderInner>,
}

impl Default for TracerProvider {
    fn default() -> Self {
        TracerProvider::builder().build()
    }
}

impl TracerProvider {
    /// Build a new tracer provider
    pub(crate) fn new(inner: Arc<TracerProviderInner>) -> Self {
        TracerProvider { inner }
    }

    /// Create a new `TracerProvider` builder.
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Span processors associated with this provider
    pub fn span_processors(&self) -> &Vec<Box<dyn SpanProcessor>> {
        &self.inner.processors
    }

    /// Config associated with this tracer
    pub fn config(&self) -> &sdk::trace::Config {
        &self.inner.config
    }
}

impl crate::trace::TracerProvider for TracerProvider {
    /// This implementation of `TracerProvider` produces `Tracer` instances.
    type Tracer = sdk::trace::Tracer;

    /// Find or create `Tracer` instance by name.
    fn get_tracer(&self, name: &'static str, version: Option<&'static str>) -> Self::Tracer {
        // Use default value if name is invalid empty string
        let component_name = if name.is_empty() {
            DEFAULT_COMPONENT_NAME
        } else {
            name
        };
        let instrumentation_lib = sdk::InstrumentationLibrary::new(component_name, version);

        sdk::trace::Tracer::new(instrumentation_lib, Arc::downgrade(&self.inner))
    }
}

/// Builder for provider attributes.
#[derive(Default, Debug)]
pub struct Builder {
    processors: Vec<Box<dyn SpanProcessor>>,
    config: sdk::trace::Config,
}

impl Builder {
    /// The `SpanExporter` that this provider should use.
    pub fn with_simple_exporter<T: SpanExporter + 'static>(self, exporter: T) -> Self {
        let mut processors = self.processors;
        processors.push(Box::new(sdk::trace::SimpleSpanProcessor::new(Box::new(
            exporter,
        ))));

        Builder { processors, ..self }
    }

    /// The `BatchProcessor` that this provider should use.
    pub fn with_batch_exporter(self, processor: sdk::trace::BatchSpanProcessor) -> Self {
        let mut processors = self.processors;
        processors.push(Box::new(processor));

        Builder { processors, ..self }
    }

    /// Add a configured `SpanExporter`
    #[cfg(feature = "tokio")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
    pub fn with_exporter<T: SpanExporter + 'static>(self, exporter: T) -> Self {
        // This future blocks currently when remaining spans are exported on processor
        // drop. We cannot assume we are in a multi-threaded tokio runtime here, so use
        // `spawn_blocking` to avoid blocking the main thread.
        let spawn = |fut| tokio::task::spawn_blocking(|| futures::executor::block_on(fut));
        let batch = sdk::trace::BatchSpanProcessor::builder(
            exporter,
            spawn,
            tokio::time::delay_for,
            tokio::time::interval,
        );
        self.with_batch_exporter(batch.build())
    }

    /// Add a configured `SpanExporter`
    #[cfg(all(feature = "async-std", not(feature = "tokio")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
    pub fn with_exporter<T: SpanExporter + 'static>(self, exporter: T) -> Self {
        let batch = sdk::trace::BatchSpanProcessor::builder(
            exporter,
            async_std::task::spawn,
            async_std::task::sleep,
            async_std::stream::interval,
        );
        self.with_batch_exporter(batch.build())
    }

    /// Add a configured `SpanExporter`
    #[cfg(all(not(feature = "async-std"), not(feature = "tokio")))]
    pub fn with_exporter<T: SpanExporter + 'static>(self, exporter: T) -> Self {
        self.with_simple_exporter(exporter)
    }

    /// The `SpanProcessor` that this provider should use.
    pub fn with_span_processor<T: SpanProcessor + 'static>(self, processor: T) -> Self {
        let mut processors = self.processors;
        processors.push(Box::new(processor));

        Builder { processors, ..self }
    }

    /// The sdk `Config` that this provider will use.
    pub fn with_config(self, config: sdk::trace::Config) -> Self {
        Builder { config, ..self }
    }

    /// Create a new provider from this configuration.
    pub fn build(self) -> TracerProvider {
        TracerProvider {
            inner: Arc::new(TracerProviderInner {
                processors: self.processors,
                config: self.config,
            }),
        }
    }
}
