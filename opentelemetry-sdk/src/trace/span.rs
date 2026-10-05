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
use crate::trace::SpanLimits;
use opentelemetry::trace::{Event, Link, SpanContext, SpanId, SpanKind, Status};
use opentelemetry::KeyValue;
use std::borrow::Cow;
use std::time::SystemTime;

/// Single operation within a trace.
#[derive(Debug)]
pub struct Span {
    span_context: SpanContext,
    recording: Option<RecordingState>,
}

#[derive(Debug)]
struct RecordingState {
    data: SpanData,
    tracer: crate::trace::SdkTracer,
    span_limits: SpanLimits,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpanData {
    /// Span parent id
    pub(crate) parent_span_id: SpanId,
    /// Parent span is remote flag (for span flags)
    pub(crate) parent_span_is_remote: bool,
    /// Span kind
    pub(crate) span_kind: SpanKind,
    /// Span name
    pub(crate) name: Cow<'static, str>,
    /// Span start time
    pub(crate) start_time: SystemTime,
    /// Span end time
    pub(crate) end_time: SystemTime,
    /// Span attributes
    pub(crate) attributes: Vec<KeyValue>,
    /// The number of attributes that were above the configured limit, and thus
    /// dropped.
    pub(crate) dropped_attributes_count: u32,
    /// Span events
    pub(crate) events: crate::trace::SpanEvents,
    /// Span Links
    pub(crate) links: crate::trace::SpanLinks,
    /// Span status
    pub(crate) status: Status,
}

impl Span {
    #[inline]
    pub(crate) fn new_recording(
        span_context: SpanContext,
        data: SpanData,
        tracer: crate::trace::SdkTracer,
        span_limits: SpanLimits,
    ) -> Self {
        Span {
            span_context,
            recording: Some(RecordingState {
                data,
                tracer,
                span_limits,
            }),
        }
    }

    #[inline]
    pub(crate) fn new_non_recording(span_context: SpanContext) -> Self {
        Span {
            span_context,
            recording: None,
        }
    }

    #[cfg(all(test, feature = "testing"))]
    fn with_data<T, F>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut SpanData) -> T,
    {
        self.recording
            .as_mut()
            .map(|recording| f(&mut recording.data))
    }

    /// Convert information in this span into `exporter::trace::SpanData`.
    /// This function copies all data from the current span, which will create a
    /// overhead.
    pub fn exported_data(&self) -> Option<crate::trace::SpanData> {
        self.recording.as_ref().map(|recording| {
            build_export_data(
                recording.data.clone(),
                self.span_context.clone(),
                &recording.tracer,
            )
        })
    }
}

impl opentelemetry::trace::Span for Span {
    /// Records events at a specific time in the context of a given `Span`.
    ///
    /// Note that the OpenTelemetry project documents certain ["standard event names and
    /// keys"](https://github.com/open-telemetry/opentelemetry-specification/tree/v0.5.0/specification/trace/semantic_conventions/README.md)
    /// which have prescribed semantic meanings.
    fn add_event_with_timestamp<T>(
        &mut self,
        name: T,
        timestamp: SystemTime,
        mut attributes: Vec<KeyValue>,
    ) where
        T: Into<Cow<'static, str>>,
    {
        if let Some(recording) = &mut self.recording {
            let span_events_limit = recording.span_limits.max_events_per_span as usize;
            let event_attributes_limit = recording.span_limits.max_attributes_per_event as usize;
            if recording.data.events.len() < span_events_limit {
                let dropped_attributes_count =
                    attributes.len().saturating_sub(event_attributes_limit);
                attributes.truncate(event_attributes_limit);

                recording.data.events.add_event(Event::new(
                    name,
                    timestamp,
                    attributes,
                    dropped_attributes_count as u32,
                ));
            } else {
                recording.data.events.dropped_count += 1;
            }
        }
    }

    /// Returns the `SpanContext` for the given `Span`.
    #[inline]
    fn span_context(&self) -> &SpanContext {
        &self.span_context
    }

    /// Returns true if this `Span` is recording information like events with the `add_event`
    /// operation, attributes using `set_attributes`, status with `set_status`, etc.
    /// Always returns false after span `end`.
    #[inline]
    fn is_recording(&self) -> bool {
        self.recording.is_some()
    }

    /// Sets a single `Attribute` where the attribute properties are passed as arguments.
    ///
    /// Note that the OpenTelemetry project documents certain ["standard
    /// attributes"](https://github.com/open-telemetry/opentelemetry-specification/tree/v0.5.0/specification/trace/semantic_conventions/README.md)
    /// that have prescribed semantic meanings.
    #[inline]
    fn set_attribute(&mut self, attribute: KeyValue) {
        if let Some(recording) = &mut self.recording {
            let span_attribute_limit = recording.span_limits.max_attributes_per_span as usize;
            if recording.data.attributes.len() < span_attribute_limit {
                recording.data.attributes.push(attribute);
            } else {
                recording.data.dropped_attributes_count += 1;
            }
        }
    }

    /// Sets the status of this `Span`.
    ///
    /// If used, this will override the default span status, which is [`Status::Unset`].
    #[inline]
    fn set_status(&mut self, status: Status) {
        if let Some(recording) = &mut self.recording {
            // check if we should update the status
            // These values form a total order: Ok > Error > Unset.
            if status > recording.data.status {
                recording.data.status = status;
            }
        }
    }

    /// Updates the `Span`'s name.
    #[inline]
    fn update_name<T>(&mut self, new_name: T)
    where
        T: Into<Cow<'static, str>>,
    {
        if let Some(recording) = &mut self.recording {
            recording.data.name = new_name.into();
        }
    }

    /// Add `Link` to this `Span`
    ///
    #[inline]
    fn add_link(&mut self, span_context: SpanContext, attributes: Vec<KeyValue>) {
        if let Some(recording) = &mut self.recording {
            let span_links_limit = recording.span_limits.max_links_per_span as usize;
            let link_attributes_limit = recording.span_limits.max_attributes_per_link as usize;
            if recording.data.links.links.len() < span_links_limit {
                let dropped_attributes_count =
                    attributes.len().saturating_sub(link_attributes_limit);
                let mut attributes = attributes;
                attributes.truncate(link_attributes_limit);
                recording.data.links.add_link(Link::new(
                    span_context,
                    attributes,
                    dropped_attributes_count as u32,
                ));
            } else {
                recording.data.links.dropped_count += 1;
            }
        }
    }

    /// Finishes the span.
    #[inline]
    fn end(&mut self) {
        self.ensure_ended_and_exported(None);
    }

    /// Finishes the span with given timestamp.
    #[inline]
    fn end_with_timestamp(&mut self, timestamp: SystemTime) {
        self.ensure_ended_and_exported(Some(timestamp));
    }
}

impl Span {
    #[inline]
    fn ensure_ended_and_exported(&mut self, timestamp: Option<SystemTime>) {
        // skip if data has already been exported or span is non-recording
        let RecordingState {
            mut data,
            tracer,
            span_limits: _,
        } = match self.recording.take() {
            Some(recording) => recording,
            None => return,
        };

        let provider = tracer.provider();
        // skip if provider has been shut down
        if provider.is_shutdown() {
            return;
        }

        // ensure end time is set via explicit end or implicitly on drop
        if let Some(timestamp) = timestamp {
            data.end_time = timestamp;
        } else if data.end_time == data.start_time {
            data.end_time = opentelemetry::time::now();
        }

        match provider.span_processors() {
            [] => {}
            [processor] => {
                processor.on_end(build_export_data(data, self.span_context.clone(), &tracer));
            }
            processors => {
                for processor in processors {
                    processor.on_end(build_export_data(
                        data.clone(),
                        self.span_context.clone(),
                        &tracer,
                    ));
                }
            }
        }
    }
}

impl Drop for Span {
    /// Report span on inner drop
    #[inline]
    fn drop(&mut self) {
        self.ensure_ended_and_exported(None);
    }
}

fn build_export_data(
    data: SpanData,
    span_context: SpanContext,
    tracer: &crate::trace::SdkTracer,
) -> crate::trace::SpanData {
    crate::trace::SpanData {
        span_context,
        parent_span_id: data.parent_span_id,
        parent_span_is_remote: data.parent_span_is_remote,
        span_kind: data.span_kind,
        name: data.name,
        start_time: data.start_time,
        end_time: data.end_time,
        attributes: data.attributes,
        dropped_attributes_count: data.dropped_attributes_count,
        events: data.events,
        links: data.links,
        status: data.status,
        instrumentation_scope: tracer.instrumentation_scope().clone(),
    }
}

#[cfg(all(test, feature = "testing"))]
mod tests {
    use super::*;
    use crate::testing::trace::NoopSpanExporter;
    use crate::trace::span_limit::{
        DEFAULT_MAX_ATTRIBUTES_PER_EVENT, DEFAULT_MAX_ATTRIBUTES_PER_LINK,
        DEFAULT_MAX_ATTRIBUTES_PER_SPAN, DEFAULT_MAX_EVENT_PER_SPAN, DEFAULT_MAX_LINKS_PER_SPAN,
    };
    use crate::trace::{SpanEvents, SpanLinks};
    use opentelemetry::trace::{self, SpanBuilder, TraceFlags, TraceId, Tracer};
    use opentelemetry::{trace::Span as _, trace::TracerProvider};
    use std::time::Duration;
    use std::vec;

    fn init() -> (crate::trace::SdkTracer, SpanData) {
        let provider = crate::trace::SdkTracerProvider::default();
        let tracer = provider.tracer("opentelemetry");
        let data = SpanData {
            parent_span_id: SpanId::from(0),
            parent_span_is_remote: false,
            span_kind: trace::SpanKind::Internal,
            name: "opentelemetry".into(),
            start_time: opentelemetry::time::now(),
            end_time: opentelemetry::time::now(),
            attributes: Vec::new(),
            dropped_attributes_count: 0,
            events: SpanEvents::default(),
            links: SpanLinks::default(),
            status: Status::Unset,
        };
        (tracer, data)
    }

    fn create_span() -> Span {
        let (tracer, data) = init();
        Span::new_recording(
            SpanContext::empty_context(),
            data,
            tracer,
            Default::default(),
        )
    }

    #[test]
    fn create_span_without_data() {
        let mut span = Span::new_non_recording(SpanContext::empty_context());
        span.with_data(|_data| panic!("there are data"));
    }

    #[test]
    fn create_span_with_data_mut() {
        let (tracer, data) = init();
        let mut span = Span::new_recording(
            SpanContext::empty_context(),
            data.clone(),
            tracer,
            Default::default(),
        );
        span.with_data(|d| assert_eq!(*d, data));
    }

    #[test]
    fn add_event() {
        let mut span = create_span();
        let name = "some_event";
        let attributes = vec![KeyValue::new("k", "v")];
        span.add_event(name, attributes.clone());
        span.with_data(|data| {
            if let Some(event) = data.events.iter().next() {
                assert_eq!(event.name, name);
                assert_eq!(event.attributes, attributes);
            } else {
                panic!("no event");
            }
        });
    }

    #[test]
    fn add_event_with_timestamp() {
        let mut span = create_span();
        let name = "some_event";
        let attributes = vec![KeyValue::new("k", "v")];
        let timestamp = opentelemetry::time::now();
        span.add_event_with_timestamp(name, timestamp, attributes.clone());
        span.with_data(|data| {
            if let Some(event) = data.events.iter().next() {
                assert_eq!(event.timestamp, timestamp);
                assert_eq!(event.name, name);
                assert_eq!(event.attributes, attributes);
            } else {
                panic!("no event");
            }
        });
    }

    #[test]
    fn record_error() {
        let mut span = create_span();
        let err = std::io::Error::from(std::io::ErrorKind::Other);
        span.record_error(&err);
        span.with_data(|data| {
            if let Some(event) = data.events.iter().next() {
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
    fn set_attribute() {
        let mut span = create_span();
        let attributes = KeyValue::new("k", "v");
        span.set_attribute(attributes.clone());
        span.with_data(|data| {
            let matching_attribute: Vec<&KeyValue> = data
                .attributes
                .iter()
                .filter(|kv| kv.key.as_str() == attributes.key.as_str())
                .collect();
            if matching_attribute.len() == 1 {
                assert_eq!(matching_attribute[0].value, attributes.value);
            } else {
                panic!("no attribute");
            }
        });
    }

    #[test]
    fn set_attributes() {
        let mut span = create_span();
        let attributes = vec![KeyValue::new("k1", "v1"), KeyValue::new("k2", "v2")];
        span.set_attributes(attributes);
        span.with_data(|data| {
            assert_eq!(data.attributes.len(), 2);
        });
    }

    #[test]
    fn set_status() {
        {
            let mut span = create_span();
            let status = Status::Ok;
            span.set_status(status.clone());
            span.with_data(|data| assert_eq!(data.status, status));
        }
        {
            let mut span = create_span();
            let status = Status::Unset;
            span.set_status(status.clone());
            span.with_data(|data| assert_eq!(data.status, status));
        }
        {
            let mut span = create_span();
            let status = Status::error("error");
            span.set_status(status.clone());
            span.with_data(|data| assert_eq!(data.status, status));
        }
        {
            let mut span = create_span();
            // ok status should be final
            span.set_status(Status::Ok);
            span.set_status(Status::error("error"));
            span.with_data(|data| assert_eq!(data.status, Status::Ok));
        }
        {
            let mut span = create_span();
            // error status should be able to override unset
            span.set_status(Status::Unset);
            span.set_status(Status::error("error"));
            span.with_data(|data| assert_ne!(data.status, Status::Ok));
        }
    }

    #[test]
    fn update_name() {
        let mut span = create_span();
        let name = "new_name";
        span.update_name(name);
        span.with_data(|data| {
            assert_eq!(data.name, name);
        });
    }

    #[test]
    fn end() {
        let mut span = create_span();
        span.end();
    }

    #[test]
    fn end_with_timestamp() {
        let mut span = create_span();
        let timestamp = opentelemetry::time::now();
        span.end_with_timestamp(timestamp);
        span.with_data(|data| assert_eq!(data.end_time, timestamp));
    }

    #[test]
    fn allows_to_get_span_context_after_end() {
        let mut span = create_span();
        span.end();
        assert_eq!(span.span_context(), &SpanContext::empty_context());
    }

    #[test]
    fn end_only_once() {
        let mut span = create_span();
        let timestamp = opentelemetry::time::now();
        span.end_with_timestamp(timestamp);
        span.end_with_timestamp(timestamp.checked_add(Duration::from_secs(10)).unwrap());
        span.with_data(|data| assert_eq!(data.end_time, timestamp));
    }

    #[test]
    fn noop_after_end() {
        let mut span = create_span();
        let initial = span.with_data(|data| data.clone()).unwrap();
        span.end();
        span.add_event("some_event", vec![KeyValue::new("k", "v")]);
        span.add_event_with_timestamp(
            "some_event",
            opentelemetry::time::now(),
            vec![KeyValue::new("k", "v")],
        );
        let err = std::io::Error::from(std::io::ErrorKind::Other);
        span.record_error(&err);
        span.set_attribute(KeyValue::new("k", "v"));
        span.set_status(Status::error("ERROR"));
        span.update_name("new_name");
        span.with_data(|data| {
            assert_eq!(data.events, initial.events);
            assert_eq!(data.attributes, initial.attributes);
            assert_eq!(data.status, initial.status);
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
        let mut span = create_span();
        span.end();
        assert!(!span.is_recording());
    }

    #[test]
    fn exceed_span_attributes_limit() {
        let exporter = NoopSpanExporter::new();
        let provider_builder =
            crate::trace::SdkTracerProvider::builder().with_simple_exporter(exporter);
        let provider = provider_builder.build();
        let tracer = provider.tracer("opentelemetry-test");

        let mut initial_attributes = Vec::new();
        let mut expected_dropped_count = 1;
        for i in 0..(DEFAULT_MAX_ATTRIBUTES_PER_SPAN + 1) {
            initial_attributes.push(KeyValue::new(format!("key {i}"), i.to_string()))
        }
        let span_builder = SpanBuilder::from_name("test_span").with_attributes(initial_attributes);

        let mut span = tracer.build(span_builder);
        expected_dropped_count += 1;
        span.set_attribute(KeyValue::new("key3", "value3"));

        expected_dropped_count += 2;
        let span_attributes_after_creation =
            vec![KeyValue::new("foo", "1"), KeyValue::new("bar", "2")];
        span.set_attributes(span_attributes_after_creation);

        let actual_span = span
            .recording
            .as_ref()
            .expect("span recording state should not be empty as we already set it before")
            .data
            .clone();
        assert_eq!(
            actual_span.attributes.len(),
            DEFAULT_MAX_ATTRIBUTES_PER_SPAN as usize,
            "Span attributes should be truncated to the max limit"
        );
        assert_eq!(
            actual_span.dropped_attributes_count, expected_dropped_count,
            "Dropped count should match the actual count of attributes dropped"
        );
    }

    #[test]
    fn exceed_event_attributes_limit() {
        let exporter = NoopSpanExporter::new();
        let provider_builder =
            crate::trace::SdkTracerProvider::builder().with_simple_exporter(exporter);
        let provider = provider_builder.build();
        let tracer = provider.tracer("opentelemetry-test");

        let mut event1 = Event::with_name("test event");
        for i in 0..(DEFAULT_MAX_ATTRIBUTES_PER_EVENT * 2) {
            event1
                .attributes
                .push(KeyValue::new(format!("key {i}"), i.to_string()))
        }
        let event2 = event1.clone();

        // add event when build
        let span_builder = tracer.span_builder("test").with_events(vec![event1]);
        let mut span = tracer.build(span_builder);

        // add event after build
        span.add_event("another test event", event2.attributes);

        let event_queue = span
            .recording
            .as_ref()
            .expect("span recording state should not be empty as we already set it before")
            .data
            .events
            .clone();
        let event_vec: Vec<_> = event_queue.iter().take(2).collect();
        #[allow(clippy::get_first)] // we want to extract first two elements
        let processed_event_1 = event_vec.get(0).expect("should have at least two events");
        let processed_event_2 = event_vec.get(1).expect("should have at least two events");
        assert_eq!(processed_event_1.attributes.len(), 128);
        assert_eq!(processed_event_2.attributes.len(), 128);
    }

    #[test]
    fn exceed_link_attributes_limit() {
        let exporter = NoopSpanExporter::new();
        let provider_builder =
            crate::trace::SdkTracerProvider::builder().with_simple_exporter(exporter);
        let provider = provider_builder.build();
        let tracer = provider.tracer("opentelemetry-test");

        let mut link = Link::with_context(SpanContext::new(
            TraceId::from(12),
            SpanId::from(12),
            TraceFlags::default(),
            false,
            Default::default(),
        ));
        for i in 0..(DEFAULT_MAX_ATTRIBUTES_PER_LINK * 2) {
            link.attributes
                .push(KeyValue::new(format!("key {i}"), i.to_string()));
        }

        let span_builder = tracer.span_builder("test").with_links(vec![link]);
        let span = tracer.build(span_builder);
        let link_queue = span
            .recording
            .as_ref()
            .expect("span recording state should not be empty as we already set it before")
            .data
            .links
            .clone();
        let link_vec: Vec<_> = link_queue.links;
        let processed_link = link_vec.first().expect("should have at least one link");
        assert_eq!(processed_link.attributes.len(), 128);
    }

    #[test]
    fn exceed_span_links_limit() {
        let exporter = NoopSpanExporter::new();
        let provider_builder =
            crate::trace::SdkTracerProvider::builder().with_simple_exporter(exporter);
        let provider = provider_builder.build();
        let tracer = provider.tracer("opentelemetry-test");

        let mut links = Vec::new();
        for _i in 0..(DEFAULT_MAX_LINKS_PER_SPAN * 2) {
            links.push(Link::with_context(SpanContext::new(
                TraceId::from(12),
                SpanId::from(12),
                TraceFlags::default(),
                false,
                Default::default(),
            )))
        }

        let span_builder = tracer.span_builder("test").with_links(links);
        let mut span = tracer.build(span_builder);

        // add links using span api after building the span
        span.add_link(
            SpanContext::new(
                TraceId::from(12),
                SpanId::from(12),
                TraceFlags::default(),
                false,
                Default::default(),
            ),
            vec![],
        );
        let link_queue = span
            .recording
            .as_ref()
            .expect("span recording state should not be empty as we already set it before")
            .data
            .links
            .clone();
        let link_vec: Vec<_> = link_queue.links;
        assert_eq!(link_vec.len(), DEFAULT_MAX_LINKS_PER_SPAN as usize);
    }

    #[test]
    fn exceed_span_events_limit() {
        let exporter = NoopSpanExporter::new();
        let provider_builder =
            crate::trace::SdkTracerProvider::builder().with_simple_exporter(exporter);
        let provider = provider_builder.build();
        let tracer = provider.tracer("opentelemetry-test");

        let mut events = Vec::new();
        for _i in 0..(DEFAULT_MAX_EVENT_PER_SPAN * 2) {
            events.push(Event::with_name("test event"))
        }

        // add events via span builder
        let span_builder = tracer.span_builder("test").with_events(events);
        let mut span = tracer.build(span_builder);

        // add events using span api after building the span
        span.add_event("test event again, after span builder", Vec::new());
        span.add_event("test event once again, after span builder", Vec::new());
        let span_events = span
            .recording
            .as_ref()
            .expect("span recording state should not be empty as we already set it before")
            .data
            .events
            .clone();
        let event_vec: Vec<_> = span_events.events;
        assert_eq!(event_vec.len(), DEFAULT_MAX_EVENT_PER_SPAN as usize);
    }

    #[test]
    fn multiple_processors_receive_span_data() {
        use crate::trace::InMemorySpanExporterBuilder;

        let exporter1 = InMemorySpanExporterBuilder::new().build();
        let exporter2 = InMemorySpanExporterBuilder::new().build();

        let provider = crate::trace::SdkTracerProvider::builder()
            .with_simple_exporter(exporter1.clone())
            .with_simple_exporter(exporter2.clone())
            .build();

        let tracer = provider.tracer("test");
        let mut span = tracer.start("multi_processor_span");
        span.set_attribute(KeyValue::new("key", "value"));
        span.end();

        let spans1 = exporter1.get_finished_spans().unwrap();
        let spans2 = exporter2.get_finished_spans().unwrap();

        assert_eq!(spans1.len(), 1);
        assert_eq!(spans2.len(), 1);
        assert_eq!(spans1[0].name, "multi_processor_span");
        assert_eq!(spans2[0].name, "multi_processor_span");
        assert_eq!(spans1[0].attributes, spans2[0].attributes);
        // Verify instrumentation scope is correctly propagated to both processors
        assert_eq!(spans1[0].instrumentation_scope.name(), "test");
        assert_eq!(spans2[0].instrumentation_scope.name(), "test");
        assert_eq!(
            spans1[0].instrumentation_scope,
            spans2[0].instrumentation_scope
        );

        let _ = provider.shutdown();
    }

    #[test]
    fn test_span_exported_data() {
        let provider = crate::trace::SdkTracerProvider::builder()
            .with_simple_exporter(NoopSpanExporter::new())
            .build();
        let tracer = provider.tracer("test");

        let mut span = tracer.start("test_span");
        span.add_event("test_event", vec![]);
        span.set_status(Status::error(""));

        let exported_data = span.exported_data();
        assert!(exported_data.is_some());
        let res = provider.shutdown();
        println!("{res:?}");
        assert!(res.is_ok());
        let dropped_span = tracer.start("span_with_dropped_provider");
        // return none if the provider has already been dropped
        assert!(dropped_span.exported_data().is_none());
    }

    #[test]
    fn non_recording_span_mutations_are_noop() {
        let span_context = SpanContext::new(
            TraceId::from_bytes([1; 16]),
            SpanId::from_bytes([2; 8]),
            TraceFlags::default(),
            false,
            opentelemetry::trace::TraceState::default(),
        );
        let mut span = Span::new_non_recording(span_context.clone());

        assert!(!span.is_recording());
        assert_eq!(span.span_context(), &span_context);
        assert!(span.exported_data().is_none());

        span.set_attribute(KeyValue::new("k", "v"));
        span.set_attributes([KeyValue::new("k1", "v1"), KeyValue::new("k2", "v2")]);
        span.add_event("event", vec![KeyValue::new("k", "v")]);
        span.add_event_with_timestamp(
            "event_with_time",
            opentelemetry::time::now(),
            vec![KeyValue::new("k", "v")],
        );
        let err = std::io::Error::from(std::io::ErrorKind::Other);
        span.record_error(&err);
        span.set_status(Status::error("boom"));
        span.update_name("new_name");
        span.add_link(SpanContext::empty_context(), vec![KeyValue::new("k", "v")]);

        assert!(!span.is_recording());
        assert_eq!(span.span_context(), &span_context);
        assert!(span.exported_data().is_none());

        span.end();
        assert!(!span.is_recording());
        assert_eq!(span.span_context(), &span_context);
        assert!(span.exported_data().is_none());
    }

    #[test]
    fn sampling_decision_drop_returns_valid_non_recording_span() {
        use opentelemetry::trace::TraceContextExt;

        let exporter = crate::trace::InMemorySpanExporterBuilder::new().build();
        let provider = crate::trace::SdkTracerProvider::builder()
            .with_sampler(crate::trace::Sampler::AlwaysOff)
            .with_simple_exporter(exporter.clone())
            .build();
        let tracer = provider.tracer("test_tracer");

        let parent_trace_id = TraceId::from_bytes([42; 16]);
        let parent_span_id = SpanId::from_bytes([7; 8]);
        let trace_state =
            opentelemetry::trace::TraceState::from_key_value([("rojo", "1")]).unwrap();
        let parent_sc = SpanContext::new(
            parent_trace_id,
            parent_span_id,
            TraceFlags::default(),
            true,
            trace_state.clone(),
        );
        let parent_cx = opentelemetry::Context::new().with_remote_span_context(parent_sc);

        let mut span = tracer.start_with_context("dropped_span", &parent_cx);

        assert!(!span.is_recording());

        let sc = span.span_context();
        assert!(sc.is_valid());
        assert_eq!(sc.trace_id(), parent_trace_id);
        assert_ne!(sc.span_id(), SpanId::INVALID);
        assert_ne!(sc.span_id(), parent_span_id);
        assert!(!sc.is_sampled());
        assert_eq!(sc.trace_state(), &trace_state);

        drop(tracer);
        drop(provider);
        assert!(exporter.is_shutdown_called());

        span.set_attribute(KeyValue::new("dropped_key", "value"));
        span.add_event("dropped_event", vec![]);
        span.set_status(Status::error("error"));

        span.end();

        let finished_spans = exporter.get_finished_spans().unwrap();
        assert!(finished_spans.is_empty());
    }

    #[test]
    fn span_processors_not_invoked_for_dropped_span() {
        use crate::error::OTelSdkResult;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        #[derive(Debug)]
        struct CountingProcessor {
            starts: Arc<AtomicUsize>,
            ends: Arc<AtomicUsize>,
        }

        impl crate::trace::SpanProcessor for CountingProcessor {
            fn on_start(&self, _span: &mut Span, _cx: &opentelemetry::Context) {
                self.starts.fetch_add(1, Ordering::SeqCst);
            }
            fn on_end(&self, _span: crate::trace::SpanData) {
                self.ends.fetch_add(1, Ordering::SeqCst);
            }
            fn force_flush(&self) -> OTelSdkResult {
                Ok(())
            }
            fn shutdown(&self) -> OTelSdkResult {
                Ok(())
            }
            fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
                Ok(())
            }
        }

        let starts = Arc::new(AtomicUsize::new(0));
        let ends = Arc::new(AtomicUsize::new(0));

        let processor = CountingProcessor {
            starts: Arc::clone(&starts),
            ends: Arc::clone(&ends),
        };

        let provider = crate::trace::SdkTracerProvider::builder()
            .with_sampler(crate::trace::Sampler::AlwaysOff)
            .with_span_processor(processor)
            .build();
        let tracer = provider.tracer("counting");

        {
            let mut span = tracer.start("test_dropped");
            span.set_attribute(KeyValue::new("foo", "bar"));
            span.end();
        }

        assert_eq!(starts.load(Ordering::SeqCst), 0);
        assert_eq!(ends.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn recording_span_exports_once_and_releases_provider_on_end() {
        let exporter = crate::trace::InMemorySpanExporterBuilder::new()
            .keep_records_on_shutdown()
            .build();
        let provider = crate::trace::SdkTracerProvider::builder()
            .with_sampler(crate::trace::Sampler::AlwaysOn)
            .with_simple_exporter(exporter.clone())
            .build();
        let tracer = provider.tracer("recording_test");

        let mut span = tracer.start("recording_span");
        let span_context = span.span_context().clone();
        assert!(span.is_recording());

        span.set_attribute(KeyValue::new("attr1", "val1"));
        span.set_status(Status::Ok);

        assert!(span.is_recording());
        assert!(span.exported_data().is_some());

        drop(tracer);
        drop(provider);
        assert!(!exporter.is_shutdown_called());

        span.end();

        assert!(exporter.is_shutdown_called());
        assert!(!span.is_recording());
        assert_eq!(span.span_context(), &span_context);
        assert!(span.exported_data().is_none());

        span.set_attribute(KeyValue::new("attr2", "val2"));
        span.end();
        drop(span);

        let finished = exporter.get_finished_spans().unwrap();
        assert_eq!(finished.len(), 1);
        assert_eq!(finished[0].name, "recording_span");
        assert_eq!(finished[0].status, Status::Ok);
        assert_eq!(finished[0].attributes.len(), 1);
        assert_eq!(finished[0].attributes[0].key.as_str(), "attr1");
    }

    #[test]
    fn telemetry_suppressed_and_shutdown_produce_non_recording_spans() {
        let exporter = crate::trace::InMemorySpanExporterBuilder::new().build();
        let provider = crate::trace::SdkTracerProvider::builder()
            .with_sampler(crate::trace::Sampler::AlwaysOn)
            .with_simple_exporter(exporter.clone())
            .build();
        let tracer = provider.tracer("suppressed_test");

        let suppressed_cx = opentelemetry::Context::new().with_telemetry_suppressed();
        let mut suppressed_span = tracer.start_with_context("suppressed", &suppressed_cx);
        assert!(!suppressed_span.is_recording());
        assert_eq!(
            suppressed_span.span_context(),
            &SpanContext::empty_context()
        );
        suppressed_span.set_attribute(KeyValue::new("a", "b"));
        suppressed_span.end();

        assert!(provider.shutdown().is_ok());
        let mut shutdown_span = tracer.start("after_shutdown");
        assert!(!shutdown_span.is_recording());
        assert_eq!(shutdown_span.span_context(), &SpanContext::empty_context());
        shutdown_span.set_attribute(KeyValue::new("a", "b"));
        shutdown_span.end();

        let finished = exporter.get_finished_spans().unwrap();
        assert!(finished.is_empty());
    }
}
