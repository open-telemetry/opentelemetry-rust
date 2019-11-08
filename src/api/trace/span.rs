use crate::api;
use std::time::SystemTime;

// A `Span` represents a single operation within a trace. `Span`s can be nested to form a trace
// tree. Each trace contains a root span, which typically describes the end-to-end latency and,
// optionally, one or more sub-spans for its sub-operations.
//
// The `Span`'s start and end timestamps reflect the elapsed real time of the operation. A `Span`'s
// start time SHOULD be set to the current time on span creation. After the `Span` is created, it
// SHOULD be possible to change its name, set its `Attributes`, and add `Links` and `Events`.
// These MUST NOT be changed after the `Span`'s end time has been set.
//
// `Spans` are not meant to be used to propagate information within a process. To prevent misuse,
// implementations SHOULD NOT provide access to a `Span`'s attributes besides its `SpanContext`.
//
// Vendors may implement the `Span` interface to effect vendor-specific logic. However, alternative
// implementations MUST NOT allow callers to create Spans directly. All `Span`s MUST be created
// via a Tracer.
pub trait Span: std::fmt::Debug {
    fn id(&self) -> u64;
    fn parent(&self) -> Option<u64>;
    //    fn children(&'a self) -> Self::Children;
    //    fn follows_from(&'a self) -> Self::Follows;
    fn add_event(&mut self, message: String) {
        self.add_event_with_timestamp(message, SystemTime::now())
    }
    fn add_event_with_timestamp(&mut self, message: String, timestamp: SystemTime);

    // Returns the `SpanContext` for the given `Span`. The returned value may be used even after
    // the `Span is finished. The returned value MUST be the same for the entire `Span` lifetime.
    // This MAY be called GetContext.
    fn get_context(&self) -> api::SpanContext;

    // Returns true if this `Span` is recording information like events with the `add_event`
    // operation, attributes using `set_attributes`, status with `set_status`, etc.
    //
    // This flag SHOULD be used to avoid expensive computations of a `Span` attributes or events in
    // case when a `Span` is definitely not recorded. Note that any child span's recording is
    // determined independently from the value of this flag (typically based on the sampled flag of
    // a `TraceFlag` on `SpanContext`).
    //
    // This flag may be true despite the entire trace being sampled out. This allows to record and
    // process information about the individual Span without sending it to the backend. An example
    // of this scenario may be recording and processing of all incoming requests for the processing
    // and building of SLA/SLO latency charts while sending only a subset - sampled spans - to the
    // backend. See also the sampling section of SDK design.
    //
    // Users of the API should only access the IsRecording property when instrumenting code and
    // never access `SampledFlag` unless used in context propagators.
    fn is_recording(&self) -> bool;

    fn set_attribute(&mut self, attribute: crate::KeyValue);
    fn end(&mut self);
}
