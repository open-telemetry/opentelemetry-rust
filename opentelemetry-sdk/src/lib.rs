//! # OpenTelemetry SDK
//!
//! This SDK provides an opinionated reference implementation of
//! the OpenTelemetry API. The SDK implements the specifics of
//! deciding which data to collect through `Sampler`s, and
//! facilitates the delivery of telemetry data to storage systems
//! through `Exporter`s. These can be configured on `Tracer` and
//! `Meter` creation.
#![warn(
    future_incompatible,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    unreachable_pub,
    unused
)]
#![allow(clippy::needless_doctest_main)]
#![cfg_attr(docsrs, feature(doc_cfg), deny(rustdoc::broken_intra_doc_links))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo.svg"
)]
#![cfg_attr(test, deny(warnings))]

pub mod export;
pub mod instrumentation;
#[cfg(feature = "metrics")]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
pub mod metrics;
#[cfg(feature = "trace")]
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
pub mod propagation;
pub mod resource;
pub mod runtime;
#[cfg(feature = "testing")]
#[doc(hidden)]
pub mod testing;
#[cfg(feature = "trace")]
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
pub mod trace;
#[doc(hidden)]
pub mod util;

pub use instrumentation::InstrumentationLibrary;
pub use resource::Resource;
