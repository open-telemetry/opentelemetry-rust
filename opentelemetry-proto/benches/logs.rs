/*
    The benchmark results
    criterion = "0.5"
    OS: macOS
    Hardware: Apple Silicon
    Batch Size: 512 logs (default batch size)

    1. Conversion: OTel struct → Protobuf struct (using group_logs_by_resource_and_scope)
    2. Serialization: Protobuf struct → bytes (prost::Message::encode_to_vec())
    3. Compression: bytes → gzip compressed bytes

    | Test                              | Conversion | Serialization | Compression | Total    | Per Log  |
    |-----------------------------------|------------|---------------|-------------|----------|----------|
    | batch_512_with_4_attrs            | ~165 µs    | ~73 µs        | ~227 µs     | ~465 µs  | ~908 ns  |
    | batch_512_with_10_attrs           | ~362 µs    | ~151 µs       | ~408 µs     | ~921 µs  | ~1799 ns |
    | batch_512_1_scope_10_targets      | ~362 µs    | ~150 µs       | ~410 µs     | ~922 µs  | ~1801 ns |

    Notes:
    - batch_512_with_4_attrs: 10 different scopes (~51 logs per scope)
    - batch_512_with_10_attrs: 10 different scopes (~51 logs per scope)
    - batch_512_1_scope_10_targets: 1 scope with 10 different targets (~51 logs per target)
      Tests grouping by target within the same scope
*/

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use opentelemetry::logs::{AnyValue, LogRecord as _, Logger, LoggerProvider, Severity};
use opentelemetry::time::now;
use opentelemetry::InstrumentationScope;
use opentelemetry_sdk::logs::{SdkLogRecord, SdkLoggerProvider};

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

#[allow(clippy::vec_box)]
fn create_log_batch(
    scopes: &[InstrumentationScope],
    batch_size: usize,
    attribute_count: usize,
) -> Vec<Box<(SdkLogRecord, InstrumentationScope)>> {
    create_log_batch_with_targets(scopes, batch_size, attribute_count, None)
}

#[allow(clippy::vec_box)]
fn create_log_batch_with_targets(
    scopes: &[InstrumentationScope],
    batch_size: usize,
    attribute_count: usize,
    targets: Option<&[&str]>,
) -> Vec<Box<(SdkLogRecord, InstrumentationScope)>> {
    // Create a temporary logger provider just for creating log records
    let temp_provider = SdkLoggerProvider::builder().build();

    let mut log_data = Vec::with_capacity(batch_size);

    for i in 0..batch_size {
        let scope = &scopes[i % scopes.len()];
        let logger = temp_provider.logger_with_scope(scope.clone());
        let mut record = logger.create_log_record();

        record.set_observed_timestamp(now());
        record.set_timestamp(now());
        record.set_severity_number(Severity::Info);
        record.set_severity_text("INFO");
        record.set_body(AnyValue::String("Benchmark log message".into()));

        // Set target if provided
        if let Some(target_list) = targets {
            let target_str = target_list[i % target_list.len()];
            record.set_target(target_str.to_string());
        }

        // Add trace context
        let trace_id =
            opentelemetry::trace::TraceId::from_hex("4bf92f3577b34da6a3ce929d0e0e4736").unwrap();
        let span_id = opentelemetry::trace::SpanId::from_hex("00f067aa0ba902b7").unwrap();
        let trace_flags = opentelemetry::trace::TraceFlags::SAMPLED;
        record.set_trace_context(trace_id, span_id, Some(trace_flags));

        // Add attributes
        for j in 0..attribute_count {
            record.add_attribute(format!("attr_{}", j), format!("value_{}", j));
        }

        log_data.push(Box::new((record, scope.clone())));
    }

    log_data
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

    // Create 10 different targets for grouping test
    let targets: Vec<String> = (0..10).map(|i| format!("target::module_{}", i)).collect();
    let target_refs: Vec<&str> = targets.iter().map(|s| s.as_str()).collect();

    // Pre-create log batches for each test case (not measured in benchmarks)
    let log_tuples_4_attrs = create_log_batch(&instrumentation_scopes, BATCH_SIZE, 4);
    let log_batch_4_attrs = LogBatch::new_with_owned_data(&log_tuples_4_attrs);

    let log_tuples_10_attrs = create_log_batch(&instrumentation_scopes, BATCH_SIZE, 10);
    let log_batch_10_attrs = LogBatch::new_with_owned_data(&log_tuples_10_attrs);

    // Batch with same scope but 10 different targets (tests grouping by target within a scope)
    let single_scope = vec![instrumentation_scopes[0].clone()];
    let log_tuples_1_scope_10_targets =
        create_log_batch_with_targets(&single_scope, BATCH_SIZE, 10, Some(&target_refs));
    let log_batch_1_scope_10_targets =
        LogBatch::new_with_owned_data(&log_tuples_1_scope_10_targets);

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

    group.bench_function("batch_512_with_1_scope_10_targets", |b| {
        b.iter(|| {
            let request = create_batch_request(black_box(&log_batch_1_scope_10_targets), &resource);
            black_box(request);
        });
    });

    group.finish();

    // Step 2: Protobuf struct to bytes (batch of 512 from 10 scopes)
    // Pre-create protobuf requests for serialization benchmarks
    let request_4_attrs = create_batch_request(&log_batch_4_attrs, &resource);
    let request_10_attrs = create_batch_request(&log_batch_10_attrs, &resource);
    let request_1_scope_10_targets = create_batch_request(&log_batch_1_scope_10_targets, &resource);

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

    group.bench_function("batch_512_with_1_scope_10_targets_to_bytes", |b| {
        b.iter(|| {
            let bytes = black_box(&request_1_scope_10_targets).encode_to_vec();
            black_box(bytes);
        });
    });

    group.finish();

    // Step 3: Bytes to compressed bytes (gzip) - batch of 512 from 10 scopes
    // Pre-serialize for compression benchmarks
    let bytes_4_attrs = request_4_attrs.encode_to_vec();
    let bytes_10_attrs = request_10_attrs.encode_to_vec();
    let bytes_1_scope_10_targets = request_1_scope_10_targets.encode_to_vec();

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

    group.bench_function("batch_512_with_1_scope_10_targets_compress", |b| {
        b.iter(|| {
            use flate2::{write::GzEncoder, Compression};
            use std::io::Write;
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder
                .write_all(black_box(&bytes_1_scope_10_targets))
                .unwrap();
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
