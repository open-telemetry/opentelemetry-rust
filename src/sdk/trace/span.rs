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
use std::any::Any;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Single operation within a trace.
#[derive(Clone, Debug)]
pub struct Span {
    id: u64,
    data: Arc<Mutex<jaeger::Span>>,
}

impl Span {
    pub(crate) fn new(id: u64, inner: jaeger::Span) -> Self {
        Span {
            id,
            data: Arc::new(Mutex::new(inner)),
        }
    }
    pub(crate) fn id(&self) -> u64 {
        self.id
    }
}

impl api::Span for Span {
    /// Records events at a specific time in the context of a given `Span`.
    ///
    /// Note that the OpenTelemetry project documents certain ["standard event names and
    /// keys"](https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/data-semantic-conventions.md)
    /// which have prescribed semantic meanings.
    fn add_event_with_timestamp(&mut self, message: String, _timestamp: SystemTime) {
        let _ = self.data.try_lock().map(|mut span_data| {
            span_data.log(|log| {
                log.std().event(message);
            });
        });
    }

    /// Returns the `SpanContext` for the given `Span`.
    fn get_context(&self) -> api::SpanContext {
        self.data
            .try_lock()
            .ok()
            .and_then(|span_data| {
                span_data.context().map(|context| {
                    let state = context.state();
                    let trace_id = u128::from_str_radix(&state.trace_id().to_string(), 16).unwrap();
                    let trace_flags = if state.is_sampled() { 1 } else { 0 };
                    let is_remote = false; // TODO determine remote state

                    api::SpanContext::new(trace_id, state.span_id(), trace_flags, is_remote)
                })
            })
            .unwrap_or_else(|| api::SpanContext::new(0, 0, 0, false))
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
        let _ = self.data.try_lock().map(|mut span_data| {
            let api::KeyValue { key, value } = attribute;
            span_data.set_tag(|| jaeger::Tag::new(key, value.to_string()));
        });
    }

    /// Sets the status of the `Span`. If used, this will override the default `Span`
    /// status, which is `OK`.
    fn set_status(&mut self, _status: String) {
        // Ignored for now
    }

    /// Updates the `Span`'s name.
    fn update_name(&mut self, new_name: String) {
        let _ = self
            .data
            .try_lock()
            .map(|mut span_data| span_data.set_operation_name(|| new_name));
    }

    /// Finishes the span.
    fn end(&mut self) {
        let _ = self.data.try_lock().map(|mut span_data| {
            span_data.set_finish_time(SystemTime::now);
        });
    }

    /// Returns self as any
    fn as_any(&self) -> &dyn Any {
        self
    }
}
