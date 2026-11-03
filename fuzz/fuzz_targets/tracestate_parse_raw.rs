//! Parses arbitrary UTF-8 strings as `tracestate` headers.
#![no_main]

use libfuzzer_sys::fuzz_target;
use opentelemetry_fuzz::check_tracestate_parse;

fuzz_target!(|header: &str| {
    check_tracestate_parse(header);
});
