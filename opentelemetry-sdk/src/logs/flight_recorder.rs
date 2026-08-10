use crate::logs::SdkLogRecord;
use opentelemetry::logs::AnyValue;
use opentelemetry::{Array, InstrumentationScope, KeyValue, Value};
use std::mem::size_of;

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
