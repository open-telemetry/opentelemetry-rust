//! # Span
//!
//! `Span`s represent a single operation within a trace. `Span`s can be nested to form a trace
//! tree. Each trace contains a root span, which typically describes the end-to-end latency and,
//! optionally, one or more sub-spans for its sub-operations.
//!
//! The `Span`'s start and end timestamps reflect the elapsed real time of the operation. A `Span`'s
//! start time is set to the current time on span creation. After the `Span` is created, it
//! is possible to change its name, set its `Attributes`, and add `Links` and `Events`.
//! These cannot be changed after the `Span`'s end time has been set.
use crate::api::Tracer;
use crate::{api, exporter, sdk};
use std::any::Any;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Single operation within a trace.
#[derive(Clone, Debug)]
pub struct Span {
    id: u64,
    inner: Arc<SpanInner>,
}

/// Inner data, processed and exported on drop
#[derive(Debug)]
struct SpanInner {
    data: Option<Mutex<exporter::trace::SpanData>>,
    tracer: sdk::Tracer,
}

impl Span {
    pub(crate) fn new(
        id: u64,
        data: Option<exporter::trace::SpanData>,
        tracer: sdk::Tracer,
    ) -> Self {
        Span {
            id,
            inner: Arc::new(SpanInner {
                data: data.map(Mutex::new),
                tracer,
            }),
        }
    }

    /// Return span id
    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    /// Operate on reference to span inner
    fn with_data<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&exporter::trace::SpanData) -> T,
    {
        self.inner
            .data
            .as_ref()
            .and_then(|inner| inner.lock().ok().map(|span_data| f(&span_data)))
    }

    /// Operate on mutable reference to span inner
    fn with_data_mut<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut exporter::trace::SpanData) -> T,
    {
        self.inner
            .data
            .as_ref()
            .and_then(|inner| inner.lock().ok().map(|mut span_data| f(&mut span_data)))
    }
}

impl api::Span for Span {
    /// Records events at a specific time in the context of a given `Span`.
    ///
    /// Note that the OpenTelemetry project documents certain ["standard event names and
    /// keys"](https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/data-semantic-conventions.md)
    /// which have prescribed semantic meanings.
    fn add_event_with_timestamp(&mut self, message: String, timestamp: SystemTime) {
        self.with_data_mut(|data| {
            data.message_events
                .push_front(api::Event { message, timestamp })
        });
    }

    /// Returns the `SpanContext` for the given `Span`.
    fn get_context(&self) -> api::SpanContext {
        self.with_data(|data| data.context.clone())
            .unwrap_or_else(|| api::SpanContext::new(0, 0, 0, false))
    }

    /// Returns true if this `Span` is recording information like events with the `add_event`
    /// operation, attributes using `set_attributes`, status with `set_status`, etc.
    fn is_recording(&self) -> bool {
        self.inner.data.is_some()
    }

    /// Sets a single `Attribute` where the attribute properties are passed as arguments.
    ///
    /// Note that the OpenTelemetry project documents certain ["standard
    /// attributes"](https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/data-semantic-conventions.md)
    /// that have prescribed semantic meanings.
    fn set_attribute(&mut self, attribute: api::KeyValue) {
        self.with_data_mut(|data| {
            data.attributes.push_front(attribute);
        });
    }

    /// Sets the status of the `Span`. If used, this will override the default `Span`
    /// status, which is `OK`.
    fn set_status(&mut self, status: api::SpanStatus) {
        self.with_data_mut(|data| {
            data.status = status;
        });
    }

    /// Updates the `Span`'s name.
    fn update_name(&mut self, new_name: String) {
        self.with_data_mut(|data| {
            data.name = new_name;
        });
    }

    /// Finishes the span.
    fn end(&mut self) {
        self.with_data_mut(|data| {
            data.end_time = SystemTime::now();
        });
    }

    /// Returns self as any
    fn as_any(&self) -> &dyn Any {
        self
    }

    /// Mark span as active
    fn mark_as_active(&self) {
        self.inner.tracer.mark_span_as_active(&self);
    }

    /// Mark span as inactive
    fn mark_as_inactive(&self) {
        self.inner.tracer.mark_span_as_inactive(self.id);
    }
}

impl Drop for SpanInner {
    /// Report span on inner drop
    fn drop(&mut self) {
        if let Some(data) = self.data.take() {
            if let Ok(mut inner) = data.lock() {
                if inner.end_time == inner.start_time {
                    inner.end_time = SystemTime::now();
                }
                let exportable_span = Arc::new(inner.clone());
                for processor in self.tracer.provider().span_processors() {
                    processor.on_end(exportable_span.clone())
                }
            }
        }
    }
}
