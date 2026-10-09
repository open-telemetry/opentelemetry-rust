use crate::growable_array::GrowableArray;
#[cfg(feature = "trace")]
use opentelemetry::trace::SpanContext;
use opentelemetry::{
    logs::{AnyValue, Severity},
    Key, SpanId, TraceFlags, TraceId,
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

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
/// LogRecord represents all data carried by a log record, and
/// is provided to `LogExporter`s as input.
pub struct SdkLogRecord {
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

impl opentelemetry::logs::LogRecord for SdkLogRecord {
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

impl SdkLogRecord {
    /// Crate only default constructor
    pub(crate) fn new() -> Self {
        SdkLogRecord {
            event_name: None,
            target: None,
            timestamp: None,
            observed_timestamp: None,
            trace_context: None,
            severity_text: None,
            severity_number: None,
            body: None,
            attributes: LogRecordAttributes::default(),
        }
    }

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

    /// Sets the value of the attribute with the specified key.
    ///
    /// This method searches the `attributes` collection for the given key:
    ///
    /// - If the key is found, the first occurrence gets the new value, and all other
    ///   occurrences of the key are removed. After the call, the key occurs exactly once.
    /// - If the key is not found, the attribute is added.
    ///
    /// A `LogProcessor` can use this method to redact a value: no copy of the old value
    /// stays on the record.
    ///
    /// This method examines all attributes on each call, because it must remove the
    /// duplicates. Use `add_attribute` if the key is not on the record yet.
    ///
    /// # Arguments
    ///
    /// - `key`: The key of the attribute to set.
    /// - `value`: The new value for the attribute.
    ///
    /// # Returns
    ///
    /// - `Some(AnyValue)`: The old value of the first occurrence, if the key was found.
    /// - `None`: If the key was not found, and a new attribute was added.
    ///
    pub fn set_attribute<K, V>(&mut self, key: K, value: V) -> Option<AnyValue>
    where
        K: Into<Key>,
        V: Into<AnyValue>,
    {
        let key = key.into();
        let mut new_value = Some(value.into());
        let mut old_value = None;

        self.attributes.retain_mut(|attr| match attr {
            Some((k, v)) if *k == key => match new_value.take() {
                Some(new_value) => {
                    old_value = Some(std::mem::replace(v, new_value));
                    true
                }
                None => false,
            },
            _ => true,
        });

        if let Some(new_value) = new_value {
            self.attributes.push(Some((key, new_value)));
        }
        old_value
    }

    /// Removes all occurrences of the attribute with the specified key.
    ///
    /// This method searches for all occurrences of the attribute with the given key
    /// in the `attributes` collection and removes them. The order of the other
    /// attributes does not change.
    ///
    /// This method examines all attributes on each call.
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
        self.attributes
            .retain_mut(|attr| !matches!(attr, Some((k, _)) if k == key))
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

#[cfg(feature = "trace")]
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
    use opentelemetry::logs::{AnyValue, LogRecord as _, Severity};
    use opentelemetry::time::now;
    use std::borrow::Cow;

    #[test]
    fn test_set_eventname() {
        let mut log_record = SdkLogRecord::new();
        log_record.set_event_name("test_event");
        assert_eq!(log_record.event_name, Some("test_event"));
    }

    #[test]
    fn test_set_target() {
        let mut log_record = SdkLogRecord::new();
        log_record.set_target("foo::bar");
        assert_eq!(log_record.target, Some(Cow::Borrowed("foo::bar")));
    }

    #[test]
    fn test_set_timestamp() {
        let mut log_record = SdkLogRecord::new();
        let now = now();
        log_record.set_timestamp(now);
        assert_eq!(log_record.timestamp, Some(now));
    }

    #[test]
    fn test_set_observed_timestamp() {
        let mut log_record = SdkLogRecord::new();
        let now = now();
        log_record.set_observed_timestamp(now);
        assert_eq!(log_record.observed_timestamp, Some(now));
    }

    #[test]
    fn test_set_severity_text() {
        let mut log_record = SdkLogRecord::new();
        log_record.set_severity_text("ERROR");
        assert_eq!(log_record.severity_text, Some("ERROR"));
    }

    #[test]
    fn test_set_severity_number() {
        let mut log_record = SdkLogRecord::new();
        let severity_number = Severity::Error;
        log_record.set_severity_number(severity_number);
        assert_eq!(log_record.severity_number, Some(Severity::Error));
    }

    #[test]
    fn test_set_body() {
        let mut log_record = SdkLogRecord::new();
        let body = AnyValue::String("Test body".into());
        log_record.set_body(body.clone());
        assert_eq!(log_record.body, Some(body));
    }

    #[test]
    fn test_set_attributes() {
        let mut log_record = SdkLogRecord::new();
        let attributes = vec![(Key::new("key"), AnyValue::String("value".into()))];
        log_record.add_attributes(attributes.clone());
        for (key, value) in attributes {
            assert!(log_record.attributes_contains(&key, &value));
        }
    }

    #[test]
    fn test_set_attribute() {
        let mut log_record = SdkLogRecord::new();
        log_record.add_attribute("key", "value");
        let key = Key::new("key");
        let value = AnyValue::String("value".into());
        assert!(log_record.attributes_contains(&key, &value));
    }

    #[test]
    fn compare_trace_context() {
        let trace_context = TraceContext {
            trace_id: TraceId::from(1),
            span_id: SpanId::from(1),
            trace_flags: Some(TraceFlags::default()),
        };

        let trace_context_cloned = trace_context.clone();

        assert_eq!(trace_context, trace_context_cloned);

        let trace_context_different = TraceContext {
            trace_id: TraceId::from(2),
            span_id: SpanId::from(2),
            trace_flags: Some(TraceFlags::default()),
        };

        assert_ne!(trace_context, trace_context_different);
    }

    #[test]
    fn compare_log_record() {
        let mut log_record = SdkLogRecord {
            event_name: Some("test_event"),
            target: Some(Cow::Borrowed("foo::bar")),
            timestamp: Some(now()),
            observed_timestamp: Some(now()),
            severity_text: Some("ERROR"),
            severity_number: Some(Severity::Error),
            body: Some(AnyValue::String("Test body".into())),
            attributes: LogRecordAttributes::new(),
            trace_context: Some(TraceContext {
                trace_id: TraceId::from(1),
                span_id: SpanId::from(1),
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
        let log_record_borrowed = SdkLogRecord {
            event_name: Some("test_event"),
            ..SdkLogRecord::new()
        };

        let log_record_owned = SdkLogRecord {
            event_name: Some("test_event"),
            ..SdkLogRecord::new()
        };

        assert_eq!(log_record_borrowed, log_record_owned);
    }

    #[test]
    fn test_update_attribute() {
        let mut log_record = SdkLogRecord::new();
        let key = Key::new("key1");
        let value = AnyValue::String("value1".into());
        let updated_value = AnyValue::String("updated_value".into());

        // Add a new attribute
        assert!(log_record
            .set_attribute(key.clone(), value.clone())
            .is_none());
        assert!(log_record.attributes_contains(&key, &value));

        // Update the existing attribute
        assert_eq!(
            log_record.set_attribute(key.clone(), updated_value.clone()),
            Some(value)
        );
        assert!(log_record.attributes_contains(&key, &updated_value));
    }

    #[test]
    fn test_delete_attribute() {
        let mut log_record = SdkLogRecord::new();
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

    #[test]
    fn test_set_attribute_removes_duplicates() {
        let mut log_record = SdkLogRecord::new();
        let key = Key::new("secret");
        // Put the duplicates in the inline array and in the overflow vector.
        for i in 0..PREALLOCATED_ATTRIBUTE_CAPACITY + 3 {
            if i % 3 == 0 {
                log_record.add_attribute(key.clone(), format!("secret-{i}"));
            } else {
                log_record.add_attribute(format!("key{i}"), i as i64);
            }
        }

        // Redact the attribute, the first occurrence returns its old value
        assert_eq!(
            log_record.set_attribute(key.clone(), "redacted"),
            Some(AnyValue::from("secret-0"))
        );

        // Ensure only the redacted value is left and the duplicates are removed
        let matches: Vec<_> = log_record
            .attributes_iter()
            .filter(|(k, _)| k == &key)
            .collect();
        assert_eq!(matches, vec![&(key.clone(), AnyValue::from("redacted"))]);
        assert_eq!(log_record.attributes_len(), 6);
    }

    #[test]
    fn test_remove_attribute_duplicates() {
        let mut log_record = SdkLogRecord::new();
        let key = Key::new("key1");
        let other = Key::new("other");

        // Add attributes in the inline array and in the overflow vector
        for i in 0..PREALLOCATED_ATTRIBUTE_CAPACITY + 3 {
            if i % 2 == 0 {
                log_record.add_attribute(key.clone(), i as i64);
            } else {
                log_record.add_attribute(other.clone(), i as i64);
            }
        }

        // Delete all occurrences of the attribute
        assert_eq!(log_record.remove_attribute(&key), 4);

        // Ensure it is deleted and the other attributes stay
        assert!(log_record.attributes_iter().all(|(k, _)| k == &other));
        assert_eq!(log_record.attributes_len(), 4);

        // Delete it again, nothing is removed
        assert_eq!(log_record.remove_attribute(&key), 0);
        assert_eq!(log_record.attributes_len(), 4);
    }
}
