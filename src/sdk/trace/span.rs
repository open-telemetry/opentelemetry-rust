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
use crate::{api, exporter, sdk};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Single operation within a trace.
#[derive(Clone, Debug)]
pub struct Span {
    id: api::SpanId,
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
        id: api::SpanId,
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
    /// keys"](https://github.com/open-telemetry/opentelemetry-specification/tree/v0.5.0/specification/trace/semantic_conventions/README.md)
    /// which have prescribed semantic meanings.
    fn add_event_with_timestamp(
        &self,
        name: String,
        timestamp: SystemTime,
        attributes: Vec<api::KeyValue>,
    ) {
        self.with_data_mut(|data| {
            data.message_events
                .push_back(api::Event::new(name, timestamp, attributes))
        });
    }

    /// Returns the `SpanContext` for the given `Span`.
    fn span_context(&self) -> api::SpanContext {
        self.with_data(|data| data.span_context.clone())
            .unwrap_or_else(|| {
                api::SpanContext::new(api::TraceId::invalid(), api::SpanId::invalid(), 0, false)
            })
    }

    /// Returns true if this `Span` is recording information like events with the `add_event`
    /// operation, attributes using `set_attributes`, status with `set_status`, etc.
    fn is_recording(&self) -> bool {
        self.inner.data.is_some()
    }

    /// Sets a single `Attribute` where the attribute properties are passed as arguments.
    ///
    /// Note that the OpenTelemetry project documents certain ["standard
    /// attributes"](https://github.com/open-telemetry/opentelemetry-specification/tree/v0.5.0/specification/trace/semantic_conventions/README.md)
    /// that have prescribed semantic meanings.
    fn set_attribute(&self, attribute: api::KeyValue) {
        self.with_data_mut(|data| {
            data.attributes.insert(attribute);
        });
    }

    /// Sets the status of the `Span`. If used, this will override the default `Span`
    /// status, which is `OK`.
    fn set_status(&self, code: api::StatusCode, message: String) {
        self.with_data_mut(|data| {
            data.status_code = code;
            data.status_message = message
        });
    }

    /// Updates the `Span`'s name.
    fn update_name(&self, new_name: String) {
        self.with_data_mut(|data| {
            data.name = new_name;
        });
    }

    /// Finishes the span.
    fn end(&self) {
        self.with_data_mut(|data| {
            data.end_time = SystemTime::now();
        });
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
