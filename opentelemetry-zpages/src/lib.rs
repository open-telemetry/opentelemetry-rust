//! ZPages implementation for Opentelemetry
//!
//! # Overview
//! zPages are an in-process alternative to external exporters. When included,
//! they collect and aggregate tracing and metrics information in the
//! background; this data is served on web pages or APIs when requested.
//!
//! Currently only tracez components are available. And some of those are still
//! work in progress. Known limitation includes
//!  - The sampled running span doesn't reflect the changes made to the span.
//!  - The API only returns the json response.
//!  - Users have to build their own http server from the components provided.
//!
//! # Get start
//! The first step is to initiate the [`ZPagesSpanProcessor`] and install it in [`TracerProvider`].
//!
//! ```no_run
//! # use opentelemetry_zpages::tracez;
//! # use opentelemetry::{global, trace::Tracer};
//! # use opentelemetry_sdk::{runtime::Tokio, trace::TracerProvider};
//! # use std::sync::Arc;
//!
//! # fn main() {
//!     let (processor, querier) = tracez(5, Tokio);
//!     let provider = TracerProvider::builder()
//!         .with_span_processor(processor)
//!         .build();
//!     global::set_tracer_provider(provider);
//! # }
//! ```
//!
//! Once the [`ZPagesSpanProcessor`] installed. It will record spans when they
//! start or end.
//!
//! Users can then use the [`TracezQuerier`] to query the aggregated span information.
//!
//! A detailed example can also be founded [here].
//!
//!
//! [`ZPagesSpanProcessor`]: trace::span_processor::ZPagesSpanProcessor
//! [`TracerProvider`]: opentelemetry_sdk::trace::TracerProvider
//! [here]: https://github.com/open-telemetry/opentelemetry-rust/tree/main/examples/zpages
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
#![cfg_attr(
    docsrs,
    feature(doc_cfg, doc_auto_cfg),
    deny(rustdoc::broken_intra_doc_links)
)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/master/assets/logo.svg"
)]
#![cfg_attr(test, deny(warnings))]

use trace::span_queue::SpanQueue;

mod trace;

pub use trace::{
    span_processor::ZPagesSpanProcessor, tracez, TracezError, TracezQuerier, TracezResponse,
};
