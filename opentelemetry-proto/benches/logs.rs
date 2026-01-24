/*
    The benchmark results
    criterion = "0.5"
    OS: macOS
    Hardware: Apple Silicon

    Setup:
    - Batch Size: 512 logs (default batch size)
    - 1 InstrumentationScope (realistic for tracing-appender)
    - 10 different targets (~51 logs per target)

    Pipeline stages:
    1. Conversion: OTel struct → Protobuf struct (using group_logs_by_resource_and_scope)
    2. Serialization: Protobuf struct → bytes (prost::Message::encode_to_vec())
    3. Compression: bytes → gzip or zstd compressed bytes

    Run: cargo bench --bench logs --features gen-tonic-messages -p opentelemetry-proto

    | Test                              | Conversion | Serialization | Gzip Compress | Zstd Compress |
    |-----------------------------------|------------|---------------|---------------|---------------|
    | batch_512_with_4_attrs            | ~170 µs    | ~66 µs        | ~249 µs       | ~30 µs        |
    | batch_512_with_10_attrs           | ~365 µs    | ~133 µs       | ~396 µs       | ~38 µs        |

    === Compression Ratios (512 logs) ===
    4 attrs:  87945 bytes -> gzip: 3817 bytes (4.3%), zstd: 2512 bytes (2.9%)
    10 attrs: 152457 bytes -> gzip: 5055 bytes (3.3%), zstd: 2528 bytes (1.7%)
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
    targets: &[&str],
    batch_size: usize,
    attribute_count: usize,
) -> Vec<Box<(SdkLogRecord, InstrumentationScope)>> {
    // Single instrumentation scope (realistic for tracing-appender usage)
    let scope = InstrumentationScope::builder("opentelemetry-appender-tracing")
        .with_version("0.28.0")
        .with_attributes([
            opentelemetry::KeyValue::new("scope.type", "library"),
            opentelemetry::KeyValue::new("scope.id", "0"),
            opentelemetry::KeyValue::new("scope.enabled", true),
        ])
        .build();

    // Create a temporary logger just for creating log records
    // The logger's scope doesn't matter since LogBatch uses the scope from the tuple
    let temp_provider = SdkLoggerProvider::builder().build();
    let logger = temp_provider.logger("benchmark");

    let mut log_data = Vec::with_capacity(batch_size);

    for i in 0..batch_size {
        let mut record = logger.create_log_record();

        record.set_observed_timestamp(now());
        record.set_timestamp(now());
        record.set_severity_number(Severity::Info);
        record.set_severity_text("INFO");
        record.set_body(AnyValue::String("Benchmark log message".into()));

        // Set target (10 different targets, ~51 logs per target)
        record.set_target(targets[i % targets.len()].to_string());

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

    // Create 10 different targets (~51 logs per target)
    let targets: Vec<String> = (0..10).map(|i| format!("target::module_{}", i)).collect();
    let target_refs: Vec<&str> = targets.iter().map(|s| s.as_str()).collect();

    // Pre-create log batches for each test case (not measured in benchmarks)
    let log_tuples_4_attrs = create_log_batch(&target_refs, BATCH_SIZE, 4);
    let log_batch_4_attrs = LogBatch::new_with_owned_data(&log_tuples_4_attrs);

    let log_tuples_10_attrs = create_log_batch(&target_refs, BATCH_SIZE, 10);
    let log_batch_10_attrs = LogBatch::new_with_owned_data(&log_tuples_10_attrs);

    // Step 1: OTel struct to Protobuf struct (batch of 512 with 1 scope and 10 targets)
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

    group.finish();

    // Step 2: Protobuf struct to bytes (batch of 512 with 1 scope and 10 targets)
    // Pre-create protobuf requests for serialization benchmarks
    let request_4_attrs = create_batch_request(&log_batch_4_attrs, &resource);
    let request_10_attrs = create_batch_request(&log_batch_10_attrs, &resource);

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

    group.finish();

    // Step 3: Bytes to compressed bytes - batch of 512 with 1 scope and 10 targets
    // Pre-serialize for compression benchmarks
    let bytes_4_attrs = request_4_attrs.encode_to_vec();
    let bytes_10_attrs = request_10_attrs.encode_to_vec();

    // Print compression ratios for reference
    {
        use flate2::{write::GzEncoder, Compression};
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&bytes_4_attrs).unwrap();
        let gzip_4 = encoder.finish().unwrap();

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&bytes_10_attrs).unwrap();
        let gzip_10 = encoder.finish().unwrap();

        let zstd_4 = zstd::bulk::compress(&bytes_4_attrs, 0).unwrap();
        let zstd_10 = zstd::bulk::compress(&bytes_10_attrs, 0).unwrap();

        println!("\n=== Compression Ratios (512 logs) ===");
        println!(
            "4 attrs:  {} bytes -> gzip: {} bytes ({:.1}%), zstd: {} bytes ({:.1}%)",
            bytes_4_attrs.len(),
            gzip_4.len(),
            (gzip_4.len() as f64 / bytes_4_attrs.len() as f64) * 100.0,
            zstd_4.len(),
            (zstd_4.len() as f64 / bytes_4_attrs.len() as f64) * 100.0
        );
        println!(
            "10 attrs: {} bytes -> gzip: {} bytes ({:.1}%), zstd: {} bytes ({:.1}%)",
            bytes_10_attrs.len(),
            gzip_10.len(),
            (gzip_10.len() as f64 / bytes_10_attrs.len() as f64) * 100.0,
            zstd_10.len(),
            (zstd_10.len() as f64 / bytes_10_attrs.len() as f64) * 100.0
        );
        println!();
    }

    // Gzip compression (same as OTLP exporter with gzip-http feature)
    let mut group = c.benchmark_group("log_batch_compression_gzip");

    group.bench_function("batch_512_with_4_attributes_gzip", |b| {
        b.iter(|| {
            use flate2::{write::GzEncoder, Compression};
            use std::io::Write;
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(black_box(&bytes_4_attrs)).unwrap();
            let compressed = encoder.finish().unwrap();
            black_box(compressed);
        });
    });

    group.bench_function("batch_512_with_10_attributes_gzip", |b| {
        b.iter(|| {
            use flate2::{write::GzEncoder, Compression};
            use std::io::Write;
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(black_box(&bytes_10_attrs)).unwrap();
            let compressed = encoder.finish().unwrap();
            black_box(compressed);
        });
    });

    group.finish();

    // Zstd compression (same as OTLP exporter with zstd-http feature)
    let mut group = c.benchmark_group("log_batch_compression_zstd");

    group.bench_function("batch_512_with_4_attributes_zstd", |b| {
        b.iter(|| {
            let compressed = zstd::bulk::compress(black_box(&bytes_4_attrs), 0).unwrap();
            black_box(compressed);
        });
    });

    group.bench_function("batch_512_with_10_attributes_zstd", |b| {
        b.iter(|| {
            let compressed = zstd::bulk::compress(black_box(&bytes_10_attrs), 0).unwrap();
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
