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
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

#[derive(Debug)]
enum SpanInnerData {
    Unique(Option<SpanData>),
    Shared(Arc<Mutex<Option<SpanData>>>),
}

impl Default for SpanInnerData {
    fn default() -> Self {
        SpanInnerData::Unique(None)
    }
}

/// Single operation within a trace.
#[derive(Debug)]
pub struct Span {
    span_context: SpanContext,
    data: SpanInnerData,
    tracer: crate::trace::SdkTracer,
    span_limits: SpanLimits,
}

/// Thread safe reference to a span.
#[derive(Debug, Clone)]
pub struct SpanHandle {
    span_context: SpanContext,
    data: Arc<Mutex<Option<SpanData>>>,
    tracer: crate::trace::SdkTracer,
    span_limits: SpanLimits,
}

impl opentelemetry::trace::Span for SpanHandle {
    /// Records events at a specific time in the context of a given `Span`.
    fn add_event_with_timestamp<T>(
        &mut self,
        name: T,
        timestamp: SystemTime,
        attributes: Vec<KeyValue>,
    ) where
        T: Into<Cow<'static, str>>,
    {
        let span_events_limit = self.span_limits.max_events_per_span as usize;
        let span_attributes_limit = self.span_limits.max_attributes_per_event as usize;
        self.with_data(|data| {
            data_add_event(
                data,
                name.into(),
                timestamp,
                attributes,
                span_events_limit,
                span_attributes_limit,
            )
        });
    }

    /// Returns the `SpanContext` for the given `Span`.
    fn span_context(&self) -> &SpanContext {
        &self.span_context
    }

    /// Returns `true` if this span is recording information.
    fn is_recording(&self) -> bool {
        self.data
            .lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }

    /// Sets a single `Attribute` where the attribute properties are passed as arguments.
    fn set_attribute(&mut self, attribute: KeyValue) {
        let span_attribute_limit = self.span_limits.max_attributes_per_span as usize;
        self.with_data(|data| data_set_attribute(data, attribute, span_attribute_limit));
    }

    /// Sets the status of this `Span`.
    fn set_status(&mut self, status: Status) {
        self.with_data(|data| {
            if status > data.status {
                data.status = status;
            }
        });
    }

    /// Updates the span's name.
    fn update_name<T>(&mut self, new_name: T)
    where
        T: Into<Cow<'static, str>>,
    {
        self.with_data(|data| data.name = new_name.into());
    }

    /// Add a link to the span.
    fn add_link(&mut self, span_context: SpanContext, attributes: Vec<KeyValue>) {
        let span_links_limit = self.span_limits.max_links_per_span as usize;
        let link_attributes_limit = self.span_limits.max_attributes_per_link as usize;
        self.with_data(|data| {
            data_add_link(
                data,
                span_context,
                attributes,
                span_links_limit,
                link_attributes_limit,
            )
        });
    }

    /// Finishes the span with given timestamp.
    fn end_with_timestamp(&mut self, timestamp: SystemTime) {
        // Take data from the mutex, marking span as ended
        let data = match self.data.lock().ok().and_then(|mut guard| guard.take()) {
            Some(d) => d,
            None => return, // Already ended
        };
        let span_context = self.span_context.clone();
        end_and_export_span(data, span_context, &self.tracer, Some(timestamp));
    }
}

impl SpanHandle {
    /// Operate on a shared reference to span data
    fn with_data_ref<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&SpanData) -> T,
    {
        self.data
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().map(f))
    }

    /// Operate on a mutable reference to span data
    fn with_data<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut SpanData) -> T,
    {
        self.data
            .lock()
            .ok()
            .and_then(|mut guard| guard.as_mut().map(f))
    }

    /// Convert information in this span into `exporter::trace::SpanData`.
    pub fn exported_data(&self) -> Option<crate::trace::SpanData> {
        let (span_context, tracer) = (self.span_context.clone(), &self.tracer);

        self.with_data_ref(|data| build_export_data(data.clone(), span_context, tracer))
    }
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
    pub(crate) fn new(
        span_context: SpanContext,
        data: Option<SpanData>,
        tracer: crate::trace::SdkTracer,
        span_limit: SpanLimits,
    ) -> Self {
        Span {
            span_context,
            data: SpanInnerData::Unique(data),
            tracer,
            span_limits: span_limit,
        }
    }

    /// Operate on a mutable reference to span data
    fn with_data<T, F>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut SpanData) -> T,
    {
        match &mut self.data {
            SpanInnerData::Unique(data) => data.as_mut().map(f),
            SpanInnerData::Shared(data) => {
                data.lock().ok().and_then(|mut guard| guard.as_mut().map(f))
            }
        }
    }

    /// Convert information in this span into `exporter::trace::SpanData`.
    /// This function copies all data from the current span, which will create a
    /// overhead.
    pub fn exported_data(&self) -> Option<crate::trace::SpanData> {
        let (span_context, tracer) = (self.span_context.clone(), &self.tracer);

        match &self.data {
            SpanInnerData::Unique(data) => data
                .as_ref()
                .map(|data| build_export_data(data.clone(), span_context, tracer)),
            SpanInnerData::Shared(data) => data.lock().ok().and_then(|guard| {
                guard
                    .as_ref()
                    .map(|data| build_export_data(data.clone(), span_context, tracer))
            }),
        }
    }

    /// Get a shared handle to this span.
    ///
    /// This method upgrades the span from unique mode to shared mode
    /// (where data is protected by mutex).
    /// This allows span processors to keep a handle to the span.
    ///
    /// # Example
    ///
    /// ```ignore
    /// impl SpanProcessor for MyProcessor {
    ///     fn on_start(&self, span: &mut Span, _cx: &Context) {
    ///         let handle = span.get_handle();
    ///         // Store handle for later use
    ///     }
    /// }
    /// ```
    pub fn get_handle(&mut self) -> SpanHandle {
        self.upgrade_to_shared();

        let span_context = self.span_context.clone();
        let data = match &self.data {
            SpanInnerData::Shared(data) => data.clone(),
            SpanInnerData::Unique(_) => {
                unreachable!("span is `Shared` after upgrading")
            }
        };

        SpanHandle {
            span_context,
            data,
            tracer: self.tracer.clone(),
            span_limits: self.span_limits,
        }
    }

    /// Upgrade the span from unique to shared mode.
    fn upgrade_to_shared(&mut self) {
        if let SpanInnerData::Unique(_) = &self.data {
            let SpanInnerData::Unique(data) = std::mem::take(&mut self.data) else {
                unreachable!("span is `Unique`")
            };
            self.data = SpanInnerData::Shared(Arc::new(Mutex::new(data)));
        }
    }

    /// Check if the span is `Shared`.
    pub fn is_shared(&self) -> bool {
        matches!(self.data, SpanInnerData::Shared { .. })
    }

    /// Returns a clone of the internal span data for testing purposes.
    #[cfg(all(test, feature = "testing"))]
    pub(crate) fn data(&self) -> Option<SpanData> {
        match &self.data {
            SpanInnerData::Unique(data) => data.clone(),
            SpanInnerData::Shared(data) => data.lock().ok().and_then(|guard| guard.clone()),
        }
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
        attributes: Vec<KeyValue>,
    ) where
        T: Into<Cow<'static, str>>,
    {
        let span_events_limit = self.span_limits.max_events_per_span as usize;
        let event_attributes_limit = self.span_limits.max_attributes_per_event as usize;
        self.with_data(|data| {
            data_add_event(
                data,
                name.into(),
                timestamp,
                attributes,
                span_events_limit,
                event_attributes_limit,
            )
        });
    }

    /// Returns the `SpanContext` for the given `Span`.
    fn span_context(&self) -> &SpanContext {
        &self.span_context
    }

    /// Returns true if this `Span` is recording information like events with the `add_event`
    /// operation, attributes using `set_attributes`, status with `set_status`, etc.
    /// Always returns false after span `end`.
    fn is_recording(&self) -> bool {
        match &self.data {
            SpanInnerData::Unique(data) => data.is_some(),
            SpanInnerData::Shared(data) => {
                data.lock().map(|guard| guard.is_some()).unwrap_or(false)
            }
        }
    }

    /// Sets a single `Attribute` where the attribute properties are passed as arguments.
    ///
    /// Note that the OpenTelemetry project documents certain ["standard
    /// attributes"](https://github.com/open-telemetry/opentelemetry-specification/tree/v0.5.0/specification/trace/semantic_conventions/README.md)
    /// that have prescribed semantic meanings.
    fn set_attribute(&mut self, attribute: KeyValue) {
        let span_attribute_limit = self.span_limits.max_attributes_per_span as usize;
        self.with_data(|data| data_set_attribute(data, attribute, span_attribute_limit));
    }

    /// Sets the status of this `Span`.
    ///
    /// If used, this will override the default span status, which is [`Status::Unset`].
    fn set_status(&mut self, status: Status) {
        self.with_data(|data| {
            // check if we should update the status
            // These values form a total order: Ok > Error > Unset.
            if status > data.status {
                data.status = status;
            }
        });
    }

    /// Updates the `Span`'s name.
    fn update_name<T>(&mut self, new_name: T)
    where
        T: Into<Cow<'static, str>>,
    {
        self.with_data(|data| {
            data.name = new_name.into();
        });
    }

    /// Add `Link` to this `Span`
    ///
    fn add_link(&mut self, span_context: SpanContext, attributes: Vec<KeyValue>) {
        let span_links_limit = self.span_limits.max_links_per_span as usize;
        let link_attributes_limit = self.span_limits.max_attributes_per_link as usize;
        self.with_data(|data| {
            data_add_link(
                data,
                span_context,
                attributes,
                span_links_limit,
                link_attributes_limit,
            )
        });
    }

    /// Finishes the span with given timestamp.
    fn end_with_timestamp(&mut self, timestamp: SystemTime) {
        self.ensure_ended_and_exported(Some(timestamp));
    }
}

impl Span {
    fn ensure_ended_and_exported(&mut self, timestamp: Option<SystemTime>) {
        // Take data, skip if it has already been exported
        let data = match &mut self.data {
            SpanInnerData::Unique(data) => match data.take() {
                Some(d) => d,
                None => return, // Already ended
            },
            SpanInnerData::Shared(data) => {
                match data.lock().ok().and_then(|mut guard| guard.take()) {
                    Some(d) => d,
                    None => return, // Already ended
                }
            }
        };

        end_and_export_span(data, self.span_context.clone(), &self.tracer, timestamp);
    }
}

impl Drop for Span {
    /// Report span on inner drop
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

fn data_set_attribute(data: &mut SpanData, attribute: KeyValue, span_attribute_limit: usize) {
    if data.attributes.len() < span_attribute_limit {
        data.attributes.push(attribute);
    } else {
        data.dropped_attributes_count += 1;
    }
}

fn data_add_event(
    data: &mut SpanData,
    name: Cow<'static, str>,
    timestamp: SystemTime,
    mut attributes: Vec<KeyValue>,
    span_events_limit: usize,
    event_attributes_limit: usize,
) {
    if data.events.len() < span_events_limit {
        let dropped_attributes_count = attributes.len().saturating_sub(event_attributes_limit);
        attributes.truncate(event_attributes_limit);

        data.events.add_event(Event::new(
            name,
            timestamp,
            attributes,
            dropped_attributes_count as u32,
        ));
    } else {
        data.events.dropped_count += 1;
    }
}

fn data_add_link(
    data: &mut SpanData,
    span_context: SpanContext,
    mut attributes: Vec<KeyValue>,
    span_links_limit: usize,
    link_attributes_limit: usize,
) {
    if data.links.links.len() < span_links_limit {
        let dropped_attributes_count = attributes.len().saturating_sub(link_attributes_limit);
        attributes.truncate(link_attributes_limit);
        data.links.add_link(Link::new(
            span_context,
            attributes,
            dropped_attributes_count as u32,
        ));
    } else {
        data.links.dropped_count += 1;
    }
}

fn end_and_export_span(
    mut data: SpanData,
    span_context: SpanContext,
    tracer: &crate::trace::SdkTracer,
    timestamp: Option<SystemTime>,
) {
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
            processor.on_end(build_export_data(data, span_context, tracer));
        }
        processors => {
            for processor in processors {
                processor.on_end(build_export_data(
                    data.clone(),
                    span_context.clone(),
                    tracer,
                ));
            }
        }
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
        Span::new(
            SpanContext::empty_context(),
            Some(data),
            tracer,
            Default::default(),
        )
    }

    #[test]
    fn create_span_without_data() {
        let (tracer, _) = init();
        let mut span = Span::new(
            SpanContext::empty_context(),
            None,
            tracer,
            Default::default(),
        );
        span.with_data(|_data| panic!("there are data"));
    }

    #[test]
    fn create_span_with_data_mut() {
        let (tracer, data) = init();
        let mut span = Span::new(
            SpanContext::empty_context(),
            Some(data.clone()),
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
            .data()
            .expect("span data should not be empty as we already set it before");
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
            .data()
            .expect("span data should not be empty as we already set it before")
            .events;
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
            .data()
            .expect("span data should not be empty as we already set it before")
            .links;
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
            .data()
            .expect("span data should not be empty as we already set it before")
            .links;
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
            .data()
            .expect("span data should not be empty as we already set it before")
            .events;
        let event_vec: Vec<_> = span_events.events;
        assert_eq!(event_vec.len(), DEFAULT_MAX_EVENT_PER_SPAN as usize);
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
    fn span_starts_in_unique_mode() {
        let span = create_span();
        assert!(!span.is_shared());
    }

    #[test]
    fn get_handle_upgrades_to_shared_mode() {
        let mut span = create_span();
        assert!(!span.is_shared());
        let _handle = span.get_handle();
        assert!(span.is_shared());
    }

    #[test]
    fn handle_and_span_are_recording_when_not_ended() {
        let mut span = create_span();
        let handle = span.get_handle();
        assert!(handle.is_recording());
        assert!(span.is_recording());
    }

    #[test]
    fn handle_and_span_are_not_recording_after_span_end() {
        let mut span = create_span();
        let handle = span.get_handle();
        span.end();
        assert!(!handle.is_recording());
        assert!(!span.is_recording());
    }

    #[test]
    fn handle_modifications_visible_to_span() {
        let mut span = create_span();
        let mut handle = span.get_handle();

        handle.set_attribute(KeyValue::new("handle_key", "handle_value"));
        handle.set_status(Status::error("some error"));
        handle.update_name("new_name_from_handle");

        span.with_data(|data| {
            assert_eq!(data.name, "new_name_from_handle");
            assert!(matches!(data.status, Status::Error { ref description } if description == "some error"));
            assert!(data
                .attributes
                .iter()
                .any(|kv| kv.key.as_str() == "handle_key" && kv.value == "handle_value".into()));
        });
    }

    #[test]
    fn span_modifications_visible_to_handle() {
        let mut span = create_span();
        let handle = span.get_handle();

        span.set_attribute(KeyValue::new("span_key", "span_value"));
        span.set_status(Status::error("some error"));
        span.update_name("new_name_from_span");

        handle.with_data_ref(|data| {
            assert_eq!(data.name, "new_name_from_span");
            assert!(matches!(
                data.status,
                Status::Error { ref description } if description == "some error"
            ));
            assert!(data
                .attributes
                .iter()
                .any(|kv| kv.key.as_str() == "span_key" && kv.value == "span_value".into()));
        });
    }

    #[test]
    fn handle_add_event() {
        let mut span = create_span();
        let mut handle = span.get_handle();

        handle.add_event("ref_event", vec![KeyValue::new("event_key", "event_value")]);

        span.with_data(|data| {
            assert_eq!(data.events.len(), 1);
            let event = data.events.iter().next().unwrap();
            assert_eq!(event.name, "ref_event");
        });
    }

    #[test]
    fn handle_add_link() {
        let mut span = create_span();
        let mut handle = span.get_handle();

        handle.add_link(
            SpanContext::new(
                TraceId::from(42),
                SpanId::from(24),
                TraceFlags::default(),
                false,
                Default::default(),
            ),
            vec![],
        );

        span.with_data(|data| {
            assert_eq!(data.links.links.len(), 1);
            assert_eq!(
                data.links.links[0].span_context.trace_id(),
                TraceId::from(42)
            );
        });
    }

    #[test]
    fn handle_cloning_shares_same_data() {
        let mut span = create_span();
        let mut handle1 = span.get_handle();
        let mut handle2 = handle1.clone();

        // modify first handle
        handle1.set_attribute(KeyValue::new("key1", "value1"));

        // change is visible from second handle
        handle2.with_data_ref(|data| {
            assert!(data.attributes.iter().any(|kv| kv.key.as_str() == "key1"));
        });

        // modify second handle
        handle2.set_attribute(KeyValue::new("key2", "value2"));

        // change visible from first handle
        handle1.with_data_ref(|data| {
            assert!(data.attributes.iter().any(|kv| kv.key.as_str() == "key2"));
        });
    }

    #[test]
    fn handle_exported_data() {
        let mut span = create_span();
        let mut handle = span.get_handle();

        handle.set_attribute(KeyValue::new("export_key", "export_value"));
        handle.set_status(Status::Ok);

        let exported = handle.exported_data();
        assert!(exported.is_some());
        let data = exported.unwrap();
        assert_eq!(data.status, Status::Ok);
        assert!(data
            .attributes
            .iter()
            .any(|kv| kv.key.as_str() == "export_key"));
        assert_eq!(data.status, Status::Ok);
    }

    #[test]
    fn handle_span_context() {
        let (tracer, data) = init();
        let span_context = SpanContext::new(
            TraceId::from(123),
            SpanId::from(456),
            TraceFlags::SAMPLED,
            false,
            Default::default(),
        );
        let mut span = Span::new(span_context.clone(), Some(data), tracer, Default::default());

        let handle = span.get_handle();
        assert_eq!(handle.span_context().trace_id(), span_context.trace_id());
        assert_eq!(handle.span_context().span_id(), span_context.span_id());
    }

    #[test]
    fn is_recording_works_in_unique_mode() {
        let span = create_span();
        assert!(!span.is_shared());
        assert!(span.is_recording());
    }

    #[test]
    fn is_recording_works_in_shared_mode() {
        let mut span = create_span();
        let _handle = span.get_handle();
        assert!(span.is_shared());
        assert!(span.is_recording());
    }

    #[test]
    fn end_works_in_shared_mode() {
        let mut span = create_span();
        let handle = span.get_handle();
        assert!(span.is_recording());
        assert!(handle.is_recording());

        span.end();

        assert!(!span.is_recording());
        assert!(!handle.is_recording());
    }

    #[test]
    fn multiple_get_handle_calls_return_same_shared_state() {
        let mut span = create_span();
        let mut handle1 = span.get_handle();
        let handle2 = span.get_handle();

        handle1.set_attribute(KeyValue::new("from_handle1", "value1"));

        handle2.with_data_ref(|data| {
            assert!(data
                .attributes
                .iter()
                .any(|kv| kv.key.as_str() == "from_handle1"));
        });
    }
}
