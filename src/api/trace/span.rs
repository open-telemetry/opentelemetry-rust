//! # OpenTelemetry Span interface
//!
//! A `Span` represents a single operation within a trace. `Span`s can be nested to form a trace
//! tree. Each trace contains a root span, which typically describes the end-to-end latency and,
//! optionally, one or more sub-spans for its sub-operations.
//!
//! The `Span`'s start and end timestamps reflect the elapsed real time of the operation. A `Span`'s
//! start time SHOULD be set to the current time on span creation. After the `Span` is created, it
//! SHOULD be possible to change its name, set its `Attributes`, and add `Links` and `Events`.
//! These MUST NOT be changed after the `Span`'s end time has been set.
//!
//! `Spans` are not meant to be used to propagate information within a process. To prevent misuse,
//! implementations SHOULD NOT provide access to a `Span`'s attributes besides its `SpanContext`.
//!
//! Vendors may implement the `Span` interface to effect vendor-specific logic. However, alternative
//! implementations MUST NOT allow callers to create Spans directly. All `Span`s MUST be created
//! via a Tracer.
use crate::api;
use std::time::SystemTime;

/// Interface for a single operation within a trace.
pub trait Span: Send + Sync + std::fmt::Debug {
    /// An API to record events in the context of a given `Span`.
    ///
    /// Events have a time associated with the moment when they are
    /// added to the `Span`.
    ///
    /// Events SHOULD preserve the order in which they're set. This will typically match
    /// the ordering of the events' timestamps.
    ///
    /// Note that the OpenTelemetry project documents certain ["standard event names and
    /// keys"](https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/data-semantic-conventions.md)
    /// which have prescribed semantic meanings.
    fn add_event(&mut self, message: String) {
        self.add_event_with_timestamp(message, SystemTime::now())
    }

    /// An API to record events at a specific time in the context of a given `Span`.
    ///
    /// Events SHOULD preserve the order in which they're set. This will typically match
    /// the ordering of the events' timestamps.
    ///
    /// Note that the OpenTelemetry project documents certain ["standard event names and
    /// keys"](https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/data-semantic-conventions.md)
    /// which have prescribed semantic meanings.
    fn add_event_with_timestamp(&mut self, message: String, timestamp: SystemTime);

    /// Returns the `SpanContext` for the given `Span`. The returned value may be used even after
    /// the `Span is finished. The returned value MUST be the same for the entire `Span` lifetime.
    fn get_context(&self) -> api::SpanContext;

    /// Returns true if this `Span` is recording information like events with the `add_event`
    /// operation, attributes using `set_attributes`, status with `set_status`, etc.
    ///
    /// This flag SHOULD be used to avoid expensive computations of a `Span` attributes or events in
    /// case when a `Span` is definitely not recorded. Note that any child span's recording is
    /// determined independently from the value of this flag (typically based on the sampled flag of
    /// a `TraceFlag` on `SpanContext`).
    ///
    /// This flag may be true despite the entire trace being sampled out. This allows to record and
    /// process information about the individual Span without sending it to the backend. An example
    /// of this scenario may be recording and processing of all incoming requests for the processing
    /// and building of SLA/SLO latency charts while sending only a subset - sampled spans - to the
    /// backend. See also the sampling section of SDK design.
    ///
    /// Users of the API should only access the `is_recording` property when instrumenting code and
    /// never access `SampledFlag` unless used in context propagators.
    fn is_recording(&self) -> bool;

    /// An API to set a single `Attribute` where the attribute properties are passed
    /// as arguments. To avoid extra allocations some implementations may offer a separate API for
    /// each of the possible value types.
    ///
    /// An `Attribute` is defined as a `KeyValue` pair.
    ///
    /// Attributes SHOULD preserve the order in which they're set. Setting an attribute
    /// with the same key as an existing attribute SHOULD overwrite the existing
    /// attribute's value.
    ///
    /// Note that the OpenTelemetry project documents certain ["standard
    /// attributes"](https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/data-semantic-conventions.md)
    /// that have prescribed semantic meanings.
    fn set_attribute(&mut self, attribute: api::KeyValue);

    /// Sets the status of the `Span`. If used, this will override the default `Span`
    /// status, which is `OK`.
    ///
    /// Only the value of the last call will be recorded, and implementations are free
    /// to ignore previous calls.
    fn set_status(&mut self, status: String);

    /// Updates the `Span`'s name. After this update, any sampling behavior based on the
    /// name will depend on the implementation.
    ///
    /// It is highly discouraged to update the name of a `Span` after its creation.
    /// `Span` name is often used to group, filter and identify the logical groups of
    /// spans. Often, filtering logic will be implemented before the `Span` creation
    /// for performance reasons, and the name update may interfere with this logic.
    ///
    /// The method name is called `update_name` to differentiate this method from the
    /// regular property. It emphasizes that this operation signifies a
    /// major change for a `Span` and may lead to re-calculation of sampling or
    /// filtering decisions made previously depending on the implementation.
    fn update_name(&mut self, new_name: String);

    /// Finishes the `Span`.
    ///
    /// Implementations MUST ignore all subsequent calls to `end` (there might be
    /// exceptions when the tracer is streaming events and has no mutable state
    /// associated with the Span).
    ///
    /// Calls to `end` a Span MUST not have any effects on child `Span`s as they may
    /// still be running and can be ended later.
    ///
    ///This API MUST be non-blocking.
    fn end(&mut self);

    /// Used by global tracer to downcast to specific span type.
    fn as_any(&self) -> &dyn std::any::Any;
}
