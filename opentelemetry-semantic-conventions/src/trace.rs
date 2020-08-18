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
//! use opentelemetry::api::Tracer;
//! use opentelemetry::global;
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

use opentelemetry::api::Key;

/// Transport protocol used.
pub const NET_TRANSPORT: Key = Key::from_static_str("net.transport");

/// Remote address of the peer (dotted decimal for IPv4 or [RFC5952] for IPv6)
///
/// [RFC5952]: https://tools.ietf.org/html/rfc5952
pub const NET_PEER_IP: Key = Key::from_static_str("net.peer.ip");

/// Remote port number as an integer. E.g., `80`.
pub const NET_PEER_PORT: Key = Key::from_static_str("net.peer.port");
