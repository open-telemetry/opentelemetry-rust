#![no_main]

use libfuzzer_sys::fuzz_target;
use opentelemetry_fuzz::context_stack_model::{run, Program};

fuzz_target!(|program: Program| run(program));
