//! # Composite Propagator
//!
//! The composite propagator allows multiple propagators to be used stacked
//! together to inject or extract from multiple implementations.
use crate::api::{self, FieldIter, HttpTextFormat};
use std::collections::HashSet;
use std::fmt::Debug;

/// A propagator that chains multiple [`HttpTextFormat`] propagators together,
/// injecting or extracting by their respective HTTP header names.
///
/// Injection and extraction from this propagator will preserve the order of the
/// injectors and extractors passed in during initialization.
///
/// [`HttpTextFormat`]: ../../trait.HttpTextFormat.html
///
/// # Examples
///
/// ```
/// use opentelemetry::api::*;
/// use opentelemetry::sdk;
/// use std::collections::HashMap;
///
/// // First create 1 or more propagators
/// let correlation_propagator = CorrelationContextPropagator::new();
/// let trace_context_propagator = TraceContextPropagator::new();
///
/// // Then create a composite propagator
/// let composite_propagator = HttpTextCompositePropagator::new(vec![
///     Box::new(correlation_propagator),
///     Box::new(trace_context_propagator),
/// ]);
///
/// // Then for a given implementation of `Injector`
/// let mut injector = HashMap::new();
///
/// // And a given span
/// let example_span = sdk::Provider::default().get_tracer("example-component").start("span-name");
///
/// // with the current context, call inject to add the headers
/// composite_propagator.inject_context(&Context::current_with_span(example_span)
///                                     .with_correlations(vec![KeyValue::new("test", "example")]),
///                                     &mut injector);
///
/// // The injector now has both `otcorrelations` and `traceparent` headers
/// assert!(injector.get("otcorrelations").is_some());
/// assert!(injector.get("traceparent").is_some());
/// ```
#[derive(Debug)]
pub struct HttpTextCompositePropagator {
    propagators: Vec<Box<dyn HttpTextFormat + Send + Sync>>,
    fields: Vec<String>,
}

impl HttpTextCompositePropagator {
    /// Constructs a new propagator out of instances of [`HttpTextFormat`].
    ///
    /// [`HttpTextFormat`]: ../../trait.HttpTextFormat.html
    pub fn new(propagators: Vec<Box<dyn HttpTextFormat + Send + Sync>>) -> Self {
        let mut fields = HashSet::new();
        for propagator in &propagators {
            for field in propagator.fields() {
                fields.insert(field.to_string());
            }
        }

        HttpTextCompositePropagator {
            propagators,
            fields: fields.into_iter().collect(),
        }
    }
}

impl HttpTextFormat for HttpTextCompositePropagator {
    /// Encodes the values of the `Context` and injects them into the `Injector`.
    fn inject_context(&self, context: &api::Context, injector: &mut dyn api::Injector) {
        for propagator in &self.propagators {
            propagator.inject_context(context, injector)
        }
    }

    /// Retrieves encoded `Context` information using the `Extractor`. If no data was
    /// retrieved OR if the retrieved data is invalid, then the current `Context` is
    /// returned.
    fn extract_with_context(
        &self,
        cx: &api::Context,
        extractor: &dyn api::Extractor,
    ) -> api::Context {
        self.propagators
            .iter()
            .fold(cx.clone(), |current_cx, propagator| {
                propagator.extract_with_context(&current_cx, extractor)
            })
    }

    fn fields(&self) -> FieldIter {
        FieldIter::new(self.fields.as_slice())
    }
}
