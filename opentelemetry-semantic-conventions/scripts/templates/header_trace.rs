//! # Trace Semantic Conventions
//!
//! The [trace semantic conventions] define a set of standardized attributes to
//! be used in `Span`s.
//!
//! [trace semantic conventions]: https://github.com/open-telemetry/opentelemetry-specification/tree/master/specification/trace/semantic_conventions
//!
//! ## Usage
//!
//! ```rust
//! use opentelemetry::{global, trace::Tracer as _};
//! use opentelemetry_semantic_conventions as semcov;
//!
//! let tracer = global::tracer("my-component");
//! let _span = tracer
//!     .span_builder("span-name")
//!     .with_attributes(vec![
//!         semcov::trace::NET_PEER_IP.string("10.0.0.1"),
//!         semcov::trace::NET_PEER_PORT.i64(80),
//!     ])
//!     .start(&tracer);
//! ```
