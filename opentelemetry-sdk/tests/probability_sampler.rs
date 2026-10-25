#![cfg(feature = "trace")]

use opentelemetry::{
    trace::{
        Span, SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId, TraceState, Tracer,
        TracerProvider,
    },
    Context,
};
use opentelemetry_sdk::trace::{
    IdGenerator, ParentBasedSampler, ProbabilitySampler, Sampler, SdkTracerProvider,
};

#[derive(Debug)]
struct FixedIdGenerator(TraceId);

impl IdGenerator for FixedIdGenerator {
    fn new_trace_id(&self) -> TraceId {
        self.0
    }
    fn new_span_id(&self) -> SpanId {
        SpanId::from(1)
    }
    fn is_random(&self) -> bool {
        true
    }
}

#[test]
fn parent_based_root_threshold_propagates_to_child_span_context() {
    for (randomness, sampled) in [(0x80000000000000_u128, true), (0x7fffffffffffff, false)] {
        let provider = SdkTracerProvider::builder()
            .with_id_generator(FixedIdGenerator(TraceId::from(randomness)))
            .with_sampler(ParentBasedSampler::new(ProbabilitySampler::new(0.5)))
            .build();
        let tracer = provider.tracer("probability-test");
        let root = tracer.start_with_context("root", &Context::new());
        assert_eq!(root.span_context().is_sampled(), sampled);
        assert_eq!(
            root.span_context().trace_state().get("ot"),
            sampled.then_some("th:8")
        );
        let parent = root.span_context().clone();
        let cx = Context::new().with_span(root);
        let child = tracer.start_with_context("child", &cx);
        assert_eq!(child.span_context().is_sampled(), sampled);
        assert_eq!(child.span_context().trace_id(), parent.trace_id());
        assert_eq!(child.span_context().trace_state(), parent.trace_state());
        assert!(child.span_context().trace_flags().is_random());

        let remote_parent = Context::new().with_remote_span_context(SpanContext::new(
            parent.trace_id(),
            parent.span_id(),
            TraceFlags::RANDOM.with_sampled(true),
            true,
            parent.trace_state().clone(),
        ));
        let remote_child = tracer.start_with_context("remote-child", &remote_parent);
        assert!(remote_child.span_context().is_sampled());
        assert_eq!(
            remote_child.span_context().trace_state(),
            parent.trace_state()
        );
    }
}

#[test]
fn parent_based_overrides_propagate_probability_sampler_tracestate() {
    let probability = ProbabilitySampler::new(0.5);
    let provider = SdkTracerProvider::builder()
        .with_sampler(
            ParentBasedSampler::new(probability.clone())
                .with_remote_parent_sampled(probability.clone())
                .with_remote_parent_not_sampled(probability.clone())
                .with_local_parent_sampled(probability.clone())
                .with_local_parent_not_sampled(probability),
        )
        .build();
    let tracer = provider.tracer("probability-test");
    for remote in [true, false] {
        for parent_sampled in [true, false] {
            for (rv, sampled) in [("80000000000000", true), ("7fffffffffffff", false)] {
                let ot = format!("rv:{rv};th:0;foo:bar");
                let state =
                    TraceState::from_key_value([("ot", ot.as_str()), ("vendor", "value")]).unwrap();
                let cx = Context::new().with_remote_span_context(SpanContext::new(
                    TraceId::from(1),
                    SpanId::from(2),
                    TraceFlags::RANDOM.with_sampled(parent_sampled),
                    remote,
                    state,
                ));
                let span = tracer.start_with_context("child", &cx);
                assert_eq!(span.span_context().is_sampled(), sampled);
                assert_eq!(
                    span.span_context().trace_state().get("vendor"),
                    Some("value")
                );
                let expected = if sampled {
                    format!("th:8;rv:{rv};foo:bar")
                } else {
                    ot
                };
                assert_eq!(
                    span.span_context().trace_state().get("ot"),
                    Some(expected.as_str())
                );
            }
        }
    }
}

#[test]
fn legacy_parent_based_discards_delegate_tracestate() {
    let provider = SdkTracerProvider::builder()
        .with_id_generator(FixedIdGenerator(TraceId::from(0x80000000000000_u128)))
        .with_sampler(Sampler::ParentBased(Box::new(ProbabilitySampler::new(0.5))))
        .build();
    let span = provider
        .tracer("probability-test")
        .start_with_context("root", &Context::new());
    assert!(span.span_context().is_sampled());
    assert_eq!(span.span_context().trace_state().get("ot"), None);
}
