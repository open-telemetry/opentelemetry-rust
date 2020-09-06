//! # OpenTelemetry Trace SDK
//!
//! The tracing SDK consist of a few main structs:
//!
//! * The `Tracer` struct which performs all tracing operations.
//! * The `Span` struct with is a mutable object storing information about the
//! current operation execution.
//! * The `TracerProvider` struct which configures and produces `Tracer`s.
pub mod config;
pub mod evicted_hash_map;
pub mod evicted_queue;
pub mod id_generator;
pub mod provider;
pub mod sampler;
pub mod span;
pub mod span_processor;
pub mod tracer;

pub use config::config;
