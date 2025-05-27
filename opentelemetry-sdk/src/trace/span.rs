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

    /// Convert information in this span into `exporter::trace::SpanData`.
    /// This function copies all data from the current span, which will create a
    /// overhead.
    pub fn exported_data(&self) -> Option<crate::trace::SpanData> {
        let (span_context, tracer) = (self.span_context.clone(), &self.tracer);

        self.data
            .as_ref()
            .map(|data| build_export_data(data.clone(), span_context, tracer))
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
        if let Some(data) = self.data.as_mut() {
            data.end_time = timestamp;
        }
        self.ensure_ended_and_exported();
    }
}

impl Span {
    /// Span ending logic
    ///
    /// The end timestamp of the span has to be set before calling this function
    fn ensure_ended_and_exported(&mut self) {
        if self.tracer.provider().is_shutdown() {
            return;
        }

        #[cfg(feature = "experimental_span_processor_on_ending")]
        {
            let provider = self.tracer.provider().clone();
            for processor in provider.span_processors() {
                processor.on_ending(self);
            }
        }

        let Span {
            data,
            tracer,
            span_context,
            ..
        } = self;

        // skip if data has already been exported
        let data = match data.take() {
            Some(data) => data,
            None => return,
        };
        let span_context: SpanContext =
            std::mem::replace(span_context, SpanContext::empty_context());

        let mut finished_span = FinishedSpan::new(build_export_data(data, span_context, tracer));

        let span_processors = tracer.provider().span_processors();
        for (i, processor) in span_processors.iter().enumerate() {
            finished_span.reset(i == span_processors.len() - 1);
            processor.on_end(&mut finished_span);
        }
    }
}

impl Drop for Span {
    /// Report span on inner drop
    fn drop(&mut self) {
        if let Some(ref mut data) = self.data {
            if data.end_time == data.start_time {
                data.end_time = opentelemetry::time::now();
            }
        }
        self.ensure_ended_and_exported();
    }
}

/// Represents a finished span passed to a span processor.
///
/// The data associated with the span is not writable, but it can be read
/// through the `ReadableSpan` trait.
///
/// Taking ownership of the span data is done by calling `consume`.
/// If `consume`` is never called, the on_ending method will not perform any copy of
/// the span data.
///
/// Calling any `ReadableSpan` method on the `FinishedSpan` will panic if the span data
/// has aready been consumed.
///
/// ```
/// use opentelemetry_sdk::trace::{FinishedSpan, ReadableSpan};
/// fn on_end(span: &mut FinishedSpan) {
///     // Read the span data without consuming it
///     if span.name() != Some("my_span") {
///         return;
///     }
///     // Consume the span data, potentially cloning it
///     let span = span.consume();
///     # let _ = span;
/// }
/// ```
pub struct FinishedSpan {
    span: Option<crate::trace::SpanData>,
    is_last_processor: bool,
    is_consumed: bool,
}

impl FinishedSpan {
    /// Creates a new `FinishedSpan` with the given span data.
    pub fn new(span_data: crate::trace::SpanData) -> Self {
        FinishedSpan {
            span: Some(span_data),
            is_last_processor: true,
            is_consumed: false,
        }
    }

    fn reset(&mut self, last_processor: bool) {
        self.is_last_processor = last_processor;
        self.is_consumed = false;
    }

    /// Takes ownership of the span data in the `FinishedSpan`.
    ///
    /// # Panics
    ///
    /// This function panics
    /// * if it called twice in the same SpanProcessor::on_end
    pub fn consume(&mut self) -> crate::trace::SpanData {
        if self.is_consumed {
            opentelemetry::otel_error!(name: "FinishedSpan.ConsumeTwice", message = "consume called twice on FinishedSpan in the same span processor");
        }
        self.try_consume()
            .expect("Span data has already been consumed")
    }

    /// Takes ownership of the span data in the `FinishedSpan`.
    ///
    /// Returns `None` if the span data has already been consumed.
    pub fn try_consume(&mut self) -> Option<crate::trace::SpanData> {
        if self.is_consumed {
            return None;
        }
        self.is_consumed = true;
        if self.is_last_processor {
            self.span.take()
        } else {
            self.span.clone()
        }
    }
}

impl ReadableSpan for FinishedSpan {
    fn context(&self) -> &SpanContext {
        match self.span {
            Some(ref data) => &data.span_context,
            None => &SpanContext::NONE,
        }
    }

    fn parent_span_id(&self) -> SpanId {
        match self.span {
            Some(ref data) => data.parent_span_id,
            None => SpanId::INVALID,
        }
    }

    fn span_kind(&self) -> SpanKind {
        match self.span {
            Some(ref data) => data.span_kind.clone(),
            None => SpanKind::Internal,
        }
    }

    fn name(&self) -> Option<&str> {
        self.span.as_ref().map(|s| s.name.as_ref())
    }
    fn start_time(&self) -> Option<SystemTime> {
        self.span.as_ref().map(|s| s.start_time)
    }
    fn end_time(&self) -> Option<SystemTime> {
        self.span.as_ref().map(|s| s.end_time)
    }
    fn attributes(&self) -> &[KeyValue] {
        match self.span {
            Some(ref data) => data.attributes.as_slice(),
            None => &[],
        }
    }
    fn dropped_attributes_count(&self) -> u32 {
        match self.span {
            Some(ref data) => data.dropped_attributes_count,
            None => 0,
        }
    }
    fn events(&self) -> &[Event] {
        match self.span {
            Some(ref data) => data.events.events.as_slice(),
            None => &[],
        }
    }
    fn dropped_events_count(&self) -> u32 {
        match self.span {
            Some(ref data) => data.events.dropped_count,
            None => 0,
        }
    }
    fn links(&self) -> &[Link] {
        match self.span {
            Some(ref data) => data.links.links.as_slice(),
            None => &[],
        }
    }

    fn dropped_links_count(&self) -> u32 {
        match self.span {
            Some(ref data) => data.links.dropped_count,
            None => 0,
        }
    }
    fn status(&self) -> &Status {
        match self.span {
            Some(ref data) => &data.status,
            None => &Status::Unset,
        }
    }
}

impl std::fmt::Debug for FinishedSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = f.debug_struct("FinishedSpan");
        match &self.span {
            Some(s) if !self.is_consumed => fmt.field("span", s),
            _ => fmt.field("consumed", &self.is_consumed),
        };
        fmt.finish()
    }
}

/// A trait for reading span data.
pub trait ReadableSpan {
    /// Returns the `SpanContext` of the span.
    fn context(&self) -> &SpanContext;

    /// Returns the `SpanId` of the parent span.
    fn parent_span_id(&self) -> SpanId;

    /// Returns the `SpanKind` of the span.
    ///
    /// Returns `SpanKind::Internal` if the span is not recording.
    fn span_kind(&self) -> SpanKind;

    /// Returns the name of the span.
    ///
    /// Returns `None` if the span is not recording.
    fn name(&self) -> Option<&str>;

    /// Returns the start time of the span.
    ///
    /// Returns `None` if the span is not recording.
    fn start_time(&self) -> Option<SystemTime>;

    /// Returns the end time of the span.
    ///
    /// Returns `None` if
    /// * the span is not recording.
    /// * the span has not been ended.
    fn end_time(&self) -> Option<SystemTime>;

    /// Returns the attributes of the span.
    ///
    /// Returns an empty slice if the span is not recording.
    fn attributes(&self) -> &[KeyValue];

    /// Returns the number of dropped attributes.
    fn dropped_attributes_count(&self) -> u32;

    /// Returns the events associated to the span.
    ///
    /// Returns an empty slice if the span is not recording.
    fn events(&self) -> &[Event];

    /// Returns the number of dropped events.
    fn dropped_events_count(&self) -> u32;

    /// Returns the span links associated to the span.
    ///
    /// Returns an empty slice if the span is not recording.
    fn links(&self) -> &[Link];

    /// Returns the number of dropped links.
    fn dropped_links_count(&self) -> u32;

    /// Returns the status of the span.
    ///
    /// Returns `Status::Unset` if the span is not recording.
    fn status(&self) -> &Status;
}

impl ReadableSpan for Span {
    fn context(&self) -> &SpanContext {
        &self.span_context
    }

    fn parent_span_id(&self) -> SpanId {
        self.data
            .as_ref()
            .map(|data| data.parent_span_id)
            .unwrap_or_else(|| SpanId::INVALID)
    }

    fn span_kind(&self) -> SpanKind {
        self.data
            .as_ref()
            .map(|data| data.span_kind.clone())
            .unwrap_or(SpanKind::Internal)
    }

    /// Returns the name of the span.
    ///
    /// Returns `None` if the span is not recording.
    fn name(&self) -> Option<&str> {
        Some(&self.data.as_ref()?.name)
    }

    fn start_time(&self) -> Option<SystemTime> {
        self.data.as_ref().map(|data| data.start_time)
    }

    fn end_time(&self) -> Option<SystemTime> {
        self.data.as_ref().map(|data| data.end_time)
    }

    fn attributes(&self) -> &[KeyValue] {
        self.data
            .as_ref()
            .map(|data| data.attributes.as_slice())
            .unwrap_or(&[])
    }

    fn dropped_attributes_count(&self) -> u32 {
        self.data
            .as_ref()
            .map(|data| data.dropped_attributes_count)
            .unwrap_or(0)
    }

    fn events(&self) -> &[Event] {
        self.data
            .as_ref()
            .map(|data| data.events.events.as_slice())
            .unwrap_or(&[])
    }

    fn dropped_events_count(&self) -> u32 {
        self.data
            .as_ref()
            .map(|data| data.events.dropped_count)
            .unwrap_or(0)
    }

    fn links(&self) -> &[Link] {
        self.data
            .as_ref()
            .map(|data| data.links.links.as_slice())
            .unwrap_or(&[])
    }

    fn dropped_links_count(&self) -> u32 {
        self.data
            .as_ref()
            .map(|data| data.links.dropped_count)
            .unwrap_or(0)
    }

    fn status(&self) -> &Status {
        self.data
            .as_ref()
            .map(|data| &data.status)
            .unwrap_or(&Status::Unset)
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
        self, SamplingResult, Span as _, SpanBuilder, TraceFlags, TraceId, Tracer, TracerProvider,
    };
    use std::time::Duration;
    use std::vec;

    fn init() -> (crate::trace::SdkTracer, SpanData) {
        let provider = crate::trace::SdkTracerProvider::default();
        let tracer = provider.tracer("opentelemetry");
        let data = SpanData {
            parent_span_id: SpanId::from_u64(0),
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
            initial_attributes.push(KeyValue::new(format!("key {}", i), i.to_string()))
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
                .push(KeyValue::new(format!("key {}", i), i.to_string()))
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
            TraceId::from_u128(12),
            SpanId::from_u64(12),
            TraceFlags::default(),
            false,
            Default::default(),
        ));
        for i in 0..(DEFAULT_MAX_ATTRIBUTES_PER_LINK * 2) {
            link.attributes
                .push(KeyValue::new(format!("key {}", i), i.to_string()));
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
                TraceId::from_u128(12),
                SpanId::from_u64(12),
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
                TraceId::from_u128(12),
                SpanId::from_u64(12),
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

    fn make_test_span(tracer: &SdkTracer, sampling_decision: trace::SamplingDecision) -> Span {
        tracer
            .span_builder("test_span")
            .with_sampling_result(SamplingResult {
                decision: sampling_decision,
                attributes: vec![],
                trace_state: Default::default(),
            })
            .with_attributes(vec![KeyValue::new("k", "v")])
            .with_kind(SpanKind::Client)
            .with_events(vec![Event::with_name("test_event")])
            .with_links(vec![Link::with_context(SpanContext::new(
                TraceId::from_bytes((1234_u128).to_ne_bytes()),
                SpanId::from_bytes((5678_u64).to_ne_bytes()),
                Default::default(),
                false,
                Default::default(),
            ))])
            .with_span_id(SpanId::from_bytes((1337_u64).to_ne_bytes()))
            .start(tracer)
    }

    #[test]
    fn test_readable_span() {
        use super::ReadableSpan;

        let provider = crate::trace::SdkTracerProvider::builder()
            .with_simple_exporter(NoopSpanExporter::new())
            .build();
        let tracer = provider.tracer("test");
        {
            // ReadableSpan trait methods for recording span
            let span = make_test_span(&tracer, trace::SamplingDecision::RecordOnly);

            assert_eq!(
                span.context().span_id(),
                SpanId::from_bytes((1337_u64).to_ne_bytes())
            );

            assert_eq!(span.name(), Some("test_span"));
            assert_eq!(span.span_kind(), SpanKind::Client);
            assert!(span.start_time().is_some());
            assert!(span.end_time().is_some());
            assert_eq!(span.attributes(), &[KeyValue::new("k", "v")]);
            assert_eq!(span.dropped_attributes_count(), 0);
            assert_eq!(span.events().len(), 1);
            assert_eq!(span.events()[0].name, "test_event");
            assert_eq!(span.dropped_events_count(), 0);
            assert_eq!(span.links().len(), 1);
        }

        {
            // ReadableSpan trait methods for non-recording span
            let span = make_test_span(&tracer, trace::SamplingDecision::Drop);

            assert_eq!(
                span.context().span_id(),
                SpanId::from_bytes((1337_u64).to_ne_bytes())
            );

            assert_eq!(span.name(), None);
            assert_eq!(span.span_kind(), SpanKind::Internal);
            assert!(span.start_time().is_none());
            assert!(span.end_time().is_none());
            assert_eq!(span.attributes(), &[]);
            assert_eq!(span.dropped_attributes_count(), 0);
            assert_eq!(span.events().len(), 0);
            assert_eq!(span.dropped_events_count(), 0);
            assert_eq!(span.links().len(), 0);
            assert_eq!(
                span.context().span_id(),
                SpanId::from_bytes((1337_u64).to_ne_bytes())
            );
        }
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
        println!("{:?}", res);
        assert!(res.is_ok());
        let dropped_span = tracer.start("span_with_dropped_provider");
        // return none if the provider has already been dropped
        assert!(dropped_span.exported_data().is_none());
    }

    #[test]
    fn test_finished_span_consume() {
        use super::ReadableSpan;

        #[derive(Debug)]
        struct TestSpanProcessor;
        impl SpanProcessor for TestSpanProcessor {
            fn on_end(&self, span: &mut FinishedSpan) {
                assert_eq!(
                    span.context().span_id(),
                    SpanId::from_bytes((1337_u64).to_ne_bytes())
                );

                assert_eq!(span.name(), Some("test_span"));
                assert_eq!(span.span_kind(), SpanKind::Client);
                assert!(span.start_time().is_some());
                assert!(span.end_time().is_some());
                assert_eq!(span.attributes(), &[KeyValue::new("k", "v")]);
                assert_eq!(span.dropped_attributes_count(), 0);
                assert_eq!(span.events().len(), 1);
                assert_eq!(span.events()[0].name, "test_event");
                assert_eq!(span.dropped_events_count(), 0);
                assert_eq!(span.links().len(), 1);

                let _ = span.consume();
            }

            fn on_start(&self, _span: &mut Span, _cx: &opentelemetry::Context) {}

            fn force_flush(&self) -> crate::error::OTelSdkResult {
                Ok(())
            }

            fn shutdown_with_timeout(&self, _timeout: Duration) -> crate::error::OTelSdkResult {
                Ok(())
            }
        }

        let provider = crate::trace::SdkTracerProvider::builder()
            .with_span_processor(TestSpanProcessor)
            .build();
        drop(make_test_span(
            &provider.tracer("test"),
            trace::SamplingDecision::RecordAndSample,
        ));
        let res = provider.shutdown();
        println!("{:?}", res);
        assert!(res.is_ok());
    }

    #[test]
    fn test_finished_span_consume_twice() {
        #[derive(Debug)]
        struct TestSpanProcessor;
        impl SpanProcessor for TestSpanProcessor {
            fn on_end(&self, span: &mut FinishedSpan) {
                let _ = span.consume();
                assert!(span.try_consume().is_none());
            }

            fn on_start(&self, _span: &mut Span, _cx: &opentelemetry::Context) {}

            fn force_flush(&self) -> crate::error::OTelSdkResult {
                Ok(())
            }

            fn shutdown_with_timeout(&self, _timeout: Duration) -> crate::error::OTelSdkResult {
                Ok(())
            }
        }

        let provider = crate::trace::SdkTracerProvider::builder()
            .with_span_processor(TestSpanProcessor)
            .build();
        drop(make_test_span(
            &provider.tracer("test"),
            trace::SamplingDecision::RecordAndSample,
        ));

        let res = provider.shutdown();
        println!("{:?}", res);
        assert!(res.is_ok());
    }
}
