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
    - Multiple scopes: 10 different instrumentation scopes (~51 logs per scope)
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
    | batch_512_with_4_attrs          | ~408 µs   | ~797 ns |
    | batch_512_with_10_attrs         | ~716 µs   | ~1.4 µs |
    | batch_512_with_4_attrs_gzip     | ~605 µs   | ~1.2 µs |
    | batch_512_with_10_attrs_gzip    | ~1,070 µs | ~2.1 µs |
    | batch_512_with_4_attrs_zstd     | ~390 µs   | ~762 ns |
    | batch_512_with_10_attrs_zstd    | ~690 µs   | ~1.3 µs |

    Notes:
    - Export time = Conversion + Serialization + Compression (optional) + HTTP stack overhead
*/

use criterion::{black_box, Criterion};
use opentelemetry::logs::{AnyValue, LogRecord as _, Logger, LoggerProvider, Severity};
use opentelemetry::time::now;
use opentelemetry::InstrumentationScope;
use opentelemetry::KeyValue;
#[cfg(any(feature = "gzip-http", feature = "zstd-http"))]
use opentelemetry_otlp::WithHttpConfig;
use opentelemetry_otlp::{LogExporter as OtlpLogExporter, Protocol, WithExportConfig};
use opentelemetry_sdk::logs::{LogBatch, LogExporter, SdkLogRecord, SdkLoggerProvider};
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
    scopes: &[InstrumentationScope],
    batch_size: usize,
    attribute_count: usize,
) -> Vec<Box<(SdkLogRecord, InstrumentationScope)>> {
    // Create a temporary logger just for creating log records
    // The logger's scope doesn't matter since LogBatch uses the scope from the tuple
    let temp_provider = SdkLoggerProvider::builder().build();
    let logger = temp_provider.logger("benchmark");

    let mut log_data = Vec::with_capacity(batch_size);

    for i in 0..batch_size {
        let scope = &scopes[i % scopes.len()];
        let mut record = logger.create_log_record();

        record.set_observed_timestamp(now());
        record.set_timestamp(now());
        record.set_severity_number(Severity::Info);
        record.set_severity_text("INFO");
        record.set_body(AnyValue::String("Benchmark log message".into()));

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

    // Create instrumentation scopes with attributes
    let scopes: Vec<InstrumentationScope> = (0..10)
        .map(|i| {
            InstrumentationScope::builder(format!("component.{}", i))
                .with_version(format!("1.{}.0", i))
                .with_attributes([
                    KeyValue::new("scope.type", "library"),
                    KeyValue::new("scope.id", i.to_string()),
                    KeyValue::new("scope.enabled", true),
                ])
                .build()
        })
        .collect();

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

    // Pre-create log batches for each test case (not measured)
    let log_data_4_attrs = create_log_batch(&scopes, 512, 4);
    let log_data_10_attrs = create_log_batch(&scopes, 512, 10);

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
}

criterion::criterion_group!(benches, bench_log_export_pipeline);
criterion::criterion_main!(benches);
