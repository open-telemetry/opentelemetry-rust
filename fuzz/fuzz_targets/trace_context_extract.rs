//! Extracts generated `traceparent` and `tracestate` headers with the `TraceContextPropagator`,
//! compares the result with the oracles and checks that injecting and extracting it again roundtrips.
#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry::trace::{TraceContextExt, TraceState};
use opentelemetry_fuzz::assert_tracestate_invariants;
use opentelemetry_fuzz::generators::{
    Traceparent, TraceparentMutated, TsHeader, TsHeaderLongMembers, TsHeaderMutated,
};
use opentelemetry_fuzz::oracle::{expected_traceparent, expected_tracestate, MultiFieldExtractor};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use std::collections::HashMap;

#[derive(Debug, Arbitrary)]
enum TraceparentField {
    Missing,
    Valid(Traceparent),
    Mutated(TraceparentMutated),
}

#[derive(Debug, Arbitrary)]
enum TracestateField {
    Valid(TsHeader),
    LongMembers(TsHeaderLongMembers),
    Mutated(TsHeaderMutated),
}

#[derive(Debug, Arbitrary)]
struct Input {
    traceparent: TraceparentField,
    tracestate: Vec<TracestateField>,
}

fuzz_target!(|input: Input| {
    let extractor = MultiFieldExtractor {
        traceparent: match input.traceparent {
            TraceparentField::Missing => None,
            TraceparentField::Valid(header) => Some(header.into_string()),
            TraceparentField::Mutated(header) => Some(header.into_string()),
        },
        tracestate: input
            .tracestate
            .into_iter()
            .map(|field| match field {
                TracestateField::Valid(header) => header.into_string(),
                TracestateField::LongMembers(header) => header.into_string(),
                TracestateField::Mutated(header) => header.into_string(),
            })
            .collect(),
    };

    let propagator = TraceContextPropagator::new();
    let cx = propagator.extract(&extractor);
    let span = cx.span();
    let span_context = span.span_context();

    let Some((trace_id, span_id, trace_flags)) =
        expected_traceparent(extractor.traceparent.as_deref().unwrap_or(""))
    else {
        assert!(
            !span_context.is_valid(),
            "{extractor:?} extracted {span_context:?}"
        );
        return;
    };
    assert!(
        span_context.is_valid(),
        "{extractor:?} extracted {span_context:?}"
    );
    assert!(span_context.is_remote());
    assert_eq!(span_context.trace_id(), trace_id, "{extractor:?}");
    assert_eq!(span_context.span_id(), span_id, "{extractor:?}");
    assert_eq!(span_context.trace_flags(), trace_flags, "{extractor:?}");

    // Multiple tracestate fields are combined, an invalid tracestate is discarded.
    let expected_state = expected_tracestate(&extractor.tracestate.join(","))
        .map(|members| TraceState::from_key_value(members).expect("oracle members are valid"))
        .unwrap_or_default();
    assert_eq!(span_context.trace_state(), &expected_state, "{extractor:?}");
    assert_tracestate_invariants(span_context.trace_state());

    let mut injected = HashMap::new();
    propagator.inject_context(&cx, &mut injected);
    assert_eq!(
        injected.contains_key("tracestate"),
        expected_state != TraceState::NONE,
        "{injected:?}"
    );
    let reextracted = propagator.extract(&injected);
    assert_eq!(
        reextracted.span().span_context(),
        span_context,
        "{extractor:?} was injected as {injected:?}"
    );
});
