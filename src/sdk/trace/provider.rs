//! # Trace Provider SDK
//!
//! ## Tracer Creation
//!
//! New `Tracer` instances are always created through a `Provider`.
//!
//! All configuration objects and extension points (span processors,
//! propagators) are provided by the `Provider`. `Tracer` instances do
//! not duplicate this data to avoid that different `Tracer` instances
//! of the `Provider` have different versions of these data.
use crate::exporter::trace::{jaeger, SpanExporter};
use crate::{api, sdk};
use std::collections::HashMap;
use std::sync::RwLock;

/// Default tracer name if empty string is provided.
const DEFAULT_COMPONENT_NAME: &str = "rust.opentelemetry.io/sdk/tracer";

/// Creator and registry of named `Tracer` instances.
#[derive(Debug)]
pub struct Provider {
    named_tracers: RwLock<HashMap<&'static str, sdk::Tracer>>,
    exporters: Vec<Box<dyn SpanExporter<Span = sdk::Span> + 'static>>,
    config: sdk::Config,
}

impl Default for Provider {
    fn default() -> Self {
        Provider::builder().build()
    }
}

impl Provider {
    /// Create a new `Provider` builder.
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Initialize a new `Tracer` by name.
    fn initialize_tracer(&self, component_name: &'static str) -> sdk::Tracer {
        // TODO allow non-jaeger exporter and multiple exporters
        let span_sender = self
            .exporters
            .first()
            .expect("Exporters cannot be empty")
            .as_any()
            .downcast_ref::<jaeger::Exporter>()
            .expect("Only jaeger exporters allowed")
            .span_sender
            .clone();

        let tracer = match self.config.default_sampler {
            api::Sampler::Always => jaeger::Tracer::with_sender(jaeger::AllSampler, span_sender),
            api::Sampler::Never => jaeger::Tracer::with_sender(jaeger::NullSampler, span_sender),
            api::Sampler::Parent => {
                jaeger::Tracer::with_sender(jaeger::PassiveSampler, span_sender)
            }
            api::Sampler::Probability(prob) => {
                // rustracing_jaeger does not like >1 or < 0

                let prob = if prob > 1.0 {
                    1.0
                } else if prob < 0.0 {
                    0.0
                } else {
                    prob
                };
                jaeger::Tracer::with_sender(
                    jaeger::ProbabilisticSampler::new(prob).unwrap(),
                    span_sender,
                )
            }
        };

        sdk::Tracer::new(component_name, tracer)
    }
}

impl api::Provider for Provider {
    /// This implementation of `api::Provider` produces `sdk::Tracer` instances.
    type Tracer = sdk::Tracer;

    /// Find or create `Tracer` instance by name.
    fn get_tracer(&self, name: &'static str) -> Self::Tracer {
        // Use default value if name is invalid empty string
        let component_name = if name.is_empty() {
            DEFAULT_COMPONENT_NAME
        } else {
            name
        };

        // Return named tracer if already initialized
        if let Some(tracer) = self
            .named_tracers
            .read()
            .expect("RwLock poisoned")
            .get(&component_name)
        {
            return tracer.clone();
        };

        // Else construct new named tracer
        let mut tracers = self.named_tracers.write().expect("RwLock poisoned");
        let new_tracer = self.initialize_tracer(component_name);
        tracers.insert(component_name, new_tracer.clone());

        new_tracer
    }
}

/// Builder for provider attributes.
#[derive(Default, Debug)]
pub struct Builder {
    exporters: Vec<Box<dyn SpanExporter<Span = sdk::Span>>>,
    config: sdk::Config,
}

impl Builder {
    /// The `SpanExporter` that this provider should use.
    pub fn with_exporter<T: SpanExporter<Span = sdk::Span> + 'static>(self, exporter: T) -> Self {
        let Builder {
            mut exporters,
            config,
        } = self;
        exporters.push(Box::new(exporter));

        Builder { exporters, config }
    }

    /// The sdk `Config` that this provider will use.
    pub fn with_config(self, config: sdk::Config) -> Self {
        Builder { config, ..self }
    }

    /// Create a new provider from this configuration.
    pub fn build(self) -> Provider {
        let Builder {
            mut exporters,
            config,
        } = self;
        // TODO: This can be removed when the exporter is used directly.
        let has_jaeger_exporter = exporters
            .iter()
            .any(|e| e.as_any().downcast_ref::<jaeger::Exporter>().is_some());
        if !has_jaeger_exporter {
            exporters.push(Box::new(jaeger::Exporter::init_default()))
        };

        Provider {
            named_tracers: Default::default(),
            exporters,
            config,
        }
    }
}
