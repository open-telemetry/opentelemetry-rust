#![cfg(unix)]

#[path = "common/metric_counter.rs"]
mod metric_counter;

use metric_counter::test_counter;
use opentelemetry::metrics::noop::NoopMeterProvider;
use opentelemetry_sdk::metrics::{
    data::{AggregatedMetrics, MetricData},
    InMemoryMetricExporter, PeriodicReader, SdkMeterProvider,
};

#[test]
fn counter_uses_its_provider_after_global_provider_changes() {
    let exporter1 = InMemoryMetricExporter::default();
    let provider1 = SdkMeterProvider::builder()
        .with_reader(PeriodicReader::builder(exporter1.clone()).build())
        .build();
    let exporter2 = InMemoryMetricExporter::default();
    let provider2 = SdkMeterProvider::builder()
        .with_reader(PeriodicReader::builder(exporter2.clone()).build())
        .build();

    opentelemetry::global::set_meter_provider(provider2.clone());
    test_counter(&provider1).add(10, &[]);
    opentelemetry::global::set_meter_provider(NoopMeterProvider::new());

    provider1.shutdown().unwrap();
    provider2.shutdown().unwrap();

    let finished = exporter1.get_finished_metrics().unwrap();
    let metric = finished
        .iter()
        .flat_map(|resource| resource.scope_metrics())
        .filter(|scope| scope.scope().name() == "test_meter")
        .flat_map(|scope| scope.metrics())
        .find(|metric| metric.name() == "test_counter")
        .expect("counter was not exported by its provider");
    let AggregatedMetrics::U64(MetricData::Sum(sum)) = metric.data() else {
        panic!("counter was not exported as a u64 sum");
    };
    assert_eq!(
        sum.data_points()
            .map(|point| point.value())
            .collect::<Vec<_>>(),
        vec![10]
    );
    assert!(exporter2.get_finished_metrics().unwrap().is_empty());
}
