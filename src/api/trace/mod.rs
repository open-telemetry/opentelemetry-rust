//! # OpenTelemetry Tracing API.
//!
//! The tracing API consist of a few main traits:
//!
//! * The `Tracer` trait which describes all tracing operations.
//! * The `Span` trait with is a mutable object storing information about the
//! current operation execution.
//! * The `SpanContext` struct is the portion of a `Span` which must be
//! serialized and propagated along side of a distributed context
//!
//! ## Tracer
//!
//! The OpenTelemetry library achieves in-process context propagation of `Span`s by
//! way of the `Tracer`.
//!
//! The `Tracer` is responsible for tracking the currently active `Span`, and
//! exposes methods for creating and activating new `Span`s. The `Tracer` is
//! configured with `Propagator`s which support transferring span context across
//! process boundaries.
//!
//! `Tracer`s are generally expected to be used as singletons. Implementations
//! SHOULD provide a single global default `Tracer`.
//!
//! Some applications may require multiple `Tracer` instances, e.g. to create
//! `Span`s on behalf of other applications. Implementations MAY provide a global
//! registry of `Tracer`s for such applications.
//!
//! ## Span
//!
//! A `Span` represents a single operation within a trace. Spans can be nested to
//! form a trace tree. Each trace contains a root span, which typically describes
//! the end-to-end latency and, optionally, one or more sub-spans for its
//! sub-operations.
//!
//! `Span`s encapsulate:
//!
//! - The operation name
//! - An immutable `SpanContext` that uniquely identifies the `Span`
//! - A parent span in the form of a `SpanContext`, or None
//! - A start timestamp
//! - An end timestamp
//! - An ordered mapping of `Attribute`s
//! - A list of `Link`s to other `Span`s
//! - A list of timestamped `Event`s
//! - A `Status`.
//!
//! The `Span`'s start and end timestamps reflect the elapsed real time of the
//! operation. A `Span`'s start time SHOULD be set to the current time on span
//! creation. After the `Span` is created, it SHOULD be possible to
//! change the its name, set its `Attribute`s, and add `Link`s and `Event`s. These
//! MUST NOT be changed after the `Span`'s end time has been set.
//!
//! `Span`s are not meant to be used to propagate information within a process. To
//! prevent misuse, implementations SHOULD NOT provide access to a `Span`'s
//! attributes besides its `SpanContext`.
//!
//! Vendors may implement the `Span` interface to effect vendor-specific logic.
//! However, alternative implementations MUST NOT allow callers to create `Span`s
//! directly. All `Span`s MUST be created via a `Tracer`.
//!
//! ## SpanContext
//!
//! A `SpanContext` represents the portion of a `Span` which must be serialized and
//! propagated along side of a distributed context. `SpanContext`s are immutable.
//! `SpanContext`.
//!
//! The OpenTelemetry `SpanContext` representation conforms to the [w3c TraceContext
//! specification](https://www.w3.org/TR/trace-context/). It contains two
//! identifiers - a `TraceId` and a `SpanId` - along with a set of common
//! `TraceFlags` and system-specific `TraceState` values.
//!
//! `TraceId` A valid trace identifier is a non-zero `u128`
//!
//! `SpanId` A valid span identifier is a non-zero `u64` byte.
//!
//! `TraceFlags` contain details about the trace. Unlike Tracestate values,
//! TraceFlags are present in all traces. Currently, the only `TraceFlags` is a
//! boolean `sampled`
//! [flag](https://www.w3.org/TR/trace-context/#trace-flags).
//!
//! `Tracestate` carries system-specific configuration data, represented as a list
//! of key-value pairs. TraceState allows multiple tracing systems to participate in
//! the same trace.
//!
//! `IsValid` is a boolean flag which returns true if the SpanContext has a non-zero
//! TraceID and a non-zero SpanID.
//!
//! `IsRemote` is a boolean flag which returns true if the SpanContext was propagated
//! from a remote parent.
//!
//! Please review the W3C specification for details on the [Tracestate
//! field](https://www.w3.org/TR/trace-context/#tracestate-field).
//!
pub mod event;
pub mod futures;
pub mod link;
pub mod noop;
pub mod propagator;
pub mod provider;
pub mod sampler;
pub mod span;
pub mod span_context;
pub mod span_processor;
pub mod tracer;
