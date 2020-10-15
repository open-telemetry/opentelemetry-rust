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
use crate::api::trace::TraceState;
use crate::{api, exporter, sdk};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Single operation within a trace.
#[derive(Clone, Debug)]
pub struct Span {
    id: api::trace::SpanId,
    inner: Arc<SpanInner>,
}

/// Inner data, processed and exported on drop
#[derive(Debug)]
struct SpanInner {
    data: Option<Mutex<Option<exporter::trace::SpanData>>>,
    tracer: sdk::trace::Tracer,
}

impl Span {
    pub(crate) fn new(
        id: api::trace::SpanId,
        data: Option<exporter::trace::SpanData>,
        tracer: sdk::trace::Tracer,
    ) -> Self {
        Span {
            id,
            inner: Arc::new(SpanInner {
                data: data.map(|data| Mutex::new(Some(data))),
                tracer,
            }),
        }
    }

    /// Operate on reference to span inner
    fn with_data<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&exporter::trace::SpanData) -> T,
    {
        self.inner.data.as_ref().and_then(|inner| {
            inner
                .lock()
                .ok()
                .and_then(|span_data| span_data.as_ref().map(f))
        })
    }

    /// Operate on mutable reference to span inner
    fn with_data_mut<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut exporter::trace::SpanData) -> T,
    {
        self.inner.data.as_ref().and_then(|inner| {
            inner
                .lock()
                .ok()
                .and_then(|mut span_data| span_data.as_mut().map(f))
        })
    }
}

impl api::trace::Span for Span {
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
                .push_back(api::trace::Event::new(name, timestamp, attributes))
        });
    }

    /// Returns the `SpanReference` for the given `Span`.
    fn span_context(&self) -> api::trace::SpanReference {
        self.with_data(|data| data.span_context.clone())
            .unwrap_or_else(|| {
                api::trace::SpanReference::new(
                    api::trace::TraceId::invalid(),
                    api::trace::SpanId::invalid(),
                    0,
                    false,
                    TraceState::default(),
                )
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
    fn set_status(&self, code: api::trace::StatusCode, message: String) {
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

    /// Finishes the span with given timestamp.
    fn end_with_timestamp(&self, timestamp: SystemTime) {
        self.with_data_mut(|data| {
            data.end_time = timestamp;
        });
    }
}

impl Drop for SpanInner {
    /// Report span on inner drop
    fn drop(&mut self) {
        if let Some(data) = self.data.take() {
            if let Ok(mut span_data) = data.lock().map(|mut data| data.take()) {
                if let Some(provider) = self.tracer.provider() {
                    // Set end time if unset or invalid
                    if let Some(data) = span_data.as_mut() {
                        if data.end_time <= data.start_time {
                            data.end_time = SystemTime::now();
                        }
                    }
                    let mut processors = provider.span_processors().iter().peekable();
                    while let Some(processor) = processors.next() {
                        let span_data = if processors.peek().is_none() {
                            // last loop or single processor/exporter, move data
                            span_data.take()
                        } else {
                            // clone so each exporter gets owned data
                            span_data.clone()
                        };

                        if let Some(span_data) = span_data {
                            processor.on_end(span_data);
                        }
                    }
                }
            }
        }
    }
}
