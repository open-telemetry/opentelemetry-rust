/*
    The benchmark results:
    criterion = "0.5"
    OS: macOS
    Hardware: Apple Silicon

    | Test                                        | Average time|
    |---------------------------------------------|-------------|
    | basic_with_4_attributes                     | ~289 ns     |
    | basic_with_10_attributes                    | ~681 ns     |
    | complex                                     | ~1.18 µs    |
*/

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use opentelemetry::logs::{AnyValue, LogRecord as _, Logger, LoggerProvider, Severity};
use opentelemetry::time::now;
use opentelemetry::InstrumentationScope;
use opentelemetry::Key;
use opentelemetry_sdk::logs::{LogProcessor, SdkLogRecord, SdkLoggerProvider};
use std::collections::HashMap;

#[cfg(feature = "gen-tonic-messages")]
use opentelemetry_proto::tonic::logs::v1::LogRecord as TonicLogRecord;

// Mock processor for creating log records
#[derive(Debug)]
struct MockProcessor;

impl LogProcessor for MockProcessor {
    fn emit(&self, _record: &mut SdkLogRecord, _instrumentation: &InstrumentationScope) {}

    fn force_flush(&self) -> opentelemetry_sdk::error::OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(
        &self,
        _timeout: std::time::Duration,
    ) -> opentelemetry_sdk::error::OTelSdkResult {
        Ok(())
    }
}

fn create_log_record_with_4_attributes() -> SdkLogRecord {
    let processor = MockProcessor {};
    let logger = SdkLoggerProvider::builder()
        .with_log_processor(processor)
        .build()
        .logger("test");
    let mut record = logger.create_log_record();
    record.set_observed_timestamp(now());
    record.set_timestamp(now());
    record.set_severity_number(Severity::Info);
    record.set_severity_text("INFO");
    record.set_body(AnyValue::String("Log message".into()));

    // Add trace context
    let trace_id =
        opentelemetry::trace::TraceId::from_hex("4bf92f3577b34da6a3ce929d0e0e4736").unwrap();
    let span_id = opentelemetry::trace::SpanId::from_hex("00f067aa0ba902b7").unwrap();
    let trace_flags = opentelemetry::trace::TraceFlags::SAMPLED;

    record.set_trace_context(trace_id, span_id, Some(trace_flags));

    // Add 4 attributes
    record.add_attribute("http.method", "GET");
    record.add_attribute("http.status_code", 200);
    record.add_attribute("http.url", "https://example.com/api");
    record.add_attribute("user.id", "user123");

    record
}

fn create_log_record_with_10_attributes() -> SdkLogRecord {
    let processor = MockProcessor {};
    let logger = SdkLoggerProvider::builder()
        .with_log_processor(processor)
        .build()
        .logger("test");
    let mut record = logger.create_log_record();
    record.set_observed_timestamp(now());
    record.set_timestamp(now());
    record.set_severity_number(Severity::Info);
    record.set_severity_text("INFO");
    record.set_body(AnyValue::String("Log message".into()));

    // Add trace context
    let trace_id =
        opentelemetry::trace::TraceId::from_hex("4bf92f3577b34da6a3ce929d0e0e4736").unwrap();
    let span_id = opentelemetry::trace::SpanId::from_hex("00f067aa0ba902b7").unwrap();
    let trace_flags = opentelemetry::trace::TraceFlags::SAMPLED;

    record.set_trace_context(trace_id, span_id, Some(trace_flags));

    // Add 10 attributes
    for i in 0..10 {
        record.add_attribute(format!("attr_{}", i), format!("value_{}", i));
    }

    record
}

fn create_log_record_complex() -> SdkLogRecord {
    let processor = MockProcessor {};
    let logger = SdkLoggerProvider::builder()
        .with_log_processor(processor)
        .build()
        .logger("test");
    let mut record = logger.create_log_record();
    record.set_observed_timestamp(now());
    record.set_timestamp(now());
    record.set_severity_number(Severity::Info);
    record.set_severity_text("INFO");

    // Complex nested map body
    let mut inner_map = HashMap::new();
    inner_map.insert(
        Key::new("inner_key1"),
        AnyValue::String("inner_value1".into()),
    );
    inner_map.insert(Key::new("inner_key2"), AnyValue::Int(42));
    inner_map.insert(Key::new("inner_key3"), AnyValue::Boolean(true));

    let mut outer_map = HashMap::new();
    outer_map.insert(Key::new("string_field"), AnyValue::String("value".into()));
    outer_map.insert(Key::new("number_field"), AnyValue::Int(123));
    outer_map.insert(Key::new("nested_map"), AnyValue::Map(Box::new(inner_map)));
    outer_map.insert(
        Key::new("array_field"),
        AnyValue::ListAny(Box::new(vec![
            AnyValue::String("item1".into()),
            AnyValue::String("item2".into()),
            AnyValue::Int(100),
        ])),
    );
    record.set_body(AnyValue::Map(Box::new(outer_map)));

    // Add trace context
    let trace_id =
        opentelemetry::trace::TraceId::from_hex("4bf92f3577b34da6a3ce929d0e0e4736").unwrap();
    let span_id = opentelemetry::trace::SpanId::from_hex("00f067aa0ba902b7").unwrap();
    let trace_flags = opentelemetry::trace::TraceFlags::SAMPLED;

    record.set_trace_context(trace_id, span_id, Some(trace_flags));

    // Add 10 attributes
    for i in 0..10 {
        record.add_attribute(format!("attr_{}", i), format!("value_{}", i));
    }

    record
}

#[cfg(feature = "gen-tonic-messages")]
fn bench_log_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("log_record_conversion");

    group.bench_function("basic_with_4_attributes", |b| {
        let record = create_log_record_with_4_attributes();
        b.iter(|| {
            let _proto: TonicLogRecord = black_box(&record).into();
            black_box(_proto);
        });
    });

    group.bench_function("basic_with_10_attributes", |b| {
        let record = create_log_record_with_10_attributes();
        b.iter(|| {
            let _proto: TonicLogRecord = black_box(&record).into();
            black_box(_proto);
        });
    });

    group.bench_function("complex", |b| {
        let record = create_log_record_complex();
        b.iter(|| {
            let _proto: TonicLogRecord = black_box(&record).into();
            black_box(_proto);
        });
    });

    group.finish();
}

#[cfg(not(feature = "gen-tonic-messages"))]
fn bench_log_conversion(_c: &mut Criterion) {
    // Benchmark is only available when gen-tonic-messages feature is enabled
}

criterion_group!(benches, bench_log_conversion);
criterion_main!(benches);
