//! # OpenTelemetry Semantic Conventions
//!
//! OpenTelemetry semantic conventions are agreed standardized naming patterns
//! for OpenTelemetry things. This crate aims to be the centralized place to
//! interact with these conventions.
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]
#![cfg_attr(test, deny(warnings))]

pub mod resource;
pub mod trace;
