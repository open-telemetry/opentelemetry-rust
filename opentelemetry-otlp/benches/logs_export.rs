/*
    End-to-End Log Export Benchmark

    This benchmark measures the OTLP log export performance:
    1. Conversion to protobuf
    2. HTTP request creation
    3. Serialization and optional compression
    4. Network transport (to a local fake server)

    The fake HTTP server returns 200 OK immediately to isolate client-side costs.
    Future enhancements can add TLS to measure encryption overhead.

    Setup:
    - Batch size: 512 logs (default SDK batch size)
    - 1 InstrumentationScope (realistic for tracing-appender)
    - 10 different targets (~51 logs per target)
    - Fake HTTP server: Returns 200 OK with minimal latency
    - Protocol: HTTP/Binary (protobuf)

    Run benchmarks:
    ```bash
    # Basic benchmark (no compression)
    cargo bench --bench logs_export

    # With gzip compression
    cargo bench --bench logs_export --features gzip-http

    # With zstd compression
    cargo bench --bench logs_export --features zstd-http

    # All compression variants
    cargo bench --bench logs_export --features gzip-http,zstd-http
    ```

    Benchmark Results:
    criterion = "0.5"
    Hardware: Apple M4 Pro
    | Test                            | Time      | Per Log |
    |---------------------------------|-----------|---------|
    | batch_512_with_4_attrs          | ~406 µs   | ~793 ns |
    | batch_512_with_10_attrs         | ~719 µs   | ~1.4 µs |
    | batch_512_with_4_attrs_gzip     | ~674 µs   | ~1.3 µs |
    | batch_512_with_10_attrs_gzip    | ~1,094 µs | ~2.1 µs |
    | batch_512_with_4_attrs_zstd     | ~389 µs   | ~760 ns |
    | batch_512_with_10_attrs_zstd    | ~745 µs   | ~1.5 µs |
    | raw_http_88kb_payload           | ~98 µs    | -       |
    | raw_http_4kb_payload            | ~57 µs    | -       |

    End-to-End with BatchLogProcessor (emit → batch → export → HTTP):
    Note: This approximates a real e2e scenario but differs from production:
    - Uses force_flush() to synchronize (production relies on timer/batch-size triggers)
    - Fake HTTP server with instant 200 OK (no real network latency or collector processing)
    - Controlled batch size (511 logs) to isolate force_flush as sole trigger

    | Test                              | Time      | Per Log |
    |-----------------------------------|-----------|---------|    
    | e2e_batch_511_with_4_attrs        | ~796 µs   | ~1.6 µs |
    | e2e_batch_511_with_4_attrs_zstd   | ~792 µs   | ~1.5 µs |

    Notes:
    - Export time = Conversion + Serialization + Compression (optional) + HTTP stack overhead
    - E2E time = emit() overhead + channel + batch processing + export time
    - E2E uses 511 logs (below max_export_batch_size=512) so force_flush() is the only export trigger
*/

use criterion::{black_box, Criterion};
use opentelemetry::logs::{AnyValue, LogRecord as _, Logger, LoggerProvider, Severity};
use opentelemetry::time::now;
use opentelemetry::InstrumentationScope;
use opentelemetry::KeyValue;
#[cfg(any(feature = "gzip-http", feature = "zstd-http"))]
use opentelemetry_otlp::WithHttpConfig;
use opentelemetry_otlp::{LogExporter as OtlpLogExporter, Protocol, WithExportConfig};
use opentelemetry_sdk::logs::{
    BatchConfigBuilder, BatchLogProcessor, LogBatch, LogExporter, SdkLogRecord, SdkLoggerProvider,
};
use opentelemetry_sdk::Resource;
use std::time::Duration;
use tokio::runtime::Runtime;

// Fake HTTP server that returns 200 OK immediately
async fn start_fake_otlp_server(port: u16) -> tokio::task::JoinHandle<()> {
    use hyper::service::{make_service_fn, service_fn};
    use hyper::{Body, Request, Response, Server, StatusCode};
    use std::convert::Infallible;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let request_count = Arc::new(AtomicUsize::new(0));

    async fn handle_request(
        req: Request<Body>,
        counter: Arc<AtomicUsize>,
    ) -> Result<Response<Body>, Infallible> {
        let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
        counter.fetch_add(1, Ordering::Relaxed);

        // Log body size for first few requests to verify data is being sent
        let count = counter.load(Ordering::Relaxed);
        if count <= 5 {
            println!(
                "Server received request #{}: {} bytes",
                count,
                body_bytes.len()
            );
        }

        // Return 200 OK
        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::empty())
            .unwrap())
    }

    let make_svc = make_service_fn(move |_conn| {
        let counter = request_count.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle_request(req, counter.clone()))) }
    });

    let addr = ([127, 0, 0, 1], port).into();
    let server = Server::bind(&addr).serve(make_svc);

    println!(
        "Fake OTLP server listening on http://{}:{}",
        addr.ip(),
        addr.port()
    );

    tokio::spawn(async move {
        if let Err(e) = server.await {
            eprintln!("Fake OTLP server error: {}", e);
        }
    })
}

fn create_log_exporter(endpoint: String) -> OtlpLogExporter {
    OtlpLogExporter::builder()
        .with_http()
        .with_endpoint(endpoint)
        .with_protocol(Protocol::HttpBinary)
        .with_timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to create log exporter")
}

#[cfg(feature = "gzip-http")]
fn create_log_exporter_with_gzip(endpoint: String) -> OtlpLogExporter {
    OtlpLogExporter::builder()
        .with_http()
        .with_endpoint(endpoint)
        .with_protocol(Protocol::HttpBinary)
        .with_compression(opentelemetry_otlp::Compression::Gzip)
        .with_timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to create log exporter with gzip compression")
}

#[cfg(feature = "zstd-http")]
fn create_log_exporter_with_zstd(endpoint: String) -> OtlpLogExporter {
    OtlpLogExporter::builder()
        .with_http()
        .with_endpoint(endpoint)
        .with_protocol(Protocol::HttpBinary)
        .with_compression(opentelemetry_otlp::Compression::Zstd)
        .with_timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to create log exporter with zstd compression")
}

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
        // TODO: Use realistic attribute names following OpenTelemetry semantic conventions
        // (e.g., http.request.method, http.response.status_code, url.path, etc.)
        // to better match real-world log data patterns and compression characteristics
        for j in 0..attribute_count {
            record.add_attribute(format!("attr_{}", j), format!("value_{}", j));
        }

        log_data.push(Box::new((record, scope.clone())));
    }

    log_data
}

fn bench_log_export_pipeline(c: &mut Criterion) {
    // Create runtime for async operations
    let rt = Runtime::new().unwrap();

    // Start fake OTLP server
    let port = 14318; // Standard OTLP HTTP port
    let _server_handle = rt.block_on(start_fake_otlp_server(port));

    // Give server time to start
    std::thread::sleep(Duration::from_millis(100));

    let endpoint = format!("http://localhost:{}", port);

    // Create resource once
    let resource = Resource::builder()
        .with_attributes([
            KeyValue::new("service.name", "benchmark-service"),
            KeyValue::new("service.version", "1.0.0"),
            KeyValue::new("deployment.environment", "production"),
            KeyValue::new("host.name", "benchmark-host"),
            KeyValue::new("process.pid", 12345),
        ])
        .build();

    // Create exporter once
    let mut exporter = create_log_exporter(endpoint.clone());
    exporter.set_resource(&resource);

    // Create 10 different targets (~51 logs per target)
    let targets: Vec<String> = (0..10).map(|i| format!("target::module_{}", i)).collect();
    let target_refs: Vec<&str> = targets.iter().map(|s| s.as_str()).collect();

    // Pre-create log batches for each test case (not measured)
    let log_data_4_attrs = create_log_batch(&target_refs, 512, 4);
    let log_data_10_attrs = create_log_batch(&target_refs, 512, 10);

    let mut group = c.benchmark_group("otlp_log_export");

    // Benchmark: 512 logs with 4 attributes - measures only export cost
    group.bench_function("batch_512_with_4_attrs", |b| {
        b.iter(|| {
            // Create LogBatch from pre-created data
            let batch = LogBatch::new_with_owned_data(black_box(&log_data_4_attrs));
            // Measure only the export operation
            rt.block_on(async {
                exporter.export(batch).await.unwrap();
            });
        });
    });

    // Benchmark: 512 logs with 10 attributes - measures only export cost
    group.bench_function("batch_512_with_10_attrs", |b| {
        b.iter(|| {
            // Create LogBatch from pre-created data
            let batch = LogBatch::new_with_owned_data(black_box(&log_data_10_attrs));
            // Measure only the export operation
            rt.block_on(async {
                exporter.export(batch).await.unwrap();
            });
        });
    });

    group.finish();

    #[cfg(feature = "gzip-http")]
    {
        // Create exporter with gzip compression
        let mut exporter_gzip = create_log_exporter_with_gzip(endpoint.clone());
        exporter_gzip.set_resource(&resource);

        let mut group = c.benchmark_group("otlp_log_export_gzip");

        // Benchmark: 512 logs with 4 attributes - WITH GZIP
        group.bench_function("batch_512_with_4_attrs_gzip", |b| {
            b.iter(|| {
                let batch = LogBatch::new_with_owned_data(black_box(&log_data_4_attrs));
                rt.block_on(async {
                    exporter_gzip.export(batch).await.unwrap();
                });
            });
        });

        // Benchmark: 512 logs with 10 attributes - WITH GZIP
        group.bench_function("batch_512_with_10_attrs_gzip", |b| {
            b.iter(|| {
                let batch = LogBatch::new_with_owned_data(black_box(&log_data_10_attrs));
                rt.block_on(async {
                    exporter_gzip.export(batch).await.unwrap();
                });
            });
        });

        group.finish();
    }

    #[cfg(feature = "zstd-http")]
    {
        // Create exporter with zstd compression
        let mut exporter_zstd = create_log_exporter_with_zstd(endpoint.clone());
        exporter_zstd.set_resource(&resource);

        let mut group = c.benchmark_group("otlp_log_export_zstd");

        // Benchmark: 512 logs with 4 attributes - WITH ZSTD
        group.bench_function("batch_512_with_4_attrs_zstd", |b| {
            b.iter(|| {
                let batch = LogBatch::new_with_owned_data(black_box(&log_data_4_attrs));
                rt.block_on(async {
                    exporter_zstd.export(batch).await.unwrap();
                });
            });
        });

        // Benchmark: 512 logs with 10 attributes - WITH ZSTD
        group.bench_function("batch_512_with_10_attrs_zstd", |b| {
            b.iter(|| {
                let batch = LogBatch::new_with_owned_data(black_box(&log_data_10_attrs));
                rt.block_on(async {
                    exporter_zstd.export(batch).await.unwrap();
                });
            });
        });

        group.finish();
    }

    // Benchmark: Raw HTTP overhead (sending pre-serialized bytes)
    // This isolates the HTTP client stack cost from conversion/serialization
    {
        use http::{Request, Uri};
        use opentelemetry_http::{Bytes, HttpClient};

        // Get the HTTP client from a temporary exporter
        let http_client = reqwest::Client::new();
        let uri: Uri = format!("{}/v1/logs", endpoint).parse().unwrap();

        // Pre-create payload matching 512 logs with 4 attrs (~88KB uncompressed)
        let payload_bytes = Bytes::from(vec![0u8; 88000]);

        let mut group = c.benchmark_group("http_stack_overhead");

        group.bench_function("raw_http_88kb_payload", |b| {
            b.iter(|| {
                let request = Request::builder()
                    .method("POST")
                    .uri(uri.clone())
                    .header("content-type", "application/x-protobuf")
                    .body(black_box(payload_bytes.clone()))
                    .unwrap();

                rt.block_on(async {
                    http_client.send_bytes(request).await.unwrap();
                });
            });
        });

        // Smaller payload (~4KB, similar to compressed size)
        let small_payload = Bytes::from(vec![0u8; 4000]);

        group.bench_function("raw_http_4kb_payload", |b| {
            b.iter(|| {
                let request = Request::builder()
                    .method("POST")
                    .uri(uri.clone())
                    .header("content-type", "application/x-protobuf")
                    .body(black_box(small_payload.clone()))
                    .unwrap();

                rt.block_on(async {
                    http_client.send_bytes(request).await.unwrap();
                });
            });
        });

        group.finish();
    }
}

/// Approximate end-to-end benchmark: emit() → BatchLogProcessor → OTLP Exporter → HTTP
///
/// This attempts to measure the complete pipeline including:
/// - LogRecord creation via Logger.emit()
/// - Channel send to BatchLogProcessor
/// - Batch assembly in background thread
/// - OTLP export (conversion, serialization, compression, HTTP)
///
/// Caveats (differs from real production):
/// - Uses force_flush() to synchronize measurement (production uses timer/batch-size triggers)
/// - Fake HTTP server returns 200 OK instantly (no network latency or collector processing)
/// - Emits 511 logs per iteration to avoid auto-export, making force_flush the sole trigger
///
/// Despite these differences, this provides a reasonable approximation of the
/// end-to-end cost for a single batch of logs through the full pipeline.
fn bench_e2e_with_batch_processor(c: &mut Criterion) {
    // Create runtime for async operations
    let rt = Runtime::new().unwrap();

    // Start fake OTLP server on a different port to avoid conflicts
    let port = 14319;
    let _server_handle = rt.block_on(start_fake_otlp_server(port));
    std::thread::sleep(Duration::from_millis(100));

    let endpoint = format!("http://localhost:{}", port);

    // Create OTLP exporter (no compression for baseline)
    let exporter = create_log_exporter(endpoint.clone());

    // Configure BatchLogProcessor:
    // - We emit 511 logs (below max_export_batch_size=512) so NO auto-export is triggered
    // - High scheduled_delay prevents timer-based exports during benchmark run
    // - force_flush() is the ONLY thing that triggers export
    let processor = BatchLogProcessor::builder(exporter)
        .with_batch_config(
            BatchConfigBuilder::default()
                .with_scheduled_delay(Duration::from_secs(30))
                .build(),
        )
        .build();

    let provider = SdkLoggerProvider::builder()
        .with_log_processor(processor)
        .with_resource(
            Resource::builder()
                .with_attributes([
                    KeyValue::new("service.name", "benchmark-service"),
                    KeyValue::new("service.version", "1.0.0"),
                    KeyValue::new("deployment.environment", "production"),
                ])
                .build(),
        )
        .build();

    // Use static targets to avoid lifetime issues with logger()
    static TARGETS: &[&str] = &[
        "target::module_0",
        "target::module_1",
        "target::module_2",
        "target::module_3",
        "target::module_4",
        "target::module_5",
        "target::module_6",
        "target::module_7",
        "target::module_8",
        "target::module_9",
    ];

    let mut group = c.benchmark_group("e2e_batch_processor");

    // Benchmark: emit 511 logs + force_flush (complete round-trip)
    // 511 is below max_export_batch_size (512), so force_flush is the only trigger
    group.bench_function("e2e_batch_511_with_4_attrs", |b| {
        b.iter(|| {
            // Emit 511 logs across 10 targets (below batch threshold)
            for i in 0..511 {
                let target = TARGETS[i % TARGETS.len()];
                let logger = provider.logger(target);
                let mut record = logger.create_log_record();

                record.set_observed_timestamp(now());
                record.set_severity_number(Severity::Info);
                record.set_severity_text("INFO");
                record.set_body(AnyValue::String("Benchmark log message".into()));

                // Add 4 attributes
                for j in 0..4 {
                    record.add_attribute(format!("attr_{}", j), format!("value_{}", j));
                }

                logger.emit(black_box(record));
            }

            // force_flush ensures all logs are exported before continuing
            provider.force_flush().unwrap();
        });
    });

    group.finish();

    // Benchmark with zstd compression
    #[cfg(feature = "zstd-http")]
    {
        let exporter_zstd = create_log_exporter_with_zstd(endpoint.clone());

        let processor_zstd = BatchLogProcessor::builder(exporter_zstd)
            .with_batch_config(
                BatchConfigBuilder::default()
                    .with_scheduled_delay(Duration::from_secs(30))
                    .build(),
            )
            .build();

        let provider_zstd = SdkLoggerProvider::builder()
            .with_log_processor(processor_zstd)
            .with_resource(
                Resource::builder()
                    .with_attributes([
                        KeyValue::new("service.name", "benchmark-service"),
                        KeyValue::new("service.version", "1.0.0"),
                        KeyValue::new("deployment.environment", "production"),
                    ])
                    .build(),
            )
            .build();

        let mut group = c.benchmark_group("e2e_batch_processor_zstd");

        group.bench_function("e2e_batch_511_with_4_attrs_zstd", |b| {
            b.iter(|| {
                for i in 0..511 {
                    let target = TARGETS[i % TARGETS.len()];
                    let logger = provider_zstd.logger(target);
                    let mut record = logger.create_log_record();

                    record.set_observed_timestamp(now());
                    record.set_severity_number(Severity::Info);
                    record.set_severity_text("INFO");
                    record.set_body(AnyValue::String("Benchmark log message".into()));

                    for j in 0..4 {
                        record.add_attribute(format!("attr_{}", j), format!("value_{}", j));
                    }

                    logger.emit(black_box(record));
                }

                provider_zstd.force_flush().unwrap();
            });
        });

        group.finish();

        let _ = provider_zstd.shutdown();
    }

    let _ = provider.shutdown();
}

criterion::criterion_group!(
    benches,
    bench_log_export_pipeline,
    bench_e2e_with_batch_processor
);
criterion::criterion_main!(benches);
