/*
    The benchmark results - Complete Pipeline Breakdown:
    criterion = "0.5"
    OS: macOS
    Hardware: Apple Silicon
    Batch Size: 512 logs (default batch size) from 10 different scopes (~51 logs per scope)

    Pipeline Stages:
    1. Conversion: OTel struct → Protobuf struct (using group_logs_by_resource_and_scope)
    2. Serialization: Protobuf struct → bytes (prost::Message::encode_to_vec())
    3. Compression: bytes → gzip compressed bytes

    | Test                       | Conversion | Serialization | Compression | Total    | Per Log  |
    |----------------------------|------------|---------------|-------------|----------|----------|
    | batch_512_with_4_attrs     | ~158 µs    | ~72 µs        | ~253 µs     | ~483 µs  | ~943 ns  |
    | batch_512_with_10_attrs    | ~362 µs    | ~143 µs       | ~382 µs     | ~887 µs  | ~1732 ns |
    | batch_512_complex          | ~665 µs    | ~341 µs       | ~519 µs     | ~1.52 ms | ~2979 ns |

    Key Insights:
    - With 10 scopes, group_logs_by_resource_and_scope creates 1 ResourceLogs with 10 ScopeLogs
    - Serialization improved 18-20% vs single scope (better protobuf compression with scope grouping)
    - Conversion cost dominates for complex records (~44% of total)
    - Per-record costs: conversion ~295-1,275 ns, serialization ~130-645 ns, compression ~492-988 ns
    - Benchmark accurately reflects production code path with multiple components/libraries
*/

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use opentelemetry::logs::{AnyValue, LogRecord as _, Logger, LoggerProvider, Severity};
use opentelemetry::time::now;
use opentelemetry::InstrumentationScope;
use opentelemetry::Key;
use opentelemetry_sdk::logs::{LogProcessor, SdkLogRecord, SdkLoggerProvider};
use std::collections::HashMap;

#[cfg(feature = "gen-tonic-messages")]
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;

#[cfg(feature = "gen-tonic-messages")]
use opentelemetry_proto::transform::logs::tonic::group_logs_by_resource_and_scope;

#[cfg(feature = "gen-tonic-messages")]
use opentelemetry_proto::transform::common::tonic::ResourceAttributesWithSchema;

#[cfg(feature = "gen-tonic-messages")]
use opentelemetry_sdk::logs::LogBatch;

#[cfg(feature = "gen-tonic-messages")]
use prost::Message;

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
        .logger("benchmark");
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
        .logger("benchmark");
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
        .logger("benchmark");
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
fn create_batch_request(
    log_batch: &LogBatch<'_>,
    resource: &ResourceAttributesWithSchema,
) -> ExportLogsServiceRequest {
    // This mirrors the actual OTLP exporter code path
    let resource_logs = group_logs_by_resource_and_scope(log_batch, resource);
    ExportLogsServiceRequest { resource_logs }
}

#[cfg(feature = "gen-tonic-messages")]
fn bench_log_conversion(c: &mut Criterion) {
    const BATCH_SIZE: usize = 512;
    const NUM_SCOPES: usize = 10;

    // Create 10 instrumentation scopes and resource (simulating realistic production setup)
    let instrumentation_scopes: Vec<InstrumentationScope> = (0..NUM_SCOPES)
        .map(|i| {
            InstrumentationScope::builder(format!("component.{}", i))
                .with_version(format!("1.{}.0", i))
                .build()
        })
        .collect();

    let resource = ResourceAttributesWithSchema::from(
        &opentelemetry_sdk::Resource::builder_empty()
            .with_attributes([
                opentelemetry::KeyValue::new("service.name", "benchmark-service"),
                opentelemetry::KeyValue::new("service.version", "1.0.0"),
                opentelemetry::KeyValue::new("deployment.environment", "production"),
                opentelemetry::KeyValue::new("host.name", "benchmark-host"),
                opentelemetry::KeyValue::new("process.pid", 12345),
            ])
            .build(),
    );

    // Pre-create all log records once (not measured in benchmarks)
    let records_4_attrs: Vec<_> = (0..BATCH_SIZE)
        .map(|_| create_log_record_with_4_attributes())
        .collect();
    let records_10_attrs: Vec<_> = (0..BATCH_SIZE)
        .map(|_| create_log_record_with_10_attributes())
        .collect();
    let records_complex: Vec<_> = (0..BATCH_SIZE)
        .map(|_| create_log_record_complex())
        .collect();

    // Pre-create log batches for each test case
    let log_tuples_4_attrs: Vec<Box<(SdkLogRecord, InstrumentationScope)>> = records_4_attrs
        .into_iter()
        .enumerate()
        .map(|(i, r)| Box::new((r, instrumentation_scopes[i % NUM_SCOPES].clone())))
        .collect();
    let log_batch_4_attrs = LogBatch::new_with_owned_data(&log_tuples_4_attrs);

    let log_tuples_10_attrs: Vec<Box<(SdkLogRecord, InstrumentationScope)>> = records_10_attrs
        .into_iter()
        .enumerate()
        .map(|(i, r)| Box::new((r, instrumentation_scopes[i % NUM_SCOPES].clone())))
        .collect();
    let log_batch_10_attrs = LogBatch::new_with_owned_data(&log_tuples_10_attrs);

    let log_tuples_complex: Vec<Box<(SdkLogRecord, InstrumentationScope)>> = records_complex
        .into_iter()
        .enumerate()
        .map(|(i, r)| Box::new((r, instrumentation_scopes[i % NUM_SCOPES].clone())))
        .collect();
    let log_batch_complex = LogBatch::new_with_owned_data(&log_tuples_complex);

    // Step 1: OTel struct to Protobuf struct (batch of 512 from 10 scopes)
    let mut group = c.benchmark_group("log_batch_conversion");

    group.bench_function("batch_512_with_4_attributes", |b| {
        b.iter(|| {
            let request = create_batch_request(black_box(&log_batch_4_attrs), &resource);
            black_box(request);
        });
    });

    group.bench_function("batch_512_with_10_attributes", |b| {
        b.iter(|| {
            let request = create_batch_request(black_box(&log_batch_10_attrs), &resource);
            black_box(request);
        });
    });

    group.bench_function("batch_512_complex", |b| {
        b.iter(|| {
            let request = create_batch_request(black_box(&log_batch_complex), &resource);
            black_box(request);
        });
    });

    group.finish();

    // Step 2: Protobuf struct to bytes (batch of 512 from 10 scopes)
    // Pre-create protobuf requests for serialization benchmarks
    let request_4_attrs = create_batch_request(&log_batch_4_attrs, &resource);
    let request_10_attrs = create_batch_request(&log_batch_10_attrs, &resource);
    let request_complex = create_batch_request(&log_batch_complex, &resource);

    let mut group = c.benchmark_group("log_batch_serialization");

    group.bench_function("batch_512_with_4_attributes_to_bytes", |b| {
        b.iter(|| {
            let bytes = black_box(&request_4_attrs).encode_to_vec();
            black_box(bytes);
        });
    });

    group.bench_function("batch_512_with_10_attributes_to_bytes", |b| {
        b.iter(|| {
            let bytes = black_box(&request_10_attrs).encode_to_vec();
            black_box(bytes);
        });
    });

    group.bench_function("batch_512_complex_to_bytes", |b| {
        b.iter(|| {
            let bytes = black_box(&request_complex).encode_to_vec();
            black_box(bytes);
        });
    });

    group.finish();

    // Step 3: Bytes to compressed bytes (gzip) - batch of 512 from 10 scopes
    // Pre-serialize for compression benchmarks
    let bytes_4_attrs = request_4_attrs.encode_to_vec();
    let bytes_10_attrs = request_10_attrs.encode_to_vec();
    let bytes_complex = request_complex.encode_to_vec();

    let mut group = c.benchmark_group("log_batch_compression");

    group.bench_function("batch_512_with_4_attributes_compress", |b| {
        b.iter(|| {
            use flate2::{write::GzEncoder, Compression};
            use std::io::Write;
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(black_box(&bytes_4_attrs)).unwrap();
            let compressed = encoder.finish().unwrap();
            black_box(compressed);
        });
    });

    group.bench_function("batch_512_with_10_attributes_compress", |b| {
        b.iter(|| {
            use flate2::{write::GzEncoder, Compression};
            use std::io::Write;
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(black_box(&bytes_10_attrs)).unwrap();
            let compressed = encoder.finish().unwrap();
            black_box(compressed);
        });
    });

    group.bench_function("batch_512_complex_compress", |b| {
        b.iter(|| {
            use flate2::{write::GzEncoder, Compression};
            use std::io::Write;
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(black_box(&bytes_complex)).unwrap();
            let compressed = encoder.finish().unwrap();
            black_box(compressed);
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
