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
    fn set_status(&mut self, status: api::SpanStatus);

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

    /// Mark as currently active span.
    ///
    /// This is the _synchronous_ api. If you are using futures, you
    /// need to use the async api via [`instrument`].
    ///
    /// [`instrument`]: ../futures/trait.Instrument.html#method.instrument
    fn mark_as_active(&self);

    /// Mark as no longer active.
    ///
    /// This is the _synchronous_ api. If you are using futures, you
    /// need to use the async api via [`instrument`].
    ///
    /// [`instrument`]: ../futures/trait.Instrument.html#method.instrument
    fn mark_as_inactive(&self);
}

/// `SpanKind` describes the relationship between the Span, its parents,
/// and its children in a `Trace`. `SpanKind` describes two independent
/// properties that benefit tracing systems during analysis.
///
/// The first property described by `SpanKind` reflects whether the `Span`
/// is a remote child or parent. `Span`s with a remote parent are
/// interesting because they are sources of external load. `Span`s with a
/// remote child are interesting because they reflect a non-local system
/// dependency.
///
/// The second property described by `SpanKind` reflects whether a child
/// `Span` represents a synchronous call.  When a child span is synchronous,
/// the parent is expected to wait for it to complete under ordinary
/// circumstances.  It can be useful for tracing systems to know this
/// property, since synchronous `Span`s may contribute to the overall trace
/// latency.
///
/// In order for `SpanKind` to be meaningful, callers should arrange that
/// a single `Span` does not serve more than one purpose.  For example, a
/// server-side span should not be used directly as the parent of another
/// remote span.  As a simple guideline, instrumentation should create a
/// new `Span` prior to extracting and serializing the span context for a
/// remote call.
///
/// To summarize the interpretation of these kinds:
///
/// | `SpanKind` | Synchronous | Asynchronous | Remote Incoming | Remote Outgoing |
/// |------------|-----|-----|-----|-----|
/// | `CLIENT`   | yes |     |     | yes |
/// | `SERVER`   | yes |     | yes |     |
/// | `PRODUCER` |     | yes |     | yes |
/// | `CONSUMER` |     | yes | yes |     |
/// | `INTERNAL` |     |     |     |     |
#[derive(Clone, Debug)]
pub enum SpanKind {
    /// Indicates that the span describes a synchronous request to
    /// some remote service.  This span is the parent of a remote `SERVER`
    /// span and waits for its response.
    Client,
    /// Indicates that the span covers server-side handling of a
    /// synchronous RPC or other remote request.  This span is the child of
    /// a remote `CLIENT` span that was expected to wait for a response.
    Server,
    /// Indicates that the span describes the parent of an
    /// asynchronous request.  This parent span is expected to end before
    /// the corresponding child `CONSUMER` span, possibly even before the
    /// child span starts.
    Producer,
    /// Indicates that the span describes the child of an asynchronous
    /// remote `PRODUCER` request.
    Consumer,
    /// Default value. Indicates that the span represents an
    /// internal operation within an application, as opposed to an
    /// operations with remote parents or children.
    Internal,
}

/// The `SpanStatus` interface represents the status of a finished `Span`.
/// It's composed of a canonical code in conjunction with an optional
/// descriptive message.
#[derive(Clone, Debug)]
pub enum SpanStatus {
    /// OK is returned on success.
    OK = 0,
    /// Canceled indicates the operation was canceled (typically by the caller).
    Canceled = 1,
    /// Unknown error. An example of where this error may be returned is
    /// if a Status value received from another address space belongs to
    /// an error-space that is not known in this address space. Also
    /// errors raised by APIs that do not return enough error information
    /// may be converted to this error.
    Unknown = 2,
    /// InvalidArgument indicates client specified an invalid argument.
    /// Note that this differs from FailedPrecondition. It indicates arguments
    /// that are problematic regardless of the state of the system
    /// (e.g., a malformed file name).
    InvalidArgument = 3,
    /// DeadlineExceeded means operation expired before completion.
    /// For operations that change the state of the system, this error may be
    /// returned even if the operation has completed successfully. For
    /// example, a successful response from a server could have been delayed
    /// long enough for the deadline to expire.
    DeadlineExceeded = 4,
    /// NotFound means some requested entity (e.g., file or directory) was
    /// not found.
    NotFound = 5,
    /// AlreadyExists means an attempt to create an entity failed because one
    /// already exists.
    AlreadyExists = 6,
    /// PermissionDenied indicates the caller does not have permission to
    /// execute the specified operation. It must not be used for rejections
    /// caused by exhausting some resource (use ResourceExhausted
    /// instead for those errors). It must not be
    /// used if the caller cannot be identified (use Unauthenticated
    /// instead for those errors).
    PermissionDenied = 7,
    /// ResourceExhausted indicates some resource has been exhausted, perhaps
    /// a per-user quota, or perhaps the entire file system is out of space.
    ResourceExhausted = 8,
    /// FailedPrecondition indicates operation was rejected because the
    /// system is not in a state required for the operation's execution.
    /// For example, directory to be deleted may be non-empty, an rmdir
    /// operation is applied to a non-directory, etc.
    ///
    /// A litmus test that may help a service implementor in deciding
    /// between FailedPrecondition, Aborted, and Unavailable:
    ///  (a) Use Unavailable if the client can retry just the failing call.
    ///  (b) Use Aborted if the client should retry at a higher-level
    ///      (e.g., restarting a read-modify-write sequence).
    ///  (c) Use FailedPrecondition if the client should not retry until
    ///      the system state has been explicitly fixed. E.g., if an "rmdir"
    ///      fails because the directory is non-empty, FailedPrecondition
    ///      should be returned since the client should not retry unless
    ///      they have first fixed up the directory by deleting files from it.
    ///  (d) Use FailedPrecondition if the client performs conditional
    ///      REST Get/Update/Delete on a resource and the resource on the
    ///      server does not match the condition. E.g., conflicting
    ///      read-modify-write on the same resource.
    FailedPrecondition = 9,
    /// Aborted indicates the operation was aborted, typically due to a
    /// concurrency issue like sequencer check failures, transaction aborts,
    /// etc.
    ///
    /// See litmus test above for deciding between FailedPrecondition,
    /// Aborted, and Unavailable.
    Aborted = 10,
    /// OutOfRange means operation was attempted past the valid range.
    /// E.g., seeking or reading past end of file.
    ///
    /// Unlike InvalidArgument, this error indicates a problem that may
    /// be fixed if the system state changes. For example, a 32-bit file
    /// system will generate InvalidArgument if asked to read at an
    /// offset that is not in the range [0,2^32-1], but it will generate
    /// OutOfRange if asked to read from an offset past the current
    /// file size.
    ///
    /// There is a fair bit of overlap between FailedPrecondition and
    /// OutOfRange. We recommend using OutOfRange (the more specific
    /// error) when it applies so that callers who are iterating through
    /// a space can easily look for an OutOfRange error to detect when
    /// they are done.
    OutOfRange = 11,
    /// Unimplemented indicates operation is not implemented or not
    /// supported/enabled in this service.
    Unimplemented = 12,
    /// Internal errors. Means some invariants expected by underlying
    /// system has been broken. If you see one of these errors,
    /// something is very broken.
    Internal = 13,
    /// Unavailable indicates the service is currently unavailable.
    /// This is a most likely a transient condition and may be corrected
    /// by retrying with a backoff. Note that it is not always safe to retry
    /// non-idempotent operations.
    ///
    /// See litmus test above for deciding between FailedPrecondition,
    /// Aborted, and Unavailable.
    Unavailable = 14,
    /// DataLoss indicates unrecoverable data loss or corruption.
    DataLoss = 15,
    /// Unauthenticated indicates the request does not have valid
    /// authentication credentials for the operation.
    Unauthenticated = 16,
}
