use crate::logs::SdkLogRecord;
use opentelemetry::logs::AnyValue;
use opentelemetry::logs::Severity;
use opentelemetry::{Array, InstrumentationScope, KeyValue, Value};
use std::collections::VecDeque;
use std::mem::size_of;
#[cfg(feature = "experimental_metrics_bound_instruments")]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, TryLockError};
use std::time::{Duration, Instant};

pub(crate) type BufferedLog = (SdkLogRecord, InstrumentationScope, usize);

#[derive(Clone, Copy)]
pub(crate) struct EvictionPlan {
    pub(crate) count: usize,
    pub(crate) bytes: usize,
}

pub(crate) struct FlightRecorderMetrics {
    #[cfg(feature = "experimental_metrics_bound_instruments")]
    buffered: opentelemetry::metrics::BoundCounter<u64>,
    #[cfg(feature = "experimental_metrics_bound_instruments")]
    evicted: opentelemetry::metrics::BoundCounter<u64>,
    #[cfg(feature = "experimental_metrics_bound_instruments")]
    replayed: opentelemetry::metrics::BoundCounter<u64>,
    #[cfg(feature = "experimental_metrics_bound_instruments")]
    oversized_dropped: opentelemetry::metrics::BoundCounter<u64>,
    #[cfg(feature = "experimental_metrics_bound_instruments")]
    oversized_exported: opentelemetry::metrics::BoundCounter<u64>,
    #[cfg(feature = "experimental_metrics_bound_instruments")]
    capacity_dropped: opentelemetry::metrics::BoundCounter<u64>,
    #[cfg(feature = "experimental_metrics_bound_instruments")]
    capacity_exported: opentelemetry::metrics::BoundCounter<u64>,
    #[cfg(feature = "experimental_metrics_bound_instruments")]
    scopes_admitted: opentelemetry::metrics::BoundCounter<u64>,
    #[cfg(feature = "experimental_metrics_bound_instruments")]
    scopes_rejected: opentelemetry::metrics::BoundCounter<u64>,
    #[cfg(feature = "experimental_metrics_bound_instruments")]
    scopes_discarded: opentelemetry::metrics::BoundCounter<u64>,
    #[cfg(feature = "experimental_metrics_bound_instruments")]
    triggered: opentelemetry::metrics::BoundCounter<u64>,
    #[cfg(feature = "experimental_metrics_bound_instruments")]
    handed_off: opentelemetry::metrics::BoundCounter<u64>,
}

impl FlightRecorderMetrics {
    pub(crate) fn new(component_type: &'static str) -> Self {
        #[cfg(feature = "experimental_metrics_bound_instruments")]
        {
            static INSTANCE_COUNTER: AtomicUsize = AtomicUsize::new(0);
            let instance_id = INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
            let component_name = format!("{component_type}/{instance_id}");
            let meter = opentelemetry::global::meter("otel.sdk");
            let records = meter
                .u64_counter("otel.sdk.processor.log.flight_recorder.records")
                .with_description("The number of log records handled by a flight recorder.")
                .with_unit("{log_record}")
                .build();
            let events = meter
                .u64_counter("otel.sdk.processor.log.flight_recorder.events")
                .with_description("The number of flight-recorder lifecycle events.")
                .with_unit("{event}")
                .build();
            let bind = |counter: &opentelemetry::metrics::Counter<u64>, action| {
                counter.bind(&[
                    KeyValue::new("action", action),
                    KeyValue::new("otel.component.type", component_type),
                    KeyValue::new("otel.component.name", component_name.clone()),
                ])
            };

            Self {
                buffered: bind(&records, "buffered"),
                evicted: bind(&records, "evicted"),
                replayed: bind(&records, "replayed"),
                oversized_dropped: bind(&records, "oversized_dropped"),
                oversized_exported: bind(&records, "oversized_exported"),
                capacity_dropped: bind(&records, "capacity_dropped"),
                capacity_exported: bind(&records, "capacity_exported"),
                scopes_admitted: bind(&events, "scope_admitted"),
                scopes_rejected: bind(&events, "scope_rejected"),
                scopes_discarded: bind(&events, "scope_discarded"),
                triggered: bind(&events, "triggered"),
                handed_off: bind(&events, "handed_off"),
            }
        }
        #[cfg(not(feature = "experimental_metrics_bound_instruments"))]
        {
            let _ = component_type;
            Self {}
        }
    }

    pub(crate) fn buffered(&self, count: usize) {
        #[cfg(feature = "experimental_metrics_bound_instruments")]
        self.buffered.add(u64::try_from(count).unwrap_or(u64::MAX));
        #[cfg(not(feature = "experimental_metrics_bound_instruments"))]
        let _ = count;
    }

    pub(crate) fn evicted(&self, count: usize) {
        #[cfg(feature = "experimental_metrics_bound_instruments")]
        self.evicted.add(u64::try_from(count).unwrap_or(u64::MAX));
        #[cfg(not(feature = "experimental_metrics_bound_instruments"))]
        let _ = count;
    }

    pub(crate) fn replayed(&self, count: usize) {
        #[cfg(feature = "experimental_metrics_bound_instruments")]
        self.replayed.add(u64::try_from(count).unwrap_or(u64::MAX));
        #[cfg(not(feature = "experimental_metrics_bound_instruments"))]
        let _ = count;
    }

    pub(crate) fn oversized(&self, exported: bool) {
        #[cfg(feature = "experimental_metrics_bound_instruments")]
        if exported {
            self.oversized_exported.add(1);
        } else {
            self.oversized_dropped.add(1);
        }
        #[cfg(not(feature = "experimental_metrics_bound_instruments"))]
        let _ = exported;
    }

    pub(crate) fn capacity_overflow(&self, exported: bool) {
        #[cfg(feature = "experimental_metrics_bound_instruments")]
        if exported {
            self.capacity_exported.add(1);
        } else {
            self.capacity_dropped.add(1);
        }
        #[cfg(not(feature = "experimental_metrics_bound_instruments"))]
        let _ = exported;
    }

    pub(crate) fn scope_admitted(&self) {
        #[cfg(feature = "experimental_metrics_bound_instruments")]
        self.scopes_admitted.add(1);
    }

    pub(crate) fn scope_rejected(&self) {
        #[cfg(feature = "experimental_metrics_bound_instruments")]
        self.scopes_rejected.add(1);
    }

    pub(crate) fn scope_discarded(&self) {
        #[cfg(feature = "experimental_metrics_bound_instruments")]
        self.scopes_discarded.add(1);
    }

    pub(crate) fn triggered(&self) {
        #[cfg(feature = "experimental_metrics_bound_instruments")]
        self.triggered.add(1);
    }

    pub(crate) fn handed_off(&self) {
        #[cfg(feature = "experimental_metrics_bound_instruments")]
        self.handed_off.add(1);
    }
}

pub(crate) struct LogBuffer {
    records: VecDeque<BufferedLog>,
    size_bytes: usize,
    max_records: usize,
    max_size_bytes: usize,
}

impl LogBuffer {
    pub(crate) fn new(max_records: usize, max_size_bytes: usize) -> Self {
        Self {
            records: VecDeque::new(),
            size_bytes: 0,
            max_records,
            max_size_bytes,
        }
    }

    pub(crate) fn plan_insertion(&self, size: usize) -> EvictionPlan {
        let mut remaining_records = self.records.len();
        let mut remaining_bytes = self.size_bytes;
        let mut plan = EvictionPlan { count: 0, bytes: 0 };
        for (_, _, record_size) in &self.records {
            if remaining_records < self.max_records && size <= self.max_size_bytes - remaining_bytes
            {
                break;
            }
            remaining_records -= 1;
            remaining_bytes -= record_size;
            plan.count += 1;
            plan.bytes += record_size;
        }
        plan
    }

    pub(crate) fn can_fit(&self, size: usize) -> bool {
        size <= self.max_size_bytes
    }

    pub(crate) fn insert(&mut self, log: BufferedLog, plan: EvictionPlan) {
        for _ in 0..plan.count {
            self.records.pop_front();
        }
        self.size_bytes = self.size_bytes - plan.bytes + log.2;
        self.records.push_back(log);
    }

    pub(crate) fn push_overwriting(&mut self, log: BufferedLog) -> EvictionPlan {
        let plan = self.plan_insertion(log.2);
        self.insert(log, plan);
        plan
    }

    pub(crate) fn take(&mut self) -> VecDeque<BufferedLog> {
        self.size_bytes = 0;
        std::mem::take(&mut self.records)
    }

    pub(crate) fn clear(&mut self) {
        self.records.clear();
        self.size_bytes = 0;
    }
}

pub(crate) fn should_buffer(record: &SdkLogRecord, max_severity: Severity) -> bool {
    record
        .severity_number()
        .map_or(true, |severity| severity <= max_severity)
}

pub(crate) enum TimedLockError {
    Poisoned,
    Timeout,
}

pub(crate) fn lock_with_timeout<T>(
    lock: &Mutex<T>,
    timeout: Duration,
) -> Result<MutexGuard<'_, T>, TimedLockError> {
    let start = Instant::now();
    loop {
        match lock.try_lock() {
            Ok(guard) => return Ok(guard),
            Err(TryLockError::Poisoned(_)) => return Err(TimedLockError::Poisoned),
            Err(TryLockError::WouldBlock) => {
                let Some(remaining) = timeout.checked_sub(start.elapsed()) else {
                    return Err(TimedLockError::Timeout);
                };
                if remaining.is_zero() {
                    return Err(TimedLockError::Timeout);
                }
                std::thread::sleep(remaining.min(Duration::from_millis(1)));
            }
        }
    }
}

/// Conservatively estimates retained in-memory size, not serialized OTLP size.
pub(crate) fn estimated_log_size(
    record: &SdkLogRecord,
    instrumentation: &InstrumentationScope,
) -> usize {
    let mut size = size_of::<SdkLogRecord>().saturating_add(size_of::<InstrumentationScope>());
    size = size.saturating_add(record.event_name().map_or(0, str::len));
    size = size.saturating_add(record.target().map_or(0, |target| target.len()));
    size = size.saturating_add(record.severity_text().map_or(0, str::len));
    size = size.saturating_add(record.body().map_or(0, estimated_any_value_size));
    size = record.attributes_iter().fold(size, |size, (key, value)| {
        size.saturating_add(key.as_str().len())
            .saturating_add(estimated_any_value_size(value))
    });
    size = size.saturating_add(instrumentation.name().len());
    size = size.saturating_add(instrumentation.version().map_or(0, str::len));
    size = size.saturating_add(instrumentation.schema_url().map_or(0, str::len));
    size = instrumentation.attributes().fold(size, |size, value| {
        size.saturating_add(estimated_key_value_size(value))
    });
    size
}

fn estimated_any_value_size(value: &AnyValue) -> usize {
    size_of::<AnyValue>().saturating_add(match value {
        AnyValue::Int(_) | AnyValue::Double(_) | AnyValue::Boolean(_) => 0,
        AnyValue::String(value) => value.as_str().len(),
        AnyValue::Bytes(value) => value.len(),
        AnyValue::ListAny(values) => values.iter().fold(0usize, |size, value| {
            size.saturating_add(estimated_any_value_size(value))
        }),
        AnyValue::Map(values) => values.iter().fold(0usize, |size, (key, value)| {
            size.saturating_add(key.as_str().len())
                .saturating_add(estimated_any_value_size(value))
        }),
        _ => 0,
    })
}

fn estimated_key_value_size(key_value: &KeyValue) -> usize {
    size_of::<KeyValue>()
        .saturating_add(key_value.key.as_str().len())
        .saturating_add(match &key_value.value {
            Value::Bool(_) | Value::I64(_) | Value::F64(_) => 0,
            Value::String(value) => value.as_str().len(),
            Value::Array(Array::Bool(values)) => size_of::<bool>().saturating_mul(values.len()),
            Value::Array(Array::I64(values)) => size_of::<i64>().saturating_mul(values.len()),
            Value::Array(Array::F64(values)) => size_of::<f64>().saturating_mul(values.len()),
            Value::Array(Array::String(values)) => values.iter().fold(0usize, |size, value| {
                size.saturating_add(value.as_str().len())
            }),
            _ => 0,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::logs::LogRecord;
    use opentelemetry::KeyValue;

    #[test]
    fn nested_values_increase_estimated_size() {
        let instrumentation = InstrumentationScope::builder("test").build();
        let empty = SdkLogRecord::new();
        let mut populated = SdkLogRecord::new();
        populated.set_body(AnyValue::Map(Box::new(
            [(
                "nested".into(),
                AnyValue::ListAny(Box::new(vec!["value".into()])),
            )]
            .into(),
        )));
        populated.add_attribute("attribute", "value");

        assert!(
            estimated_log_size(&populated, &instrumentation)
                > estimated_log_size(&empty, &instrumentation)
        );
    }

    #[test]
    fn instrumentation_scope_content_is_counted() {
        let record = SdkLogRecord::new();
        let empty = InstrumentationScope::builder("test").build();
        let populated = InstrumentationScope::builder("test")
            .with_version("1.0")
            .with_schema_url("https://example.com/schema")
            .with_attributes([KeyValue::new("scope.attribute", "value")])
            .build();

        assert!(estimated_log_size(&record, &populated) > estimated_log_size(&record, &empty));
    }
}
