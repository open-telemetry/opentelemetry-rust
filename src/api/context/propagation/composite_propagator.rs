//! # Composite Propagator
//!
//! The composite propagator allows multiple propagators to be used stacked
//! together to inject or extract from multiple implementations.
use crate::api::{self, HttpTextFormat};
use std::fmt::Debug;
use std::collections::HashSet;

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
/// use opentelemetry::api::trace::b3_propagator::B3Encoding;
///
/// // First create 1 or more propagators
/// let b3_propagator = B3Propagator::with_encoding(B3Encoding::SingleHeader);
/// let trace_context_propagator = TraceContextPropagator::new();
///
/// // Then create a composite propagator
/// let composite_propagator = HttpTextCompositePropagator::new(vec![
///     Box::new(b3_propagator),
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
/// composite_propagator.inject_context(&Context::current_with_span(example_span), &mut injector);
///
/// // The injector now has both `X-B3` and `traceparent` headers
/// assert!(injector.get("b3").is_some());
/// assert!(injector.get("traceparent").is_some());
/// ```
#[derive(Debug)]
pub struct HttpTextCompositePropagator {
    propagators: Vec<Box<dyn HttpTextFormat + Send + Sync>>,
}

impl HttpTextCompositePropagator {
    /// Constructs a new propagator out of instances of [`HttpTextFormat`].
    ///
    /// [`HttpTextFormat`]: ../../trait.HttpTextFormat.html
    pub fn new(propagators: Vec<Box<dyn HttpTextFormat + Send + Sync>>) -> Self {
        HttpTextCompositePropagator { propagators }
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

    fn get_fields(&self) -> Vec<String> {
        self.propagators.iter()
            .map(|p| p.get_fields())
            .flatten()
            .collect::<HashSet<String>>()
            .into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::trace::b3_propagator::B3Encoding;
    use crate::api::TraceContextExt;
    use crate::api::{B3Propagator, Context, SpanContext, SpanId, TraceContextPropagator, TraceId};
    use std::collections::HashMap;

    fn test_data() -> Vec<(&'static str, &'static str)> {
        vec![
            ("b3", "00000000000000000000000000000001-0000000000000001-0"),
            (
                "traceparent",
                "00-00000000000000000000000000000001-0000000000000001-00",
            ),
        ]
    }

    #[derive(Debug)]
    struct TestSpan(api::SpanContext);

    impl api::Span for TestSpan {
        fn add_event_with_timestamp(
            &self,
            _name: String,
            _timestamp: std::time::SystemTime,
            _attributes: Vec<api::KeyValue>,
        ) {}
        fn span_context(&self) -> api::SpanContext {
            self.0.clone()
        }
        fn is_recording(&self) -> bool {
            false
        }
        fn set_attribute(&self, _attribute: api::KeyValue) {}
        fn set_status(&self, _code: api::StatusCode, _message: String) {}
        fn update_name(&self, _new_name: String) {}
        fn end(&self) {}
    }

    #[test]
    fn inject_multiple_propagators() {
        let b3 = B3Propagator::with_encoding(B3Encoding::SingleHeader);
        let trace_context = TraceContextPropagator::new();
        let composite_propagator = HttpTextCompositePropagator {
            propagators: vec![Box::new(b3), Box::new(trace_context)],
        };

        let cx = Context::default().with_span(TestSpan(SpanContext::new(
            TraceId::from_u128(1),
            SpanId::from_u64(1),
            0,
            false,
        )));
        let mut injector = HashMap::new();
        composite_propagator.inject_context(&cx, &mut injector);

        for (header_name, header_value) in test_data() {
            assert_eq!(injector.get(header_name), Some(&header_value.to_string()));
        }
    }

    #[test]
    fn extract_multiple_propagators() {
        let b3 = B3Propagator::with_encoding(B3Encoding::SingleHeader);
        let trace_context = TraceContextPropagator::new();
        let composite_propagator = HttpTextCompositePropagator {
            propagators: vec![Box::new(b3), Box::new(trace_context)],
        };

        for (header_name, header_value) in test_data() {
            let mut extractor = HashMap::new();
            extractor.insert(header_name.to_string(), header_value.to_string());
            assert_eq!(
                composite_propagator
                    .extract(&extractor)
                    .remote_span_context(),
                Some(&SpanContext::new(
                    TraceId::from_u128(1),
                    SpanId::from_u64(1),
                    0,
                    true,
                ))
            );
        }
    }

    #[test]
    fn test_get_fields() {
        let b3 = B3Propagator::with_encoding(B3Encoding::SingleHeader);
        let b3_fields = b3.get_fields();
        let trace_context = TraceContextPropagator::new();
        let trace_context_fields = trace_context.get_fields();
        let composite_propagator = HttpTextCompositePropagator {
            propagators: vec![Box::new(b3), Box::new(trace_context)],
        };

        let mut fields = composite_propagator.get_fields();
        fields.sort();

        let mut expected = vec![b3_fields, trace_context_fields].into_iter()
            .flatten()
            .collect::<Vec<String>>();
        expected.sort();
        expected.dedup();


        assert_eq!(fields, expected);
    }
}
