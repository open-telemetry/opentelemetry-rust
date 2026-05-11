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
    | batch_512_with_4_attrs            | ~187 µs    | ~78 µs        | ~1,295 µs     | ~180 µs       |
    | batch_512_with_10_attrs           | ~397 µs    | ~152 µs       | ~2,911 µs     | ~486 µs       |
    | batch_512_with_4_attrs_no_group   | ~171 µs    | -             | -             | -             |
    | batch_512_with_10_attrs_no_group  | ~381 µs    | -             | -             | -             |

    The `_no_group` variants skip `group_logs_by_resource_and_scope`'s HashMap
    grouping step (placing all records in a single ScopeLogs) to isolate the
    cost of grouping by `target`. Delta is ~16 µs/batch (~31 ns/record) and
    largely independent of attribute count.

    === Compression Ratios (512 logs) ===
    4 attrs:  129274 bytes -> gzip: 43084 bytes (33.3%), zstd: 46229 bytes (35.8%)
    10 attrs: 226830 bytes -> gzip: 67925 bytes (29.9%), zstd: 77336 bytes (34.1%)
*/

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use opentelemetry::logs::{AnyValue, LogRecord as _, Logger, LoggerProvider, Severity};
use opentelemetry::time::now;
use opentelemetry::InstrumentationScope;
use opentelemetry_sdk::logs::{SdkLogRecord, SdkLoggerProvider};
use rand::Rng;

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

    // Seeded RNG for reproducible but varied data
    let mut rng = rand::rng();

    // Realistic log message templates
    let messages = [
        "HTTP request completed",
        "Database query executed",
        "Cache miss for key lookup",
        "User authentication successful",
        "Payment processing started",
        "File upload completed",
        "Background job scheduled",
        "Rate limit threshold reached",
        "Connection pool exhausted",
        "Configuration reload triggered",
    ];

    let severities = [
        (Severity::Trace, "TRACE"),
        (Severity::Debug, "DEBUG"),
        (Severity::Info, "INFO"),
        (Severity::Warn, "WARN"),
        (Severity::Error, "ERROR"),
    ];

    let mut log_data = Vec::with_capacity(batch_size);

    for i in 0..batch_size {
        let mut record = logger.create_log_record();

        record.set_observed_timestamp(now());
        record.set_timestamp(now());

        // Vary severity (mostly Info, some Debug/Warn/Error)
        let (severity, severity_text) = severities[i % severities.len()];
        record.set_severity_number(severity);
        record.set_severity_text(severity_text);

        // Varied log bodies with unique request IDs
        let msg = messages[i % messages.len()];
        record.set_body(AnyValue::String(
            format!("{} request_id={}", msg, rng.random::<u64>()).into(),
        ));

        // Set target (10 different targets, ~51 logs per target)
        record.set_target(targets[i % targets.len()].to_string());

        // Unique trace context per log (realistic: each log from a different span)
        let trace_id = opentelemetry::trace::TraceId::from(rng.random::<u128>());
        let span_id = opentelemetry::trace::SpanId::from(rng.random::<u64>());
        let trace_flags = opentelemetry::trace::TraceFlags::SAMPLED;
        record.set_trace_context(trace_id, span_id, Some(trace_flags));

        // Varied attribute values (keys stay the same, values differ per log)
        for j in 0..attribute_count {
            record.add_attribute(
                format!("attr_{}", j),
                format!("value_{}_{}", j, rng.random::<u32>()),
            );
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

// LOCAL ISOLATION BENCH: same per-record proto conversion as the OTLP path,
// but bypasses the HashMap-keyed grouping by `target`. All records are placed
// into a single ScopeLogs with one shared scope. Used only to measure the
// cost of `group_logs_by_resource_and_scope`'s grouping step.
#[cfg(feature = "gen-tonic-messages")]
fn create_batch_request_no_grouping(
    log_batch: &LogBatch<'_>,
    resource: &ResourceAttributesWithSchema,
) -> ExportLogsServiceRequest {
    use opentelemetry_proto::tonic::common::v1::InstrumentationScope as ProtoScope;
    use opentelemetry_proto::tonic::logs::v1::{ResourceLogs, ScopeLogs};
    use opentelemetry_proto::tonic::resource::v1::Resource as ProtoResource;

    // Pick scope from first record (matches the bench setup: 1 scope for all).
    let mut iter = log_batch.iter();
    let first_scope: Option<ProtoScope> = iter.next().map(|(_, scope)| ProtoScope {
        name: scope.name().to_string(),
        version: scope.version().map(ToString::to_string).unwrap_or_default(),
        attributes: vec![],
        dropped_attributes_count: 0,
    });

    let log_records = log_batch.iter().map(|(rec, _)| rec.into()).collect();

    ExportLogsServiceRequest {
        resource_logs: vec![ResourceLogs {
            resource: Some(ProtoResource {
                attributes: resource.attributes.0.clone(),
                dropped_attributes_count: 0,
                entity_refs: vec![],
            }),
            schema_url: resource.schema_url.clone().unwrap_or_default(),
            scope_logs: vec![ScopeLogs {
                schema_url: String::new(),
                scope: first_scope,
                log_records,
            }],
        }],
    }
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

    // Isolation: same per-record proto conversion, no HashMap grouping.
    group.bench_function("batch_512_with_4_attributes_no_grouping", |b| {
        b.iter(|| {
            let request =
                create_batch_request_no_grouping(black_box(&log_batch_4_attrs), &resource);
            black_box(request);
        });
    });

    group.bench_function("batch_512_with_10_attributes_no_grouping", |b| {
        b.iter(|| {
            let request =
                create_batch_request_no_grouping(black_box(&log_batch_10_attrs), &resource);
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
