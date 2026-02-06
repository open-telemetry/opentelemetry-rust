/*
    The benchmark results - Complete Pipeline Breakdown:
    criterion = "0.5"
    OS: macOS
    Hardware: Apple Silicon

    Pipeline Stages:
    1. Conversion: OTel struct → Protobuf struct
    2. Serialization: Protobuf struct → bytes (prost::Message::encode_to_vec())
    3. Compression: bytes → gzip compressed bytes

    | Test                        | Conversion | Serialization | Compression | Total   |
    |-----------------------------|------------|---------------|-------------|---------|
    | 1_metric_1000_datapoints    | ~207 µs    | ~109 µs       | ~199 µs     | ~515 µs |

    Note: Benchmark measures 1 metric with 1000 unique datapoints, each with 3 attributes.
    Key Insight: Cost is balanced across stages - Conversion (40%), Serialization (21%), Compression (39%).
*/

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use opentelemetry::metrics::MeterProvider;
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::metrics::{InMemoryMetricExporter, PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::Resource;
use std::time::Duration;

#[cfg(feature = "gen-tonic-messages")]
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;

#[cfg(feature = "gen-tonic-messages")]
use prost::Message;

fn create_meter_provider_with_in_memory_exporter() -> (SdkMeterProvider, InMemoryMetricExporter) {
    let exporter = InMemoryMetricExporter::default();
    let reader = PeriodicReader::builder(exporter.clone())
        .with_interval(Duration::from_millis(100))
        .build();

    let resource = Resource::builder()
        .with_service_name("benchmark-service")
        .with_attributes(vec![
            KeyValue::new("host.name", "benchmark-host"),
            KeyValue::new("environment", "production"),
        ])
        .build();

    let provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resource)
        .build();

    (provider, exporter)
}

fn record_metrics_batch(provider: &SdkMeterProvider, data_points_per_metric: usize) {
    let meter = provider.meter("benchmark-meter");

    // Create a single counter metric
    let counter = meter.u64_counter("http.server.duration").build();

    // Record data points with different attribute sets
    // 2 fixed attributes + 1 varying attribute to ensure all datapoints are unique
    for i in 0..data_points_per_metric {
        let attrs = vec![
            KeyValue::new("http.method", "GET"),
            KeyValue::new("http.route", "/api/users"),
            KeyValue::new("http.url", format!("https://api.example.com/users/{}", i)),
        ];

        counter.add(1, &attrs);
    }
}

fn collect_metrics(exporter: &InMemoryMetricExporter) -> Vec<ResourceMetrics> {
    exporter.get_finished_metrics().unwrap_or_default()
}

fn create_test_metrics(data_points_per_metric: usize) -> ResourceMetrics {
    let (provider, exporter) = create_meter_provider_with_in_memory_exporter();
    record_metrics_batch(&provider, data_points_per_metric);
    provider.force_flush().unwrap();
    let metrics_vec = collect_metrics(&exporter);
    metrics_vec.into_iter().next().unwrap()
}

#[cfg(feature = "gen-tonic-messages")]
fn bench_metrics_conversion(c: &mut Criterion) {
    // Step 1: OTel struct to Protobuf struct
    let mut group = c.benchmark_group("metrics_conversion");

    group.bench_function("1_metric_1000_datapoints_to_proto", |b| {
        let resource_metrics = create_test_metrics(1000);

        b.iter(|| {
            let _proto: ExportMetricsServiceRequest = black_box(&resource_metrics).into();
            black_box(_proto);
        });
    });

    group.finish();

    // Step 2: Protobuf struct to bytes
    let mut group = c.benchmark_group("metrics_serialization");

    group.bench_function("1_metric_1000_datapoints_to_bytes", |b| {
        let resource_metrics = create_test_metrics(1000);
        let proto: ExportMetricsServiceRequest = (&resource_metrics).into();

        b.iter(|| {
            let bytes = black_box(&proto).encode_to_vec();
            black_box(bytes);
        });
    });

    group.finish();

    // Step 3: Bytes to compressed bytes (gzip)
    let mut group = c.benchmark_group("metrics_compression");

    group.bench_function("1_metric_1000_datapoints_compress", |b| {
        let resource_metrics = create_test_metrics(1000);
        let proto: ExportMetricsServiceRequest = (&resource_metrics).into();
        let bytes = proto.encode_to_vec();

        b.iter(|| {
            use flate2::{write::GzEncoder, Compression};
            use std::io::Write;
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(black_box(&bytes)).unwrap();
            let compressed = encoder.finish().unwrap();
            black_box(compressed);
        });
    });

    group.finish();
}

#[cfg(not(feature = "gen-tonic-messages"))]
fn bench_metrics_conversion(_c: &mut Criterion) {
    // Benchmark is only available when gen-tonic-messages feature is enabled
}

criterion_group!(benches, bench_metrics_conversion);
criterion_main!(benches);
