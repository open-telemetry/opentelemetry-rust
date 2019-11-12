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
use crate::exporter::trace::jaeger;
use crate::{api, sdk};
use std::collections::HashMap;
use std::sync::RwLock;
use std::thread;

/// Default tracer name if empty string is provided.
const DEFAULT_TRACER_NAME: &str = "rust.opentelemetry.io/sdk/tracer";

/// Creator and registry of named `Tracer` instances.
#[derive(Debug, Default)]
pub struct Provider {
    named_tracers: RwLock<HashMap<String, sdk::Tracer>>,
}

impl Provider {
    /// Create a new `Provider` with an empty set of `Tracer`s.
    pub fn new() -> Self {
        Provider {
            named_tracers: Default::default(),
        }
    }

    /// Initialize a new `Tracer` by name.
    fn initialize_tracer(&self, service_name: String) -> sdk::Tracer {
        let (span_tx, span_rx) = crossbeam_channel::bounded(10);
        let tracer = jaeger::Tracer::with_sender(jaeger::AllSampler, span_tx);

        // Spin up thread to report finished spans
        let _ = thread::Builder::new()
            .name("Jaeger span reporter".to_string())
            .spawn(move || {
                let reporter = jaeger::JaegerCompactReporter::new(&service_name)
                    .expect("Can't initialize jaeger reporter");
                for span in span_rx {
                    let _ = reporter.report(&[span]);
                }
            });

        sdk::Tracer(tracer)
    }
}

impl api::Provider for Provider {
    /// This implementation of `api::Provider` produces `sdk::Tracer` instances.
    type Tracer = sdk::Tracer;

    /// Find or create `Tracer` instance by name.
    fn get_tracer(&self, name: &'static str) -> Self::Tracer {
        // Use default value if name is invalid empty string
        let service_name = if name.is_empty() {
            DEFAULT_TRACER_NAME.to_string()
        } else {
            name.to_string()
        };

        // Return named tracer if already initialized
        if let Some(tracer) = self
            .named_tracers
            .read()
            .expect("RwLock poisoned")
            .get(&service_name)
        {
            return tracer.clone();
        };

        // Else construct new named tracer
        let mut tracers = self.named_tracers.write().expect("RwLock poisoned");
        let new_tracer = self.initialize_tracer(service_name.clone());
        tracers.insert(service_name, new_tracer.clone());

        new_tracer
    }
}
