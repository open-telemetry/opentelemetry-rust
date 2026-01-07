/*
    The benchmark results:
    criterion = "0.5"
    OS: macOS
    Hardware: Apple Silicon

    | Test                                        | Average time|
    |---------------------------------------------|-------------|
    | 1_metric_1000_datapoints_3_attrs            | ~205 µs     |

    Note: Benchmark measures conversion of 1 metric with 1000 unique datapoints,
    each with 3 attributes (2 fixed + 1 varying), to protobuf format.
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
    let mut group = c.benchmark_group("metrics_batch_conversion");

    // 1 metric with 1000 data points, 3 attributes per data point
    group.bench_function("1_metric_1000_datapoints_3_attrs", |b| {
        let resource_metrics = create_test_metrics(1000);

        b.iter(|| {
            let _proto: ExportMetricsServiceRequest = black_box(&resource_metrics).into();
            black_box(_proto);
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
