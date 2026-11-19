//! Parses `tracestate` headers shaped like the grammar and compares the result with the oracle.
#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use opentelemetry_fuzz::check_tracestate_parse;
use opentelemetry_fuzz::generators::{TsHeader, TsHeaderLongMembers, TsHeaderMutated};

#[derive(Debug, Arbitrary)]
enum Input {
    Valid(TsHeader),
    LongMembers(TsHeaderLongMembers),
    Mutated(TsHeaderMutated),
}

fuzz_target!(|input: Input| {
    let header = match &input {
        Input::Valid(header) => header.as_str(),
        Input::LongMembers(header) => header.as_str(),
        Input::Mutated(header) => header.as_str(),
    };
    check_tracestate_parse(header);
});
