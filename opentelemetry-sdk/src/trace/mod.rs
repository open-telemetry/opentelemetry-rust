//! # OpenTelemetry Trace SDK
//!
//! The tracing SDK consist of a few main structs:
//!
//! * The [`Tracer`] struct which performs all tracing operations.
//! * The [`Span`] struct with is a mutable object storing information about the
//! current operation execution.
//! * The [`TracerProvider`] struct which configures and produces [`Tracer`]s.
mod config;
mod evicted_hash_map;
mod evicted_queue;
mod id_generator;
mod provider;
mod sampler;
mod span;
mod span_limit;
mod span_processor;
mod tracer;

pub use config::{config, Config};
pub use evicted_hash_map::EvictedHashMap;
pub use evicted_queue::EvictedQueue;
pub use id_generator::{aws::XrayIdGenerator, IdGenerator, RandomIdGenerator};
pub use provider::{Builder, TracerProvider};
pub use sampler::{Sampler, ShouldSample};
pub use span::Span;
pub use span_limit::SpanLimits;
pub use span_processor::{
    BatchConfig, BatchMessage, BatchSpanProcessor, BatchSpanProcessorBuilder, SimpleSpanProcessor,
    SpanProcessor,
};
pub use tracer::Tracer;

#[cfg(feature = "jaeger_remote_sampler")]
pub use sampler::{JaegerRemoteSampler, JaegerRemoteSamplerBuilder};

#[cfg(test)]
mod runtime_tests;

#[cfg(all(test, feature = "testing"))]
mod tests {
    use super::*;
    use crate::testing::trace::InMemorySpanExporterBuilder;
    use opentelemetry::{
        trace::{Span, Tracer, TracerProvider as _},
        KeyValue,
    };

    #[test]
    fn tracing_in_span() {
        // Arrange
        let exporter = InMemorySpanExporterBuilder::new().build();
        let provider = TracerProvider::builder()
            .with_span_processor(SimpleSpanProcessor::new(Box::new(exporter.clone())))
            .build();

        // Act
        let tracer = provider.tracer("test_tracer");
        tracer.in_span("span_name", |_cx| {});

        provider.force_flush();

        // Assert
        let exported_spans = exporter
            .get_finished_spans()
            .expect("Spans are expected to be exported.");
        assert_eq!(exported_spans.len(), 1);
        let span = &exported_spans[0];
        assert_eq!(span.name, "span_name");
        assert_eq!(span.instrumentation_lib.name, "test_tracer");
    }

    #[test]
    fn tracing_tracer_start() {
        // Arrange
        let exporter = InMemorySpanExporterBuilder::new().build();
        let provider = TracerProvider::builder()
            .with_span_processor(SimpleSpanProcessor::new(Box::new(exporter.clone())))
            .build();

        // Act
        let tracer = provider.tracer("test_tracer");
        let mut span = tracer.start("span_name");
        span.set_attribute(KeyValue::new("key1", "value1"));
        drop(span);
        provider.force_flush();

        // Assert
        let exported_spans = exporter
            .get_finished_spans()
            .expect("Spans are expected to be exported.");
        assert_eq!(exported_spans.len(), 1);
        let span = &exported_spans[0];
        assert_eq!(span.name, "span_name");
        assert_eq!(span.instrumentation_lib.name, "test_tracer");
    }
}
