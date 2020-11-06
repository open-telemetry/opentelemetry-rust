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
use crate::trace::{Event, SpanContext, SpanId, SpanKind, StatusCode};
use crate::{api, exporter, sdk, KeyValue};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Single operation within a trace.
#[derive(Clone, Debug)]
pub struct Span {
    inner: Arc<SpanInner>,
}

/// Inner data, processed and exported on end
#[derive(Debug)]
struct SpanInner {
    span_context: SpanContext,
    data: Option<Mutex<Option<SpanData>>>,
    tracer: sdk::trace::Tracer,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpanData {
    /// Span parent id
    pub(crate) parent_span_id: SpanId,
    /// Span kind
    pub(crate) span_kind: SpanKind,
    /// Span name
    pub(crate) name: String,
    /// Span start time
    pub(crate) start_time: SystemTime,
    /// Span end time
    pub(crate) end_time: SystemTime,
    /// Span attributes
    pub(crate) attributes: sdk::trace::EvictedHashMap,
    /// Span Message events
    pub(crate) message_events: sdk::trace::EvictedQueue<api::trace::Event>,
    /// Span Links
    pub(crate) links: sdk::trace::EvictedQueue<api::trace::Link>,
    /// Span status code
    pub(crate) status_code: StatusCode,
    /// Span status message
    pub(crate) status_message: String,
    /// Resource contains attributes representing an entity that produced this span.
    pub(crate) resource: Arc<sdk::Resource>,
}

impl Span {
    pub(crate) fn new(
        span_context: SpanContext,
        data: Option<SpanData>,
        tracer: sdk::trace::Tracer,
    ) -> Self {
        Span {
            inner: Arc::new(SpanInner {
                span_context,
                data: data.map(|data| Mutex::new(Some(data))),
                tracer,
            }),
        }
    }

    /// Operate on a mutable reference to span data
    fn with_data<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut SpanData) -> T,
    {
        self.inner.data.as_ref().and_then(|inner| {
            inner
                .lock()
                .ok()
                .and_then(|mut span_data| span_data.as_mut().map(f))
        })
    }
}

impl crate::trace::Span for Span {
    /// Records events at a specific time in the context of a given `Span`.
    ///
    /// Note that the OpenTelemetry project documents certain ["standard event names and
    /// keys"](https://github.com/open-telemetry/opentelemetry-specification/tree/v0.5.0/specification/trace/semantic_conventions/README.md)
    /// which have prescribed semantic meanings.
    fn add_event_with_timestamp(
        &self,
        name: String,
        timestamp: SystemTime,
        attributes: Vec<KeyValue>,
    ) {
        self.with_data(|data| {
            data.message_events
                .push_back(Event::new(name, timestamp, attributes))
        });
    }

    /// Returns the `SpanContext` for the given `Span`.
    fn span_context(&self) -> &SpanContext {
        &self.inner.span_context
    }

    /// Returns true if this `Span` is recording information like events with the `add_event`
    /// operation, attributes using `set_attributes`, status with `set_status`, etc.
    /// Always returns false after span `end`.
    fn is_recording(&self) -> bool {
        if let Some(data) = &self.inner.data {
            if let Ok(span_data) = data.lock() {
                return span_data.is_some();
            }
        }
        false
    }

    /// Sets a single `Attribute` where the attribute properties are passed as arguments.
    ///
    /// Note that the OpenTelemetry project documents certain ["standard
    /// attributes"](https://github.com/open-telemetry/opentelemetry-specification/tree/v0.5.0/specification/trace/semantic_conventions/README.md)
    /// that have prescribed semantic meanings.
    fn set_attribute(&self, attribute: KeyValue) {
        self.with_data(|data| {
            data.attributes.insert(attribute);
        });
    }

    /// An API to set multiple `Attribute`. This function works similar with `set_attribute` but can
    /// add multiple attributes at once. For more details, refer to `set_attribute`.
    fn extend_attributes(&self, attributes: impl Iterator<Item = KeyValue>) {
        self.with_data(|data| {
            attributes.for_each(|attr| {
                data.attributes.insert(attr);
            });
        });
    }

    /// Sets the status of the `Span`. If used, this will override the default `Span`
    /// status, which is `Unset`.
    fn set_status(&self, code: StatusCode, message: String) {
        self.with_data(|data| {
            data.status_code = code;
            data.status_message = message
        });
    }

    /// Updates the `Span`'s name.
    fn update_name(&self, new_name: String) {
        self.with_data(|data| {
            data.name = new_name;
        });
    }

    /// Finishes the span with given timestamp.
    fn end_with_timestamp(&self, timestamp: SystemTime) {
        self.inner.ensure_ended_and_exported(Some(timestamp));
    }
}

impl SpanInner {
    fn ensure_ended_and_exported(&self, timestamp: Option<SystemTime>) {
        if let Some(data) = &self.data {
            if let Ok(mut span_data) = data.lock().map(|mut data| data.take()) {
                // Ensure end time is set via explicit end or implicitly on drop
                if let Some(span_data) = span_data.as_mut() {
                    if let Some(timestamp) = timestamp {
                        span_data.end_time = timestamp;
                    } else if span_data.end_time == span_data.start_time {
                        span_data.end_time = SystemTime::now();
                    }
                }

                // Notify each span processor that the span has ended
                if let Some(provider) = self.tracer.provider() {
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
                            processor.on_end(build_export_data(
                                span_data,
                                self.span_context.clone(),
                                &self.tracer,
                            ));
                        }
                    }
                }
            }
        }
    }
}

impl Drop for SpanInner {
    /// Report span on inner drop
    fn drop(&mut self) {
        self.ensure_ended_and_exported(None);
    }
}

fn build_export_data(
    data: SpanData,
    span_context: SpanContext,
    tracer: &sdk::trace::Tracer,
) -> exporter::trace::SpanData {
    exporter::trace::SpanData {
        span_context,
        parent_span_id: data.parent_span_id,
        span_kind: data.span_kind,
        name: data.name,
        start_time: data.start_time,
        end_time: data.end_time,
        attributes: data.attributes,
        message_events: data.message_events,
        links: data.links,
        status_code: data.status_code,
        status_message: data.status_message,
        resource: data.resource,
        instrumentation_lib: *tracer.instrumentation_library(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{api, api::core::KeyValue, api::trace::Span as _, api::trace::TracerProvider};
    use std::time::Duration;

    fn init() -> (sdk::trace::Tracer, SpanData) {
        let provider = sdk::trace::TracerProvider::default();
        let config = provider.config();
        let tracer = provider.get_tracer("opentelemetry", Some(env!("CARGO_PKG_VERSION")));
        let data = SpanData {
            parent_span_id: SpanId::from_u64(0),
            span_kind: api::trace::SpanKind::Internal,
            name: "opentelemetry".to_string(),
            start_time: SystemTime::now(),
            end_time: SystemTime::now(),
            attributes: sdk::trace::EvictedHashMap::new(config.max_attributes_per_span, 0),
            message_events: sdk::trace::EvictedQueue::new(config.max_events_per_span),
            links: sdk::trace::EvictedQueue::new(config.max_links_per_span),
            status_code: StatusCode::Unset,
            status_message: "".to_string(),
            resource: config.resource.clone(),
        };
        (tracer, data)
    }

    fn create_span() -> Span {
        let (tracer, data) = init();
        Span::new(SpanContext::empty_context(), Some(data), tracer)
    }

    #[test]
    fn create_span_without_data() {
        let (tracer, _) = init();
        let span = Span::new(SpanContext::empty_context(), None, tracer);
        span.with_data(|_data| panic!("there are data"));
    }

    #[test]
    fn create_span_with_data_mut() {
        let (tracer, data) = init();
        let span = Span::new(SpanContext::empty_context(), Some(data.clone()), tracer);
        span.with_data(|d| assert_eq!(*d, data));
    }

    #[test]
    fn add_event() {
        let span = create_span();
        let name = "some_event".to_string();
        let attributes = vec![KeyValue::new("k", "v")];
        span.add_event(name.clone(), attributes.clone());
        span.with_data(|data| {
            if let Some(event) = data.message_events.iter().next() {
                assert_eq!(event.name, name);
                assert_eq!(event.attributes, attributes);
            } else {
                panic!("no event");
            }
        });
    }

    #[test]
    fn add_event_with_timestamp() {
        let span = create_span();
        let name = "some_event".to_string();
        let attributes = vec![KeyValue::new("k", "v")];
        let timestamp = SystemTime::now();
        span.add_event_with_timestamp(name.clone(), timestamp, attributes.clone());
        span.with_data(|data| {
            if let Some(event) = data.message_events.iter().next() {
                assert_eq!(event.timestamp, timestamp);
                assert_eq!(event.name, name);
                assert_eq!(event.attributes, attributes);
            } else {
                panic!("no event");
            }
        });
    }

    #[test]
    fn record_exception() {
        let span = create_span();
        let err = std::io::Error::from(std::io::ErrorKind::Other);
        span.record_exception(&err);
        span.with_data(|data| {
            if let Some(event) = data.message_events.iter().next() {
                assert_eq!(event.name, "exception");
                assert_eq!(
                    event.attributes,
                    vec![KeyValue::new("exception.message", err.to_string())]
                );
            } else {
                panic!("no event");
            }
        });
    }

    #[test]
    fn record_exception_with_stacktrace() {
        let span = create_span();
        let err = std::io::Error::from(std::io::ErrorKind::Other);
        let stacktrace = "stacktrace...".to_string();
        span.record_exception_with_stacktrace(&err, stacktrace.clone());
        span.with_data(|data| {
            if let Some(event) = data.message_events.iter().next() {
                assert_eq!(event.name, "exception");
                assert_eq!(
                    event.attributes,
                    vec![
                        KeyValue::new("exception.message", err.to_string()),
                        KeyValue::new("exception.stacktrace", stacktrace),
                    ]
                );
            } else {
                panic!("no event");
            }
        });
    }

    #[test]
    fn set_attribute() {
        let span = create_span();
        let attributes = KeyValue::new("k", "v");
        span.set_attribute(attributes.clone());
        span.with_data(|data| {
            if let Some(val) = data.attributes.get(&attributes.key) {
                assert_eq!(*val, attributes.value);
            } else {
                panic!("no attribute");
            }
        });
    }

    #[test]
    fn set_status() {
        let span = create_span();
        let status = StatusCode::Ok;
        let message = "OK".to_string();
        span.set_status(status.clone(), message.clone());
        span.with_data(|data| {
            assert_eq!(data.status_code, status);
            assert_eq!(data.status_message, message);
        });
    }

    #[test]
    fn update_name() {
        let span = create_span();
        let name = "new_name".to_string();
        span.update_name(name.clone());
        span.with_data(|data| {
            assert_eq!(data.name, name);
        });
    }

    #[test]
    fn end() {
        let span = create_span();
        span.end();
    }

    #[test]
    fn end_with_timestamp() {
        let span = create_span();
        let timestamp = SystemTime::now();
        span.end_with_timestamp(timestamp);
        span.with_data(|data| assert_eq!(data.end_time, timestamp));
    }

    #[test]
    fn allows_to_get_span_context_after_end() {
        let span = create_span();
        span.end();
        assert_eq!(span.span_context(), &SpanContext::empty_context());
    }

    #[test]
    fn allows_to_get_span_context_after_clone_drop() {
        let span = create_span();
        drop(span.clone());
        assert_eq!(span.span_context(), &SpanContext::empty_context());
    }

    #[test]
    fn end_only_once() {
        let span = create_span();
        let timestamp = SystemTime::now();
        span.end_with_timestamp(timestamp);
        span.end_with_timestamp(timestamp.checked_add(Duration::from_secs(10)).unwrap());
        span.with_data(|data| assert_eq!(data.end_time, timestamp));
    }

    #[test]
    fn noop_after_end() {
        let span = create_span();
        let initial = span.with_data(|data| data.clone()).unwrap();
        span.end();
        span.add_event("some_event".to_string(), vec![KeyValue::new("k", "v")]);
        span.add_event_with_timestamp(
            "some_event".to_string(),
            SystemTime::now(),
            vec![KeyValue::new("k", "v")],
        );
        let err = std::io::Error::from(std::io::ErrorKind::Other);
        span.record_exception(&err);
        span.record_exception_with_stacktrace(&err, "stacktrace...".to_string());
        span.set_attribute(KeyValue::new("k", "v"));
        span.set_status(StatusCode::Error, "ERROR".to_string());
        span.update_name("new_name".to_string());
        span.with_data(|data| {
            assert_eq!(data.message_events, initial.message_events);
            assert_eq!(data.attributes, initial.attributes);
            assert_eq!(data.status_code, initial.status_code);
            assert_eq!(data.status_message, initial.status_message);
            assert_eq!(data.name, initial.name);
        });
    }

    #[test]
    fn is_recording_true_when_not_ended() {
        let span = create_span();
        assert!(span.is_recording());
    }

    #[test]
    fn is_recording_false_after_end() {
        let span = create_span();
        span.end();
        assert!(!span.is_recording());
    }
}
