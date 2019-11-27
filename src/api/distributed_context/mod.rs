//! # OpenTelemetry Distributed Context API
//!
//! OpenTelemetry uses `Propagators` to serialize and deserialize `SpanContext`
//! into a binary or text format. Currently there are two types of propagators:
//!
//! - `BinaryFormat` which is used to serialize and deserialize a value into
//! a binary representation.
//! - `HTTPTextFormat` which is used to inject and extract a value as text into
//! `Carrier`s that travel in-band across process boundaries.
pub mod binary_propagator;
pub mod http_b3_propagator;
pub mod trace_context_propagator;
