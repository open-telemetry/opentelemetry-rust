use crate::growable_array::GrowableArray;
use opentelemetry::logs::LogRecord as _;
use opentelemetry::{
    logs::{AnyValue, Severity},
    trace::{SpanContext, SpanId, TraceFlags, TraceId},
    Key,
};
use std::{borrow::Cow, time::SystemTime};

// According to a Go-specific study mentioned on https://go.dev/blog/slog,
// up to 5 attributes is the most common case.
const PREALLOCATED_ATTRIBUTE_CAPACITY: usize = 5;

/// Represents a collection of log record attributes with a predefined capacity.
///
/// This type uses `GrowableArray` to store key-value pairs of log attributes, where each attribute is an `Option<(Key, AnyValue)>`.
/// The initial attributes are allocated in a fixed-size array of capacity `PREALLOCATED_ATTRIBUTE_CAPACITY`.
/// If more attributes are added beyond this capacity, additional storage is handled by dynamically growing a vector.
pub(crate) type LogRecordAttributes =
    GrowableArray<Option<(Key, AnyValue)>, PREALLOCATED_ATTRIBUTE_CAPACITY>;

#[derive(Debug, Default, Clone, PartialEq)]
#[non_exhaustive]
/// LogRecord represents all data carried by a log record, and
/// is provided to `LogExporter`s as input.
pub struct LogRecord {
    /// Event name. Optional as not all the logging API support it.
    pub(crate) event_name: Option<&'static str>,

    /// Target of the log record
    pub(crate) target: Option<Cow<'static, str>>,

    /// Record timestamp
    pub(crate) timestamp: Option<SystemTime>,

    /// Timestamp for when the record was observed by OpenTelemetry
    pub(crate) observed_timestamp: Option<SystemTime>,

    /// Trace context for logs associated with spans
    pub(crate) trace_context: Option<TraceContext>,

    /// The original severity string from the source
    pub(crate) severity_text: Option<&'static str>,

    /// The corresponding severity value, normalized
    pub(crate) severity_number: Option<Severity>,

    /// Record body
    pub(crate) body: Option<AnyValue>,

    /// Additional attributes associated with this record
    pub(crate) attributes: LogRecordAttributes,
}

impl opentelemetry::logs::LogRecord for LogRecord {
    fn set_event_name(&mut self, name: &'static str) {
        self.event_name = Some(name);
    }

    // Sets the `target` of a record
    fn set_target<T>(&mut self, _target: T)
    where
        T: Into<Cow<'static, str>>,
    {
        self.target = Some(_target.into());
    }

    fn set_timestamp(&mut self, timestamp: SystemTime) {
        self.timestamp = Some(timestamp);
    }

    fn set_observed_timestamp(&mut self, timestamp: SystemTime) {
        self.observed_timestamp = Some(timestamp);
    }

    fn set_severity_text(&mut self, severity_text: &'static str) {
        self.severity_text = Some(severity_text);
    }

    fn set_severity_number(&mut self, severity_number: Severity) {
        self.severity_number = Some(severity_number);
    }

    fn set_body(&mut self, body: AnyValue) {
        self.body = Some(body);
    }

    fn add_attributes<I, K, V>(&mut self, attributes: I)
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<Key>,
        V: Into<AnyValue>,
    {
        for (key, value) in attributes.into_iter() {
            self.add_attribute(key, value);
        }
    }

    fn add_attribute<K, V>(&mut self, key: K, value: V)
    where
        K: Into<Key>,
        V: Into<AnyValue>,
    {
        self.attributes.push(Some((key.into(), value.into())));
    }

    fn set_trace_context(
        &mut self,
        trace_id: TraceId,
        span_id: SpanId,
        trace_flags: Option<TraceFlags>,
    ) {
        self.trace_context = Some(TraceContext {
            trace_id,
            span_id,
            trace_flags,
        });
    }
}

impl LogRecord {
    /// Returns the event name
    #[inline]
    pub fn event_name(&self) -> Option<&'static str> {
        self.event_name
    }

    /// Returns the target
    #[inline]
    pub fn target(&self) -> Option<&Cow<'static, str>> {
        self.target.as_ref()
    }

    /// Returns the timestamp
    #[inline]
    pub fn timestamp(&self) -> Option<SystemTime> {
        self.timestamp
    }

    /// Returns the observed timestamp
    #[inline]
    pub fn observed_timestamp(&self) -> Option<SystemTime> {
        self.observed_timestamp
    }

    /// Returns the trace context
    #[inline]
    pub fn trace_context(&self) -> Option<&TraceContext> {
        self.trace_context.as_ref()
    }

    /// Returns the severity text
    #[inline]
    pub fn severity_text(&self) -> Option<&'static str> {
        self.severity_text
    }

    /// Returns the severity number
    #[inline]
    pub fn severity_number(&self) -> Option<Severity> {
        self.severity_number
    }

    /// Returns the body
    #[inline]
    pub fn body(&self) -> Option<&AnyValue> {
        self.body.as_ref()
    }

    /// Provides an iterator over the attributes.
    #[inline]
    pub fn attributes_iter(&self) -> impl Iterator<Item = &(Key, AnyValue)> {
        self.attributes.iter().filter_map(|opt| opt.as_ref())
    }

    #[allow(dead_code)]
    /// Returns the number of attributes in the `LogRecord`.
    pub(crate) fn attributes_len(&self) -> usize {
        self.attributes.len()
    }

    #[allow(dead_code)]
    /// Checks if the `LogRecord` contains the specified attribute.
    pub(crate) fn attributes_contains(&self, key: &Key, value: &AnyValue) -> bool {
        self.attributes
            .iter()
            .flatten()
            .any(|(k, v)| k == key && v == value)
    }

    /// Updates the first occurrence of an attribute with the specified key.
    ///
    /// This method searches for the first occurrence of the attribute with the given key
    /// in the `attributes` collection. If the key is found, its value is updated with the
    /// provided value. If the key is not found, the attribute is added.
    ///
    /// # Arguments
    ///
    /// - `key`: A reference to the key of the attribute to update.
    /// - `value`: A new value for the attribute.
    ///
    /// # Returns
    ///
    /// - `Some(AnyValue)`: The old value of the attribute if found and updated.
    /// - `None`: If the attribute was not found, and a new one was added.
    ///
    pub fn update_attribute(&mut self, key: &Key, value: AnyValue) -> Option<AnyValue> {
        // First, search for the attribute mutably
        if let Some(attr) = self
            .attributes
            .iter_mut()
            .find(|opt| opt.as_ref().map(|(k, _)| k == key).unwrap_or(false))
        {
            // Take the old value and update the attribute
            let old_value = attr.take().map(|(_, v)| v);
            *attr = Some((key.clone(), value));
            return old_value;
        }

        // If not found, add a new attribute
        self.add_attribute(key.clone(), value.clone());
        None
    }

    /// Removes all occurrences of an attribute with the specified key.
    ///
    /// This method searches for all occurrences of the attribute with the given key
    /// in the `attributes` collection and removes them.
    ///
    /// # Arguments
    ///
    /// - `key`: A reference to the key of the attribute to remove.
    ///
    /// # Returns
    ///
    /// - The number of removed occurrences of the key.
    ///
    pub fn remove_attribute(&mut self, key: &Key) -> usize {
        let mut deleted_count = 0;

        // Loop to find and remove all occurrences
        while let Some(index) = {
            // Isolate the immutable borrow in a block scope
            let position = self
                .attributes
                .iter()
                .position(|opt| opt.as_ref().map(|(k, _)| k == key).unwrap_or(false));
            position
        } {
            // Now proceed with the mutable borrow and remove the item
            self.attributes.remove_at(index);
            deleted_count += 1;
        }

        deleted_count
    }
}

/// TraceContext stores the trace context for logs that have an associated
/// span.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct TraceContext {
    /// Trace id
    pub trace_id: TraceId,
    /// Span Id
    pub span_id: SpanId,
    /// Trace flags
    pub trace_flags: Option<TraceFlags>,
}

impl From<&SpanContext> for TraceContext {
    fn from(span_context: &SpanContext) -> Self {
        TraceContext {
            trace_id: span_context.trace_id(),
            span_id: span_context.span_id(),
            trace_flags: Some(span_context.trace_flags()),
        }
    }
}

#[cfg(all(test, feature = "testing"))]
mod tests {
    use super::*;
    use opentelemetry::logs::{AnyValue, Severity};
    use std::borrow::Cow;
    use std::time::SystemTime;

    #[test]
    fn test_set_eventname() {
        let mut log_record = LogRecord::default();
        log_record.set_event_name("test_event");
        assert_eq!(log_record.event_name, Some("test_event"));
    }

    #[test]
    fn test_set_target() {
        let mut log_record = LogRecord::default();
        log_record.set_target("foo::bar");
        assert_eq!(log_record.target, Some(Cow::Borrowed("foo::bar")));
    }

    #[test]
    fn test_set_timestamp() {
        let mut log_record = LogRecord::default();
        let now = SystemTime::now();
        log_record.set_timestamp(now);
        assert_eq!(log_record.timestamp, Some(now));
    }

    #[test]
    fn test_set_observed_timestamp() {
        let mut log_record = LogRecord::default();
        let now = SystemTime::now();
        log_record.set_observed_timestamp(now);
        assert_eq!(log_record.observed_timestamp, Some(now));
    }

    #[test]
    fn test_set_severity_text() {
        let mut log_record = LogRecord::default();
        log_record.set_severity_text("ERROR");
        assert_eq!(log_record.severity_text, Some("ERROR"));
    }

    #[test]
    fn test_set_severity_number() {
        let mut log_record = LogRecord::default();
        let severity_number = Severity::Error;
        log_record.set_severity_number(severity_number);
        assert_eq!(log_record.severity_number, Some(Severity::Error));
    }

    #[test]
    fn test_set_body() {
        let mut log_record = LogRecord::default();
        let body = AnyValue::String("Test body".into());
        log_record.set_body(body.clone());
        assert_eq!(log_record.body, Some(body));
    }

    #[test]
    fn test_set_attributes() {
        let mut log_record = LogRecord::default();
        let attributes = vec![(Key::new("key"), AnyValue::String("value".into()))];
        log_record.add_attributes(attributes.clone());
        for (key, value) in attributes {
            assert!(log_record.attributes_contains(&key, &value));
        }
    }

    #[test]
    fn test_set_attribute() {
        let mut log_record = LogRecord::default();
        log_record.add_attribute("key", "value");
        let key = Key::new("key");
        let value = AnyValue::String("value".into());
        assert!(log_record.attributes_contains(&key, &value));
    }

    #[test]
    fn compare_trace_context() {
        let trace_context = TraceContext {
            trace_id: TraceId::from_u128(1),
            span_id: SpanId::from_u64(1),
            trace_flags: Some(TraceFlags::default()),
        };

        let trace_context_cloned = trace_context.clone();

        assert_eq!(trace_context, trace_context_cloned);

        let trace_context_different = TraceContext {
            trace_id: TraceId::from_u128(2),
            span_id: SpanId::from_u64(2),
            trace_flags: Some(TraceFlags::default()),
        };

        assert_ne!(trace_context, trace_context_different);
    }

    #[test]
    fn compare_log_record() {
        let mut log_record = LogRecord {
            event_name: Some("test_event"),
            target: Some(Cow::Borrowed("foo::bar")),
            timestamp: Some(SystemTime::now()),
            observed_timestamp: Some(SystemTime::now()),
            severity_text: Some("ERROR"),
            severity_number: Some(Severity::Error),
            body: Some(AnyValue::String("Test body".into())),
            attributes: LogRecordAttributes::new(),
            trace_context: Some(TraceContext {
                trace_id: TraceId::from_u128(1),
                span_id: SpanId::from_u64(1),
                trace_flags: Some(TraceFlags::default()),
            }),
        };
        log_record.add_attribute(Key::new("key"), AnyValue::String("value".into()));

        let log_record_cloned = log_record.clone();

        assert_eq!(log_record, log_record_cloned);

        let mut log_record_different = log_record.clone();
        log_record_different.event_name = Some("different_event");

        assert_ne!(log_record, log_record_different);
    }

    #[test]
    fn compare_log_record_target_borrowed_eq_owned() {
        let log_record_borrowed = LogRecord {
            event_name: Some("test_event"),
            ..Default::default()
        };

        let log_record_owned = LogRecord {
            event_name: Some("test_event"),
            ..Default::default()
        };

        assert_eq!(log_record_borrowed, log_record_owned);
    }

    #[test]
    fn test_update_attribute() {
        let mut log_record = LogRecord::default();
        let key = Key::new("key1");
        let value = AnyValue::String("value1".into());
        let updated_value = AnyValue::String("updated_value".into());

        // Add a new attribute
        assert!(log_record.update_attribute(&key, value.clone()).is_none());
        assert!(log_record.attributes_contains(&key, &value));

        // Update the existing attribute
        assert_eq!(
            log_record.update_attribute(&key, updated_value.clone()),
            Some(value)
        );
        assert!(log_record.attributes_contains(&key, &updated_value));
    }

    #[test]
    fn test_delete_attribute() {
        let mut log_record = LogRecord::default();
        let key = Key::new("key1");
        let value = AnyValue::String("value1".into());

        // Add an attribute
        log_record.add_attribute(key.clone(), value.clone());
        assert!(log_record.attributes_contains(&key, &value));

        // Delete the attribute
        let del_count = log_record.remove_attribute(&key);
        assert_eq!(del_count, 1);

        // Ensure it is deleted
        assert!(!log_record.attributes_contains(&key, &value));
    }
}
