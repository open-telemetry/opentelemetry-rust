//! # Trace Semantic Conventions
//!
//! The [trace semantic conventions] define a set of standardized attributes to
//! be used in `Span`s.
//!
//! [trace semantic conventions]: https://github.com/open-telemetry/semantic-conventions/tree/main/model/trace
//!
//! ## Usage
//!
//! [`tracing`]:
//! 
//! ```
//! use opentelemetry_semantic_conventions as semconv;
//! use tracing::span;
//!
//! let span = span!(
//!     LEVEL::INFO,
//!     "handle_request",
//!     { semconv::trace::NET_PEER_NAME = "example.org" },
//!     { semconv::trace::NET_PEER_PORT = 80 }
//! );
//! ```
//! 
//! OpenTelemetry SDK:
//! 
//! ```
//! use opentelemetry::KeyValue;
//! use opentelemetry::{global, trace::Tracer as _};
//! use opentelemetry_semantic_conventions as semconv;
//!
//! let tracer = global::tracer("my-component");
//! let _span = tracer
//!     .span_builder("span-name")
//!     .with_attributes(vec![
//!         KeyValue::new(semconv::trace::NET_PEER_NAME, "example.org"),
//!         KeyValue::new(semconv::trace::NET_PEER_PORT, 80i64),
//!     ])
//!     .start(&tracer);
//! ```
