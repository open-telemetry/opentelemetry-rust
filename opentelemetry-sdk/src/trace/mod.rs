//! # OpenTelemetry Trace SDK
//!
//! The tracing SDK consist of a few main structs:
//!
//! * The [`Tracer`] struct which performs all tracing operations.
//! * The [`Span`] struct with is a mutable object storing information about the
//! current operation execution.
//! * The [`TracerProvider`] struct which configures and produces [`Tracer`]s.
mod config;
mod events;
mod id_generator;
mod links;
mod provider;
mod sampler;
mod span;
mod span_limit;
mod span_processor;
mod tracer;

pub use config::{config, Config};
pub use events::SpanEvents;

#[deprecated(
    since = "0.21.3",
    note = "XrayId Generator has been migrated to the opentelemetry-aws crate"
)]
pub use id_generator::aws::XrayIdGenerator;
pub use id_generator::{IdGenerator, RandomIdGenerator};
pub use links::SpanLinks;
pub use provider::{Builder, TracerProvider};
pub use sampler::{Sampler, ShouldSample};
pub use span::Span;
pub use span_limit::SpanLimits;
pub use span_processor::{
    BatchConfig, BatchConfigBuilder, BatchSpanProcessor, BatchSpanProcessorBuilder,
    SimpleSpanProcessor, SpanProcessor,
};
pub use tracer::Tracer;

#[cfg(feature = "jaeger_remote_sampler")]
pub use sampler::{JaegerRemoteSampler, JaegerRemoteSamplerBuilder};

#[cfg(test)]
mod runtime_tests;

#[cfg(all(test, feature = "testing"))]
mod tests {
    use super::*;
    use crate::{
        testing::trace::InMemorySpanExporterBuilder,
        trace::span_limit::{DEFAULT_MAX_EVENT_PER_SPAN, DEFAULT_MAX_LINKS_PER_SPAN},
    };
    use opentelemetry::testing::trace::TestSpan;
    use opentelemetry::trace::{
        SamplingDecision, SamplingResult, SpanKind, Status, TraceContextExt, TraceState,
    };
    use opentelemetry::{
        trace::{
            Event, Link, Span, SpanBuilder, SpanContext, SpanId, TraceFlags, TraceId, Tracer,
            TracerProvider as _,
        },
        Context, KeyValue,
    };

    #[test]
    fn tracer_in_span() {
        // Arrange
        let exporter = InMemorySpanExporterBuilder::new().build();
        let provider = TracerProvider::builder()
            .with_span_processor(SimpleSpanProcessor::new(Box::new(exporter.clone())))
            .build();

        // Act
        let tracer = provider.tracer("test_tracer");
        tracer.in_span("span_name", |cx| {
            let span = cx.span();
            assert!(span.is_recording());
            span.update_name("span_name_updated");
            span.set_attribute(KeyValue::new("attribute1", "value1"));
            span.add_event("test-event".to_string(), vec![]);
        });

        // Assert
        let exported_spans = exporter
            .get_finished_spans()
            .expect("Spans are expected to be exported.");
        assert_eq!(exported_spans.len(), 1);
        let span = &exported_spans[0];
        assert_eq!(span.name, "span_name_updated");
        assert_eq!(span.instrumentation_lib.name, "test_tracer");
        assert_eq!(span.attributes.len(), 1);
        assert_eq!(span.events.len(), 1);
        assert_eq!(span.events[0].name, "test-event");
        assert_eq!(span.span_context.trace_flags(), TraceFlags::SAMPLED);
        assert!(!span.span_context.is_remote());
        assert_eq!(span.status, Status::Unset);
    }

    #[test]
    fn tracer_start() {
        // Arrange
        let exporter = InMemorySpanExporterBuilder::new().build();
        let provider = TracerProvider::builder()
            .with_span_processor(SimpleSpanProcessor::new(Box::new(exporter.clone())))
            .build();

        // Act
        let tracer = provider.tracer("test_tracer");
        let mut span = tracer.start("span_name");
        span.set_attribute(KeyValue::new("attribute1", "value1"));
        span.add_event("test-event".to_string(), vec![]);
        span.set_status(Status::error("cancelled"));
        provider.force_flush();
        drop(span);

        // Assert
        let exported_spans = exporter
            .get_finished_spans()
            .expect("Spans are expected to be exported.");
        assert_eq!(exported_spans.len(), 1);
        let span = &exported_spans[0];
        assert_eq!(span.name, "span_name");
        assert_eq!(span.instrumentation_lib.name, "test_tracer");
        assert_eq!(span.attributes.len(), 1);
        assert_eq!(span.events.len(), 1);
        assert_eq!(span.events[0].name, "test-event");
        assert_eq!(span.span_context.trace_flags(), TraceFlags::SAMPLED);
        assert!(!span.span_context.is_remote());
        let status_expected = Status::error("cancelled");
        assert_eq!(span.status, status_expected);
    }

    #[test]
    fn tracer_span_builder() {
        // Arrange
        let exporter = InMemorySpanExporterBuilder::new().build();
        let provider = TracerProvider::builder()
            .with_span_processor(SimpleSpanProcessor::new(Box::new(exporter.clone())))
            .build();

        // Act
        let tracer = provider.tracer("test_tracer");
        let mut span = tracer
            .span_builder("span_name")
            .with_kind(SpanKind::Server)
            .start(&tracer);
        span.set_attribute(KeyValue::new("attribute1", "value1"));
        span.add_event("test-event".to_string(), vec![]);
        span.set_status(Status::Ok);
        drop(span);

        // Assert
        let exported_spans = exporter
            .get_finished_spans()
            .expect("Spans are expected to be exported.");
        assert_eq!(exported_spans.len(), 1);
        let span = &exported_spans[0];
        assert_eq!(span.name, "span_name");
        assert_eq!(span.span_kind, SpanKind::Server);
        assert_eq!(span.instrumentation_lib.name, "test_tracer");
        assert_eq!(span.attributes.len(), 1);
        assert_eq!(span.events.len(), 1);
        assert_eq!(span.events[0].name, "test-event");
        assert_eq!(span.span_context.trace_flags(), TraceFlags::SAMPLED);
        assert!(!span.span_context.is_remote());
        assert_eq!(span.status, Status::Ok);
    }

    #[test]
    fn exceed_span_links_limit() {
        // Arrange
        let exporter = InMemorySpanExporterBuilder::new().build();
        let provider = TracerProvider::builder()
            .with_span_processor(SimpleSpanProcessor::new(Box::new(exporter.clone())))
            .build();

        // Act
        let tracer = provider.tracer("test_tracer");

        let mut links = Vec::new();
        for _i in 0..(DEFAULT_MAX_LINKS_PER_SPAN * 2) {
            links.push(Link::new(
                SpanContext::new(
                    TraceId::from_u128(12),
                    SpanId::from_u64(12),
                    TraceFlags::default(),
                    false,
                    Default::default(),
                ),
                Vec::new(),
            ))
        }

        let span_builder = SpanBuilder::from_name("span_name").with_links(links);
        let mut span = tracer.build(span_builder);
        span.end();

        // Assert
        let exported_spans = exporter
            .get_finished_spans()
            .expect("Spans are expected to be exported.");
        assert_eq!(exported_spans.len(), 1);
        let span = &exported_spans[0];
        assert_eq!(span.name, "span_name");
        assert_eq!(span.links.len(), DEFAULT_MAX_LINKS_PER_SPAN as usize);
    }

    #[test]
    fn exceed_span_events_limit() {
        // Arrange
        let exporter = InMemorySpanExporterBuilder::new().build();
        let provider = TracerProvider::builder()
            .with_span_processor(SimpleSpanProcessor::new(Box::new(exporter.clone())))
            .build();

        // Act
        let tracer = provider.tracer("test_tracer");

        let mut events = Vec::new();
        for _i in 0..(DEFAULT_MAX_EVENT_PER_SPAN * 2) {
            events.push(Event::with_name("test event"))
        }

        // add events via span builder
        let span_builder = SpanBuilder::from_name("span_name").with_events(events);
        let mut span = tracer.build(span_builder);

        // add events using span api after building the span
        span.add_event("test event again, after span builder", Vec::new());
        span.add_event("test event once again, after span builder", Vec::new());
        span.end();

        // Assert
        let exported_spans = exporter
            .get_finished_spans()
            .expect("Spans are expected to be exported.");
        assert_eq!(exported_spans.len(), 1);
        let span = &exported_spans[0];
        assert_eq!(span.name, "span_name");
        assert_eq!(span.events.len(), DEFAULT_MAX_EVENT_PER_SPAN as usize);
        assert_eq!(span.events.dropped_count, DEFAULT_MAX_EVENT_PER_SPAN + 2);
    }

    #[test]
    fn trace_state_for_dropped_sampler() {
        let exporter = InMemorySpanExporterBuilder::new().build();
        let provider = TracerProvider::builder()
            .with_config(config().with_sampler(Sampler::AlwaysOff))
            .with_span_processor(SimpleSpanProcessor::new(Box::new(exporter.clone())))
            .build();

        let tracer = provider.tracer("test");
        let trace_state = TraceState::from_key_value(vec![("foo", "bar")]).unwrap();

        let parent_context = Context::new().with_span(TestSpan(SpanContext::new(
            TraceId::from_u128(10000),
            SpanId::from_u64(20),
            TraceFlags::SAMPLED,
            true,
            trace_state.clone(),
        )));

        let span = tracer.start_with_context("span", &parent_context);
        assert_eq!(
            span.span_context().trace_state().get("foo"),
            trace_state.get("foo")
        )
    }

    #[derive(Clone, Debug, Default)]
    struct TestRecordOnlySampler {}

    impl ShouldSample for TestRecordOnlySampler {
        fn should_sample(
            &self,
            parent_context: Option<&Context>,
            _trace_id: TraceId,
            _name: &str,
            _span_kind: &SpanKind,
            _attributes: &[KeyValue],
            _links: &[Link],
        ) -> SamplingResult {
            let trace_state = parent_context
                .unwrap()
                .span()
                .span_context()
                .trace_state()
                .clone();
            SamplingResult {
                decision: SamplingDecision::RecordOnly,
                attributes: vec![KeyValue::new("record_only_key", "record_only_value")],
                trace_state,
            }
        }
    }

    #[test]
    fn trace_state_for_record_only_sampler() {
        let exporter = InMemorySpanExporterBuilder::new().build();
        let provider = TracerProvider::builder()
            .with_config(config().with_sampler(TestRecordOnlySampler::default()))
            .with_span_processor(SimpleSpanProcessor::new(Box::new(exporter.clone())))
            .build();

        let tracer = provider.tracer("test");
        let trace_state = TraceState::from_key_value(vec![("foo", "bar")]).unwrap();

        let parent_context = Context::new().with_span(TestSpan(SpanContext::new(
            TraceId::from_u128(10000),
            SpanId::from_u64(20),
            TraceFlags::SAMPLED,
            true,
            trace_state.clone(),
        )));

        let span = tracer.build_with_context(
            SpanBuilder::from_name("span")
                .with_attributes(vec![KeyValue::new("extra_attr_key", "extra_attr_value")]),
            &parent_context,
        );
        assert!(!span.span_context().trace_flags().is_sampled());
        assert_eq!(
            span.exported_data().unwrap().attributes,
            vec![
                KeyValue::new("extra_attr_key", "extra_attr_value"),
                KeyValue::new("record_only_key", "record_only_value")
            ]
        );
        assert_eq!(span.span_context().trace_state().get("foo"), Some("bar"));
    }
}
