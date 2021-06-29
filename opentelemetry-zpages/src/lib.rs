//! zPages implementation for Opentelemetry
//!
//! # Overview
//! zPages are an in-process alternative to external processors.
#![warn(
    future_incompatible,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    unreachable_pub,
    unused
)]
#![allow(elided_lifetimes_in_paths)]
#![cfg_attr(docsrs, feature(doc_cfg), deny(broken_intra_doc_links))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/master/assets/logo.svg"
)]
// #![cfg_attr(test, deny(warnings))]

use trace::span_queue::SpanQueue;

mod trace;
#[allow(clippy::all, unreachable_pub, dead_code)]
#[rustfmt::skip]
mod proto;
mod transform;

pub use trace::{span_processor::ZPagesSpanProcessor, TracezMessage, TracezQuery, TracezResponse};

#[macro_use]
extern crate lazy_static;
