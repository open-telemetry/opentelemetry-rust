use opentelemetry::{metrics::MeterProvider, InstrumentationScope, KeyValue};
use opentelemetry_sdk::{
    error::OTelSdkError,
    metrics::{
        data::{Gauge, GaugeDataPoint, Metric, MetricData, ScopeMetrics, Sum, SumDataPoint},
        MetricProducer, PeriodicReader, SdkMeterProvider, Temporality,
    },
    Resource,
};
use std::time::SystemTime;

// A snapshot from a system that already aggregates its own metrics.
#[derive(Debug)]
struct ExternalMetrics {
    start_time: SystemTime,
    requests: u64,
    temperature: f64,
}

impl MetricProducer for ExternalMetrics {
    fn produce(&self, _resource: &Resource) -> Result<Vec<ScopeMetrics>, OTelSdkError> {
        let time = SystemTime::now();
        let requests = Sum::new(
            vec![SumDataPoint::builder(self.requests)
                .with_attributes(vec![KeyValue::new("source", "external-system")])
                .build()],
            Temporality::Cumulative,
            true,
            self.start_time,
            time,
        );
        let temperature = Gauge::builder(
            vec![GaugeDataPoint::builder(self.temperature).build()],
            time,
        )
        .build();

        Ok(vec![ScopeMetrics::builder()
            .with_scope(InstrumentationScope::builder("external-system-bridge").build())
            .with_metrics(vec![
                Metric::builder("external.requests", MetricData::from(requests).into())
                    .with_unit("{request}")
                    .build(),
                Metric::builder("external.temperature", MetricData::from(temperature).into())
                    .with_unit("Cel")
                    .build(),
            ])
            .build()])
    }
}

fn main() -> Result<(), OTelSdkError> {
    let exporter = opentelemetry_stdout::MetricExporterBuilder::default()
        .with_temporality(Temporality::Cumulative)
        .build();
    let reader = PeriodicReader::builder(exporter)
        .with_producer(ExternalMetrics {
            start_time: SystemTime::now(),
            requests: 42,
            temperature: 23.5,
        })
        .build();
    let provider = SdkMeterProvider::builder()
        .with_resource(
            Resource::builder_empty()
                .with_service_name("metric-producer-example")
                .build(),
        )
        .with_reader(reader)
        .build();
    provider
        .meter("application")
        .u64_counter("sdk.requests")
        .build()
        .add(7, &[]);

    // Shutdown collects once and exports SDK and external metrics together.
    provider.shutdown()
}
