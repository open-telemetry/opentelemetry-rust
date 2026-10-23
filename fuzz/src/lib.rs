//! Shared helpers for the fuzz targets in `fuzz_targets/`, see the README.

pub mod generators;
pub mod oracle;

use opentelemetry::trace::TraceState;
use oracle::{expected_tracestate, is_valid_key, is_valid_value, MAX_LIST_MEMBERS};
use std::str::FromStr;

/// Parses `header` with [`TraceState::from_str`] and checks the result against the grammar oracle
/// and the [`TraceState`] invariants.
pub fn check_tracestate_parse(header: &str) {
    let parsed = TraceState::from_str(header);
    let expected = expected_tracestate(header);
    match (&parsed, &expected) {
        (Ok(trace_state), Some(expected)) => {
            let members: Vec<(String, String)> = trace_state
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value.to_owned()))
                .collect();
            assert_eq!(&members, expected, "header: {header:?}");
            assert_tracestate_invariants(trace_state);
            assert_header_roundtrip(trace_state);
        }
        (Err(_), None) => {}
        _ => panic!("header: {header:?}\nparsed: {parsed:?}\nexpected: {expected:?}"),
    }
}

/// Asserts that `trace_state` holds at most 32 list-members with valid and unique keys and valid
/// values.
pub fn assert_tracestate_invariants(trace_state: &TraceState) {
    let members: Vec<(&str, &str)> = trace_state.into_iter().collect();
    assert!(
        members.len() <= MAX_LIST_MEMBERS,
        "{} list members in {trace_state:?}",
        members.len()
    );
    for (i, (key, value)) in members.iter().enumerate() {
        assert!(is_valid_key(key), "invalid key {key:?} in {trace_state:?}");
        assert!(
            is_valid_value(value),
            "invalid value {value:?} in {trace_state:?}"
        );
        assert!(
            members[..i].iter().all(|(existing, _)| existing != key),
            "duplicate key {key:?} in {trace_state:?}"
        );
    }
}

/// Asserts that the header serialized from `trace_state` parses back into the same `TraceState`.
pub fn assert_header_roundtrip(trace_state: &TraceState) {
    let header = trace_state.header();
    match TraceState::from_str(&header) {
        Ok(reparsed) => assert_eq!(&reparsed, trace_state, "header: {header:?}"),
        Err(err) => panic!("{trace_state:?} serialized to {header:?}, which fails to parse: {err}"),
    }
}

/// Deterministic pseudo-random fuzzer inputs for unit tests.
#[cfg(test)]
pub(crate) fn test_inputs(count: usize) -> impl Iterator<Item = Vec<u8>> {
    use rand::{rngs::StdRng, Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(0);
    (0..count).map(move |_| {
        let mut data = vec![0; rng.random_range(0..4096)];
        rng.fill(data.as_mut_slice());
        data
    })
}
