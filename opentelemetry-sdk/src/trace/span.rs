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
    data: Option<SpanData>,
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
    /// Whether end_time was explicitly set via `end_with_timestamp`
    pub(crate) end_time_set: bool,
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
            data,
            tracer,
            span_limits: span_limit,
        }
    }

    /// Operate on a mutable reference to span data
    fn with_data<T, F>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut SpanData) -> T,
    {
        self.data.as_mut().map(f)
    }

    /// Convert information in this span into `SpanData`.
    ///
    /// This function clones all data from the current span. For read-only inspection
    /// in `on_start`, prefer using the clone-free inherent read methods (`attributes()`,
    /// `name()`, `span_kind()`, etc.) instead.
    pub fn exported_data(&self) -> Option<crate::trace::SpanData> {
        let (span_context, tracer) = (self.span_context.clone(), &self.tracer);

        self.data
            .as_ref()
            .map(|data| build_export_data(data.clone(), span_context, tracer))
    }

    /// Returns the `SpanId` of the parent span.
    ///
    /// Returns `SpanId::INVALID` if the span has no parent or is not recording.
    pub fn parent_span_id(&self) -> SpanId {
        self.data
            .as_ref()
            .map(|data| data.parent_span_id)
            .unwrap_or(SpanId::INVALID)
    }

    /// Returns the `SpanKind` of the span.
    ///
    /// Returns `SpanKind::Internal` if the span is not recording.
    pub fn span_kind(&self) -> &SpanKind {
        self.data
            .as_ref()
            .map(|data| &data.span_kind)
            .unwrap_or(&SpanKind::Internal)
    }

    /// Returns the name of the span.
    ///
    /// Returns `None` if the span is not recording.
    pub fn name(&self) -> Option<&str> {
        Some(&self.data.as_ref()?.name)
    }

    /// Returns the start time of the span.
    ///
    /// Returns `None` if the span is not recording.
    pub fn start_time(&self) -> Option<SystemTime> {
        self.data.as_ref().map(|data| data.start_time)
    }

    /// Returns the attributes of the span.
    ///
    /// Returns an empty slice if the span is not recording.
    pub fn attributes(&self) -> &[KeyValue] {
        self.data
            .as_ref()
            .map(|data| data.attributes.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the number of dropped attributes.
    pub fn dropped_attributes_count(&self) -> u32 {
        self.data
            .as_ref()
            .map(|data| data.dropped_attributes_count)
            .unwrap_or(0)
    }

    /// Returns the events associated to the span.
    ///
    /// Returns an empty slice if the span is not recording.
    pub fn events(&self) -> &[Event] {
        self.data
            .as_ref()
            .map(|data| data.events.events.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the number of dropped events.
    pub fn dropped_events_count(&self) -> u32 {
        self.data
            .as_ref()
            .map(|data| data.events.dropped_count)
            .unwrap_or(0)
    }

    /// Returns the span links associated to the span.
    ///
    /// Returns an empty slice if the span is not recording.
    pub fn links(&self) -> &[Link] {
        self.data
            .as_ref()
            .map(|data| data.links.links.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the number of dropped links.
    pub fn dropped_links_count(&self) -> u32 {
        self.data
            .as_ref()
            .map(|data| data.links.dropped_count)
            .unwrap_or(0)
    }

    /// Returns the status of the span.
    ///
    /// Returns `Status::Unset` if the span is not recording.
    pub fn status(&self) -> &Status {
        self.data
            .as_ref()
            .map(|data| &data.status)
            .unwrap_or(&Status::Unset)
    }

    /// Returns the instrumentation scope of the span.
    pub fn instrumentation_scope(&self) -> &opentelemetry::InstrumentationScope {
        self.tracer.instrumentation_scope()
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
        let span_events_limit = self.span_limits.max_events_per_span as usize;
        let event_attributes_limit = self.span_limits.max_attributes_per_event as usize;
        self.with_data(|data| {
            if data.events.len() < span_events_limit {
                let dropped_attributes_count =
                    attributes.len().saturating_sub(event_attributes_limit);
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
        self.data.is_some()
    }

    /// Sets a single `Attribute` where the attribute properties are passed as arguments.
    ///
    /// Note that the OpenTelemetry project documents certain ["standard
    /// attributes"](https://github.com/open-telemetry/opentelemetry-specification/tree/v0.5.0/specification/trace/semantic_conventions/README.md)
    /// that have prescribed semantic meanings.
    fn set_attribute(&mut self, attribute: KeyValue) {
        let span_attribute_limit = self.span_limits.max_attributes_per_span as usize;
        self.with_data(|data| {
            if data.attributes.len() < span_attribute_limit {
                data.attributes.push(attribute);
            } else {
                data.dropped_attributes_count += 1;
            }
        });
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
            if data.links.links.len() < span_links_limit {
                let dropped_attributes_count =
                    attributes.len().saturating_sub(link_attributes_limit);
                let mut attributes = attributes;
                attributes.truncate(link_attributes_limit);
                data.links.add_link(Link::new(
                    span_context,
                    attributes,
                    dropped_attributes_count as u32,
                ));
            } else {
                data.links.dropped_count += 1;
            }
        });
    }

    /// Finishes the span with given timestamp.
    fn end_with_timestamp(&mut self, timestamp: SystemTime) {
        if let Some(ref mut data) = self.data {
            data.end_time = timestamp;
            data.end_time_set = true;
        }
        self.end_and_export();
    }
}

impl Span {
    /// Exports the span to all registered processors.
    ///
    /// Takes ownership of span data, sets end time if not already set,
    /// and passes the finished span to each processor.
    fn end_and_export(&mut self) {
        // Take data first so Drop won't re-enter if provider is shut down
        let mut data = match self.data.take() {
            Some(data) => data,
            None => return,
        };

        let provider = self.tracer.provider();
        if provider.is_shutdown() {
            return;
        }
        // Short-circuit before cloning span context or building export data
        // if no processors are registered.
        let span_processors = provider.span_processors();
        if span_processors.is_empty() {
            return;
        }

        // Set end time to now if not explicitly set via end_with_timestamp
        if !data.end_time_set {
            data.end_time = opentelemetry::time::now();
        }

        // The span context is cloned rather than moved out of the span: the API
        // spec requires `Span::span_context()` to keep returning a valid context
        // after the span has ended.
        // https://opentelemetry.io/docs/specs/otel/trace/api/#get-context
        let span_context = self.span_context.clone();

        let mut span_data = Some(build_export_data(data, span_context, &self.tracer));

        if let Some((last, processors)) = span_processors.split_last() {
            for processor in processors {
                processor.on_end(FinishedSpan::borrowed(
                    span_data
                        .as_ref()
                        .expect("span_data present for borrowed processors"),
                ));
            }
            last.on_end(FinishedSpan::owned(&mut span_data));
        }
    }
}

impl Drop for Span {
    /// Report span on inner drop
    fn drop(&mut self) {
        self.end_and_export();
    }
}

/// Represents a finished span passed to a span processor.
///
/// Processors can read the span data without cloning via [`span_data`](FinishedSpan::span_data).
/// If ownership of the [`SpanData`](crate::trace::SpanData) is needed, call
/// [`into_owned`](FinishedSpan::into_owned). The last registered processor
/// receives an owned wrapper and can move the data by calling `into_owned()`;
/// earlier processors receive a borrowed wrapper and clone on `into_owned()`.
///
/// ```
/// use opentelemetry_sdk::trace::FinishedSpan;
/// fn on_end(span: FinishedSpan<'_>) {
///     // Read the span data without taking ownership
///     if span.span_data().name != "my_span" {
///         return;
///     }
///     // Take ownership of the span data (clones for borrowed, moves for owned)
///     let span_data = span.into_owned();
///     # let _ = span_data;
/// }
/// ```
pub struct FinishedSpan<'a> {
    inner: FinishedSpanInner<'a>,
}

enum FinishedSpanInner<'a> {
    Borrowed(&'a crate::trace::SpanData),
    Owned(&'a mut Option<crate::trace::SpanData>),
}

impl<'a> FinishedSpan<'a> {
    pub(crate) fn borrowed(span_data: &'a crate::trace::SpanData) -> Self {
        FinishedSpan {
            inner: FinishedSpanInner::Borrowed(span_data),
        }
    }

    pub(crate) fn owned(span_data: &'a mut Option<crate::trace::SpanData>) -> Self {
        FinishedSpan {
            inner: FinishedSpanInner::Owned(span_data),
        }
    }
}

#[cfg(any(feature = "testing", test))]
impl<'a> FinishedSpan<'a> {
    /// Creates a borrowed [`FinishedSpan`] for testing or benchmarking purposes.
    ///
    /// This is a public test/benchmark-only borrowed constructor available when
    /// the `testing` feature is enabled or during test compilation.
    pub fn from_ref(span_data: &'a crate::trace::SpanData) -> Self {
        Self::borrowed(span_data)
    }
}

impl FinishedSpan<'_> {
    /// Returns a clone-free immutable view of the finished span's data.
    pub fn span_data(&self) -> &crate::trace::SpanData {
        match &self.inner {
            FinishedSpanInner::Borrowed(data) => data,
            FinishedSpanInner::Owned(data) => {
                data.as_ref().expect("SpanData present in FinishedSpan")
            }
        }
    }

    /// Converts this finished span into an owned [`SpanData`](crate::trace::SpanData).
    ///
    /// If this `FinishedSpan` wraps borrowed data (e.g. for non-final processors),
    /// the data will be cloned. If it wraps owned data (for the last registered
    /// processor), the data is moved out with zero copies.
    pub fn into_owned(self) -> crate::trace::SpanData {
        match self.inner {
            FinishedSpanInner::Borrowed(data) => data.clone(),
            FinishedSpanInner::Owned(data) => {
                data.take().expect("SpanData present in FinishedSpan")
            }
        }
    }

    /// Creates a borrowed [`FinishedSpan`] view from `self`.
    ///
    /// This allows delegating/composite processors to pass borrowed views to
    /// preceding child processors while retaining ownership to forward to the
    /// final child processor.
    pub fn reborrow(&self) -> FinishedSpan<'_> {
        FinishedSpan::borrowed(self.span_data())
    }
}

impl std::fmt::Debug for FinishedSpan<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FinishedSpan")
            .field("span", self.span_data())
            .finish()
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
    use crate::trace::{SdkTracer, SpanEvents, SpanLinks, SpanProcessor};
    use opentelemetry::trace::{
        Span as _, SpanBuilder, SpanKind, TraceFlags, TraceId, Tracer, TracerProvider,
    };
    use std::sync::Arc;
    use std::time::Duration;
    use std::vec;

    fn init() -> (crate::trace::SdkTracer, SpanData) {
        let provider = crate::trace::SdkTracerProvider::default();
        let tracer = provider.tracer("opentelemetry");
        let data = SpanData {
            parent_span_id: SpanId::from(0),
            parent_span_is_remote: false,
            span_kind: SpanKind::Internal,
            name: "opentelemetry".into(),
            start_time: opentelemetry::time::now(),
            end_time: opentelemetry::time::now(),
            end_time_set: false,
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
    fn end_with_no_processors() {
        let provider = crate::trace::SdkTracerProvider::builder().build();
        let tracer = provider.tracer("test");
        let mut span = tracer.start("test_span");
        let before = span.span_context().clone();
        span.end();
        assert_eq!(span.span_context(), &before);
        assert!(!span.is_recording());
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
        // The API spec requires `span_context()` to keep returning a valid
        // context after the span has ended, so `end_and_export` must clone the
        // context rather than move it out.
        // https://opentelemetry.io/docs/specs/otel/trace/api/#get-context
        //
        // Uses a real tracer-created span rather than `create_span()`, which
        // starts from `empty_context()` and would therefore pass regardless of
        // what `end()` does to the context.
        let provider = crate::trace::SdkTracerProvider::builder()
            .with_simple_exporter(NoopSpanExporter::new())
            .build();
        let tracer = provider.tracer("test");
        let mut span = tracer.start("test_span");

        let before = span.span_context().clone();
        assert!(before.is_valid(), "span context should be valid before end");

        span.end();

        assert_eq!(
            span.span_context(),
            &before,
            "span context must remain valid and unchanged after end"
        );
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
            .data
            .clone()
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
            .data
            .clone()
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
            .data
            .clone()
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
            .data
            .clone()
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
            .data
            .clone()
            .expect("span data should not be empty as we already set it before")
            .events;
        let event_vec: Vec<_> = span_events.events;
        assert_eq!(event_vec.len(), DEFAULT_MAX_EVENT_PER_SPAN as usize);
    }

    fn make_test_span(tracer: &SdkTracer) -> Span {
        let mut span = tracer.start("test_span");
        span.set_attribute(KeyValue::new("k", "v"));
        span.add_event("test_event", vec![]);
        span
    }

    #[test]
    fn test_live_span_accessors_recording() {
        let provider = crate::trace::SdkTracerProvider::builder()
            .with_simple_exporter(NoopSpanExporter::new())
            .build();
        let tracer = provider.tracer("test");

        let span = make_test_span(&tracer);

        assert!(span.span_context().span_id() != SpanId::INVALID);
        assert_eq!(span.name(), Some("test_span"));
        assert!(span.start_time().is_some());
        assert_eq!(span.attributes(), &[KeyValue::new("k", "v")]);
        assert_eq!(span.dropped_attributes_count(), 0);
        assert_eq!(span.events().len(), 1);
        assert_eq!(span.events()[0].name, "test_event");
        assert_eq!(span.dropped_events_count(), 0);
    }

    #[test]
    fn test_live_span_accessors_non_recording() {
        use crate::trace::Sampler;

        let provider = crate::trace::SdkTracerProvider::builder()
            .with_sampler(Sampler::AlwaysOff)
            .with_simple_exporter(NoopSpanExporter::new())
            .build();
        let tracer = provider.tracer("test");

        let span = make_test_span(&tracer);

        assert!(span.span_context().span_id() != SpanId::INVALID);
        assert_eq!(span.name(), None);
        assert_eq!(span.span_kind(), &SpanKind::Internal);
        assert!(span.start_time().is_none());
        assert_eq!(span.attributes(), &[]);
        assert_eq!(span.dropped_attributes_count(), 0);
        assert_eq!(span.events().len(), 0);
        assert_eq!(span.dropped_events_count(), 0);
        assert_eq!(span.links().len(), 0);
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
    fn test_finished_span_borrowed_into_owned() {
        let span_data = crate::trace::SpanData {
            span_context: SpanContext::empty_context(),
            parent_span_id: SpanId::INVALID,
            parent_span_is_remote: false,
            span_kind: SpanKind::Internal,
            name: "test".into(),
            start_time: opentelemetry::time::now(),
            end_time: opentelemetry::time::now(),
            attributes: vec![KeyValue::new("k", "v")],
            dropped_attributes_count: 0,
            events: SpanEvents::default(),
            links: SpanLinks::default(),
            status: Status::Unset,
            instrumentation_scope: Default::default(),
        };

        let orig_addr = span_data.attributes.as_ptr() as usize;
        let finished = FinishedSpan::borrowed(&span_data);
        assert_eq!(finished.span_data().name, "test");
        assert_eq!(finished.span_data().attributes.as_ptr() as usize, orig_addr);

        let owned = finished.into_owned();
        assert_eq!(owned, span_data);
        assert_ne!(
            owned.attributes.as_ptr() as usize,
            orig_addr,
            "Borrowed::into_owned must clone into a distinct allocation"
        );
    }

    #[test]
    fn test_finished_span_owned_into_owned() {
        let span_data = crate::trace::SpanData {
            span_context: SpanContext::empty_context(),
            parent_span_id: SpanId::INVALID,
            parent_span_is_remote: false,
            span_kind: SpanKind::Internal,
            name: "test".into(),
            start_time: opentelemetry::time::now(),
            end_time: opentelemetry::time::now(),
            attributes: vec![KeyValue::new("k", "v")],
            dropped_attributes_count: 0,
            events: SpanEvents::default(),
            links: SpanLinks::default(),
            status: Status::Unset,
            instrumentation_scope: Default::default(),
        };

        let orig_addr = span_data.attributes.as_ptr() as usize;
        let mut slot = Some(span_data);
        let finished = FinishedSpan::owned(&mut slot);
        assert_eq!(finished.span_data().name, "test");
        assert_eq!(finished.span_data().attributes.as_ptr() as usize, orig_addr);

        let owned = finished.into_owned();
        assert_eq!(
            owned.attributes.as_ptr() as usize,
            orig_addr,
            "Owned::into_owned must preserve allocation identity"
        );
        assert!(
            slot.is_none(),
            "Owned::into_owned must empty the supplied slot"
        );
    }

    #[test]
    fn test_finished_span_reborrow() {
        let span_data = crate::trace::SpanData {
            span_context: SpanContext::empty_context(),
            parent_span_id: SpanId::INVALID,
            parent_span_is_remote: false,
            span_kind: SpanKind::Internal,
            name: "test".into(),
            start_time: opentelemetry::time::now(),
            end_time: opentelemetry::time::now(),
            attributes: vec![KeyValue::new("k", "v")],
            dropped_attributes_count: 0,
            events: SpanEvents::default(),
            links: SpanLinks::default(),
            status: Status::Unset,
            instrumentation_scope: Default::default(),
        };

        let orig_addr = span_data.attributes.as_ptr() as usize;
        let mut slot = Some(span_data);
        let finished = FinishedSpan::owned(&mut slot);

        let reborrowed = finished.reborrow();
        let cloned_from_reborrow = reborrowed.into_owned();
        assert_ne!(
            cloned_from_reborrow.attributes.as_ptr() as usize,
            orig_addr,
            "reborrowed into_owned must always clone"
        );
        assert_eq!(
            finished.span_data().attributes.as_ptr() as usize,
            orig_addr,
            "reborrow must not consume the original owned slot"
        );

        let owned = finished.into_owned();
        assert_eq!(
            owned.attributes.as_ptr() as usize,
            orig_addr,
            "final into_owned on original must still move zero-copy"
        );
        assert!(slot.is_none());
    }

    #[test]
    fn test_finished_span_debug() {
        let span_data = crate::trace::SpanData {
            span_context: SpanContext::empty_context(),
            parent_span_id: SpanId::INVALID,
            parent_span_is_remote: false,
            span_kind: SpanKind::Internal,
            name: "debug_test".into(),
            start_time: opentelemetry::time::now(),
            end_time: opentelemetry::time::now(),
            attributes: vec![],
            dropped_attributes_count: 0,
            events: SpanEvents::default(),
            links: SpanLinks::default(),
            status: Status::Unset,
            instrumentation_scope: Default::default(),
        };

        let finished = FinishedSpan::borrowed(&span_data);
        let debug_str = format!("{finished:?}");
        assert!(debug_str.contains("FinishedSpan"));
        assert!(debug_str.contains("debug_test"));
    }

    #[test]
    fn test_zero_copy_multiple_processors() {
        #[derive(Debug, Clone, Default)]
        struct AddressCapturingProcessor {
            start_addr: Arc<std::sync::Mutex<Option<usize>>>,
            end_addr: Arc<std::sync::Mutex<Option<usize>>>,
        }
        impl SpanProcessor for AddressCapturingProcessor {
            fn on_start(&self, span: &mut Span, _cx: &opentelemetry::Context) {
                *self.start_addr.lock().unwrap() = Some(span.attributes().as_ptr() as usize);
            }
            fn on_end(&self, span: FinishedSpan<'_>) {
                let owned = span.into_owned();
                *self.end_addr.lock().unwrap() = Some(owned.attributes.as_ptr() as usize);
            }
            fn force_flush(&self) -> crate::error::OTelSdkResult {
                Ok(())
            }
            fn shutdown_with_timeout(&self, _timeout: Duration) -> crate::error::OTelSdkResult {
                Ok(())
            }
        }

        let first = AddressCapturingProcessor::default();
        let last = AddressCapturingProcessor::default();

        let provider = crate::trace::SdkTracerProvider::builder()
            .with_span_processor(first.clone())
            .with_span_processor(last.clone())
            .build();

        let tracer = provider.tracer("test");
        let span = tracer
            .span_builder("test_span")
            .with_attributes(vec![KeyValue::new("k", "v")])
            .start(&tracer);
        drop(span);
        provider.shutdown().unwrap();

        let initial_addr = first.start_addr.lock().unwrap().unwrap();
        assert_ne!(
            initial_addr, 0,
            "attribute vector must not be empty/dangling"
        );

        let first_end_addr = first.end_addr.lock().unwrap().unwrap();
        let last_end_addr = last.end_addr.lock().unwrap().unwrap();

        assert_ne!(
            first_end_addr, initial_addr,
            "first processor must receive a clone with a distinct vector allocation"
        );
        assert_eq!(
            last_end_addr, initial_addr,
            "last processor must receive the original vector allocation zero-copy"
        );
    }

    #[test]
    fn test_zero_copy_single_processor() {
        #[derive(Debug, Clone, Default)]
        struct SingleAddressProcessor {
            start_addr: Arc<std::sync::Mutex<Option<usize>>>,
            end_addr: Arc<std::sync::Mutex<Option<usize>>>,
        }
        impl SpanProcessor for SingleAddressProcessor {
            fn on_start(&self, span: &mut Span, _cx: &opentelemetry::Context) {
                *self.start_addr.lock().unwrap() = Some(span.attributes().as_ptr() as usize);
            }
            fn on_end(&self, span: FinishedSpan<'_>) {
                let owned = span.into_owned();
                *self.end_addr.lock().unwrap() = Some(owned.attributes.as_ptr() as usize);
            }
            fn force_flush(&self) -> crate::error::OTelSdkResult {
                Ok(())
            }
            fn shutdown_with_timeout(&self, _timeout: Duration) -> crate::error::OTelSdkResult {
                Ok(())
            }
        }

        let proc = SingleAddressProcessor::default();
        let provider = crate::trace::SdkTracerProvider::builder()
            .with_span_processor(proc.clone())
            .build();

        let tracer = provider.tracer("test");
        let span = tracer
            .span_builder("single_test_span")
            .with_attributes(vec![KeyValue::new("k", "v")])
            .start(&tracer);
        drop(span);
        provider.shutdown().unwrap();

        let initial_addr = proc.start_addr.lock().unwrap().unwrap();
        assert_ne!(initial_addr, 0);
        let end_addr = proc.end_addr.lock().unwrap().unwrap();

        assert_eq!(
            end_addr, initial_addr,
            "single processor must receive the original vector allocation zero-copy"
        );
    }

    #[test]
    fn test_readonly_preceding_processor_observes_complete_data() {
        #[derive(Debug, Clone, Default)]
        struct ReadOnlyProcessor {
            observed_name: Arc<std::sync::Mutex<Option<String>>>,
            observed_attributes_len: Arc<std::sync::Mutex<Option<usize>>>,
        }
        impl SpanProcessor for ReadOnlyProcessor {
            fn on_start(&self, _span: &mut Span, _cx: &opentelemetry::Context) {}
            fn on_end(&self, span: FinishedSpan<'_>) {
                *self.observed_name.lock().unwrap() = Some(span.span_data().name.to_string());
                *self.observed_attributes_len.lock().unwrap() =
                    Some(span.span_data().attributes.len());
                // Does NOT call into_owned()
            }
            fn force_flush(&self) -> crate::error::OTelSdkResult {
                Ok(())
            }
            fn shutdown_with_timeout(&self, _timeout: Duration) -> crate::error::OTelSdkResult {
                Ok(())
            }
        }

        #[derive(Debug, Clone, Default)]
        struct OwnershipProcessor {
            captured_addr: Arc<std::sync::Mutex<Option<usize>>>,
            initial_addr: Arc<std::sync::Mutex<Option<usize>>>,
        }
        impl SpanProcessor for OwnershipProcessor {
            fn on_start(&self, span: &mut Span, _cx: &opentelemetry::Context) {
                *self.initial_addr.lock().unwrap() = Some(span.attributes().as_ptr() as usize);
            }
            fn on_end(&self, span: FinishedSpan<'_>) {
                let owned = span.into_owned();
                *self.captured_addr.lock().unwrap() = Some(owned.attributes.as_ptr() as usize);
            }
            fn force_flush(&self) -> crate::error::OTelSdkResult {
                Ok(())
            }
            fn shutdown_with_timeout(&self, _timeout: Duration) -> crate::error::OTelSdkResult {
                Ok(())
            }
        }

        let readonly = ReadOnlyProcessor::default();
        let ownership = OwnershipProcessor::default();

        let provider = crate::trace::SdkTracerProvider::builder()
            .with_span_processor(readonly.clone())
            .with_span_processor(ownership.clone())
            .build();

        let tracer = provider.tracer("test");
        let span = tracer
            .span_builder("my_span")
            .with_attributes(vec![KeyValue::new("key", "val")])
            .start(&tracer);
        drop(span);
        provider.shutdown().unwrap();

        assert_eq!(
            readonly.observed_name.lock().unwrap().as_deref(),
            Some("my_span")
        );
        assert_eq!(*readonly.observed_attributes_len.lock().unwrap(), Some(1));

        let initial_addr = ownership.initial_addr.lock().unwrap().unwrap();
        let captured_addr = ownership.captured_addr.lock().unwrap().unwrap();
        assert_eq!(
            captured_addr, initial_addr,
            "last registered processor receives original zero-copy even when preceding processor only reads"
        );
    }

    #[test]
    fn test_composite_delegating_processor() {
        #[derive(Debug)]
        struct CompositeProcessor {
            children: Vec<Box<dyn SpanProcessor>>,
        }
        impl SpanProcessor for CompositeProcessor {
            fn on_start(&self, span: &mut Span, cx: &opentelemetry::Context) {
                for child in &self.children {
                    child.on_start(span, cx);
                }
            }
            fn on_end(&self, span: FinishedSpan<'_>) {
                if let Some((last, predecessors)) = self.children.split_last() {
                    for child in predecessors {
                        child.on_end(span.reborrow());
                    }
                    last.on_end(span);
                }
            }
            fn force_flush(&self) -> crate::error::OTelSdkResult {
                Ok(())
            }
            fn shutdown_with_timeout(&self, _timeout: Duration) -> crate::error::OTelSdkResult {
                Ok(())
            }
        }

        #[derive(Debug, Clone, Default)]
        struct AddressRecorder {
            start_addr: Arc<std::sync::Mutex<Option<usize>>>,
            end_addr: Arc<std::sync::Mutex<Option<usize>>>,
        }
        impl SpanProcessor for AddressRecorder {
            fn on_start(&self, span: &mut Span, _cx: &opentelemetry::Context) {
                *self.start_addr.lock().unwrap() = Some(span.attributes().as_ptr() as usize);
            }
            fn on_end(&self, span: FinishedSpan<'_>) {
                let owned = span.into_owned();
                *self.end_addr.lock().unwrap() = Some(owned.attributes.as_ptr() as usize);
            }
            fn force_flush(&self) -> crate::error::OTelSdkResult {
                Ok(())
            }
            fn shutdown_with_timeout(&self, _timeout: Duration) -> crate::error::OTelSdkResult {
                Ok(())
            }
        }

        let child1 = AddressRecorder::default();
        let child2 = AddressRecorder::default();

        let composite = CompositeProcessor {
            children: vec![Box::new(child1.clone()), Box::new(child2.clone())],
        };

        let provider = crate::trace::SdkTracerProvider::builder()
            .with_span_processor(composite)
            .build();

        let tracer = provider.tracer("test");
        let span = tracer
            .span_builder("composite_span")
            .with_attributes(vec![KeyValue::new("composite_key", "value")])
            .start(&tracer);
        drop(span);
        provider.shutdown().unwrap();

        let initial_addr = child1.start_addr.lock().unwrap().unwrap();
        assert_ne!(initial_addr, 0);

        let child1_end = child1.end_addr.lock().unwrap().unwrap();
        let child2_end = child2.end_addr.lock().unwrap().unwrap();

        assert_ne!(
            child1_end, initial_addr,
            "child1 (reborrowed) must receive clone"
        );
        assert_eq!(
            child2_end, initial_addr,
            "child2 (final) must receive original zero-copy"
        );
    }

    #[test]
    fn test_live_span_read_and_mutate_in_on_start() {
        #[derive(Debug, Clone, Default)]
        struct InspectAndMutateProcessor {
            seen_name: Arc<std::sync::Mutex<Option<String>>>,
            seen_kind: Arc<std::sync::Mutex<Option<SpanKind>>>,
            seen_parent_id: Arc<std::sync::Mutex<Option<SpanId>>>,
            seen_start_time: Arc<std::sync::Mutex<Option<SystemTime>>>,
            seen_attributes_count: Arc<std::sync::Mutex<Option<usize>>>,
            seen_dropped_attributes: Arc<std::sync::Mutex<Option<u32>>>,
            seen_events_count: Arc<std::sync::Mutex<Option<usize>>>,
            seen_dropped_events: Arc<std::sync::Mutex<Option<u32>>>,
            seen_links_count: Arc<std::sync::Mutex<Option<usize>>>,
            seen_dropped_links: Arc<std::sync::Mutex<Option<u32>>>,
            seen_status: Arc<std::sync::Mutex<Option<Status>>>,
            seen_scope: Arc<std::sync::Mutex<Option<String>>>,
            end_has_mutated: Arc<std::sync::Mutex<bool>>,
        }

        impl SpanProcessor for InspectAndMutateProcessor {
            fn on_start(&self, span: &mut Span, _cx: &opentelemetry::Context) {
                // Verify clone-free inherent read methods
                *self.seen_name.lock().unwrap() = span.name().map(String::from);
                *self.seen_kind.lock().unwrap() = Some(span.span_kind().clone());
                *self.seen_parent_id.lock().unwrap() = Some(span.parent_span_id());
                *self.seen_start_time.lock().unwrap() = span.start_time();
                *self.seen_attributes_count.lock().unwrap() = Some(span.attributes().len());
                *self.seen_dropped_attributes.lock().unwrap() =
                    Some(span.dropped_attributes_count());
                *self.seen_events_count.lock().unwrap() = Some(span.events().len());
                *self.seen_dropped_events.lock().unwrap() = Some(span.dropped_events_count());
                *self.seen_links_count.lock().unwrap() = Some(span.links().len());
                *self.seen_dropped_links.lock().unwrap() = Some(span.dropped_links_count());
                *self.seen_status.lock().unwrap() = Some(span.status().clone());
                *self.seen_scope.lock().unwrap() =
                    Some(span.instrumentation_scope().name().to_string());

                // Mutate live span
                span.set_attribute(KeyValue::new("added_in_start", "yes"));
                // Immediately read mutated attributes
                assert!(span
                    .attributes()
                    .iter()
                    .any(|kv| kv.key.as_str() == "added_in_start"));
            }

            fn on_end(&self, span: FinishedSpan<'_>) {
                let data = span.span_data();
                *self.end_has_mutated.lock().unwrap() = data
                    .attributes
                    .iter()
                    .any(|kv| kv.key.as_str() == "added_in_start");
            }

            fn force_flush(&self) -> crate::error::OTelSdkResult {
                Ok(())
            }
            fn shutdown_with_timeout(&self, _timeout: Duration) -> crate::error::OTelSdkResult {
                Ok(())
            }
        }

        let proc = InspectAndMutateProcessor::default();
        let provider = crate::trace::SdkTracerProvider::builder()
            .with_span_processor(proc.clone())
            .build();

        let tracer = provider.tracer("test_scope");
        let mut span = tracer.start("inspect_span");
        span.set_attribute(KeyValue::new("initial_key", "initial_val"));
        drop(span);
        provider.shutdown().unwrap();

        assert_eq!(
            proc.seen_name.lock().unwrap().as_deref(),
            Some("inspect_span")
        );
        assert_eq!(*proc.seen_kind.lock().unwrap(), Some(SpanKind::Internal));
        assert_eq!(*proc.seen_parent_id.lock().unwrap(), Some(SpanId::INVALID));
        assert!(proc.seen_start_time.lock().unwrap().is_some());
        assert_eq!(*proc.seen_attributes_count.lock().unwrap(), Some(0)); // before start mutated
        assert_eq!(*proc.seen_dropped_attributes.lock().unwrap(), Some(0));
        assert_eq!(*proc.seen_events_count.lock().unwrap(), Some(0));
        assert_eq!(*proc.seen_dropped_events.lock().unwrap(), Some(0));
        assert_eq!(*proc.seen_links_count.lock().unwrap(), Some(0));
        assert_eq!(*proc.seen_dropped_links.lock().unwrap(), Some(0));
        assert_eq!(*proc.seen_status.lock().unwrap(), Some(Status::Unset));
        assert_eq!(
            proc.seen_scope.lock().unwrap().as_deref(),
            Some("test_scope")
        );
        assert!(
            *proc.end_has_mutated.lock().unwrap(),
            "mutation in on_start must persist to on_end"
        );
    }

    #[test]
    fn test_end_with_timestamp_equal_to_start_time() {
        use crate::trace::InMemorySpanExporter;

        let exporter = InMemorySpanExporter::default();
        let provider = crate::trace::SdkTracerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();
        let tracer = provider.tracer("test");

        let mut span = tracer.start("test_span");
        let start_time = span.data.as_ref().unwrap().start_time;

        // Ensure the refactored end path preserves an explicitly supplied
        // zero-duration timestamp where end_time == start_time.
        span.end_with_timestamp(start_time);

        let spans = exporter.get_finished_spans().unwrap();
        assert_eq!(spans.len(), 1);
        assert_eq!(
            spans[0].end_time, start_time,
            "end_with_timestamp should preserve the explicit timestamp even when equal to start_time"
        );
    }
}
