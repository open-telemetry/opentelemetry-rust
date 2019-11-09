//! # Trace Span SDK
//!
//! `Span`s represent a single operation within a trace. `Span`s can be nested to form a trace
//! tree. Each trace contains a root span, which typically describes the end-to-end latency and,
//! optionally, one or more sub-spans for its sub-operations.
//!
//! The `Span`'s start and end timestamps reflect the elapsed real time of the operation. A `Span`'s
//! start time is set to the current time on span creation. After the `Span` is created, it
//! is possible to change its name, set its `Attributes`, and add `Links` and `Events`.
//! These cannot be changed after the `Span`'s end time has been set.
use crate::api;
use crate::exporter::trace::jaeger;
use std::time::SystemTime;

/// Single operation within a trace.
#[derive(Debug)]
pub struct Span(jaeger::Span);

impl Span {
    /// Creates a new span, used internally by `sdk::Tracer`.
    pub(crate) fn new(span: jaeger::Span) -> Self {
        Span(span)
    }
}

impl api::Span for Span {
    /// Records events at a specific time in the context of a given `Span`.
    ///
    /// Note that the OpenTelemetry project documents certain ["standard event names and
    /// keys"](https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/data-semantic-conventions.md)
    /// which have prescribed semantic meanings.
    fn add_event_with_timestamp(&mut self, message: String, _timestamp: SystemTime) {
        self.0.log(|log| {
            log.std().event(message);
        });
    }

    /// Adds links to a given `Span`.
    ///
    /// Linked Spans can be from the same or a different trace.
    fn add_link(&mut self, _link: api::SpanContext) {
        // Ignored for now
    }

    /// Returns the `SpanContext` for the given `Span`.
    fn get_context(&self) -> api::SpanContext {
        match self.0.context() {
            Some(context) => {
                let state = context.state();
                let trace_id = u128::from_str_radix(&state.trace_id().to_string(), 16).unwrap();
                let trace_flags = if state.is_sampled() { 1 } else { 0 };
                let is_remote = false; // TODO determine remote state

                api::SpanContext::new(trace_id, state.span_id(), trace_flags, is_remote)
            }
            None => api::SpanContext::new(rand::random(), 0, 0, false),
        }
    }

    /// Returns true if this `Span` is recording information like events with the `add_event`
    /// operation, attributes using `set_attributes`, status with `set_status`, etc.
    fn is_recording(&self) -> bool {
        true
    }

    /// Sets a single `Attribute` where the attribute properties are passed as arguments.
    ///
    /// Note that the OpenTelemetry project documents certain ["standard
    /// attributes"](https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/data-semantic-conventions.md)
    /// that have prescribed semantic meanings.
    fn set_attribute(&mut self, attribute: api::KeyValue) {
        let api::KeyValue { key, value } = attribute;
        self.0.set_tag(|| jaeger::Tag::new(key, value.to_string()))
    }

    /// Sets the status of the `Span`. If used, this will override the default `Span`
    /// status, which is `OK`.
    fn set_status(&mut self, _status: String) {
        // Ignored for now
    }

    /// Updates the `Span`'s name.
    fn update_name(&mut self, new_name: String) {
        self.0.set_operation_name(|| new_name)
    }

    /// Finishes the span.
    fn end(&mut self) {
        self.0.set_finish_time(SystemTime::now)
    }
}
