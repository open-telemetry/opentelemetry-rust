//! # OpenTelemetry Trace SDK
//!
//! The tracing SDK consist of a few main structs:
//!
//! * The `Tracer` struct which performs all tracing operations.
//! * The `Span` struct with is a mutable object storing information about the
//! current operation execution.
//! * The `Provider` struct which configures and produces `Tracer`s.
pub mod config;
pub mod provider;
pub mod span;
pub mod tracer;
