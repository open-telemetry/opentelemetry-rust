//! # OpenTelemetry Contrib
//!
//! This is a library for extensions that are not part of the core API, but still may be useful for
//! some users.
//!
//! Typically, those include vendor specific propagators.
//!
//! ## Crate Feature Flags
//!
//! The following crate feature flags are available:
//!
//! * `binary-propagator`: Adds Experimental binary propagator to propagate trace context using binary format.
//! * `base64-format`: Enables base64 format support for binary propagators.
//! * `datadog`: Adds a Datadog trace exporter.
//! * `reqwest-blocking-client`: Export spans using the reqwest blocking http
//!   client.
//! * `reqwest-client`: Export spans using the reqwest non-blocking http client.
//! * `surf-client`: Export spans using the surf non-blocking http client.
#![warn(
    future_incompatible,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    unreachable_pub,
    unused
)]
#![cfg_attr(docsrs, feature(doc_cfg), deny(broken_intra_doc_links))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/master/assets/logo.svg"
)]
#![cfg_attr(test, deny(warnings))]

pub mod trace;
