#![cfg(feature = "testing")]

use opentelemetry::{metrics::MeterProvider, InstrumentationScope, Key};
use opentelemetry_sdk::{
    error::{OTelSdkError, OTelSdkResult},
    metrics::{
        data::{
            AggregatedMetrics, Gauge, GaugeDataPoint, Metric, MetricData, ResourceMetrics,
            ScopeMetrics,
        },
        exporter::PushMetricExporter,
        InMemoryMetricExporterBuilder, MetricProducer, PeriodicReader, SdkMeterProvider,
        Temporality,
    },
    Resource,
};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, SystemTime},
};

#[derive(Debug)]
struct Producer {
    scopes: Vec<&'static str>,
    calls: Arc<AtomicUsize>,
    fail: Arc<AtomicBool>,
}

impl MetricProducer for Producer {
    fn produce(&self, resource: &Resource) -> Result<Vec<ScopeMetrics>, OTelSdkError> {
        assert_eq!(
            resource.get(&Key::new("service.name")),
            Some("external-test".into())
        );
        self.calls.fetch_add(1, Ordering::Relaxed);
        if self.fail.load(Ordering::Relaxed) {
            return Err(OTelSdkError::InternalFailure(
                "external source unavailable".into(),
            ));
        }
        Ok(self
            .scopes
            .iter()
            .map(|name| {
                let gauge = Gauge::builder(
                    vec![GaugeDataPoint::builder(42_u64).build()],
                    SystemTime::UNIX_EPOCH + Duration::from_secs(10),
                )
                .build();
                ScopeMetrics::builder()
                    .with_scope(InstrumentationScope::builder(*name).build())
                    .with_metrics(vec![Metric::builder(
                        "external.value",
                        MetricData::from(gauge).into(),
                    )
                    .build()])
                    .build()
            })
            .collect())
    }
}

fn provider_with_producers(
    async_runtime: bool,
    exporter: impl PushMetricExporter,
    producers: Vec<Producer>,
) -> SdkMeterProvider {
    let builder = SdkMeterProvider::builder().with_resource(
        Resource::builder_empty()
            .with_service_name("external-test")
            .build(),
    );
    if async_runtime {
        #[cfg(all(
            feature = "experimental_metrics_periodicreader_with_async_runtime",
            feature = "rt-tokio-current-thread"
        ))]
        {
            use opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader;

            let mut reader =
                PeriodicReader::builder(exporter, opentelemetry_sdk::runtime::TokioCurrentThread)
                    .with_interval(Duration::from_secs(3600));
            for producer in producers {
                reader = reader.with_producer(producer);
            }
            builder.with_reader(reader.build()).build()
        }
        #[cfg(not(all(
            feature = "experimental_metrics_periodicreader_with_async_runtime",
            feature = "rt-tokio-current-thread"
        )))]
        unreachable!("async test requires a runtime");
    } else {
        let mut reader = PeriodicReader::builder(exporter).with_interval(Duration::from_secs(3600));
        for producer in producers {
            reader = reader.with_producer(producer);
        }
        builder.with_reader(reader.build()).build()
    }
}

fn exercise_reader(async_runtime: bool, temporality: Temporality) {
    let exporter = InMemoryMetricExporterBuilder::new()
        .with_temporality(temporality)
        .build();
    let calls = Arc::new(AtomicUsize::new(0));
    let fail = Arc::new(AtomicBool::new(false));
    let other_fail = Arc::new(AtomicBool::new(false));
    let producer = |scopes, fail| Producer {
        scopes,
        calls: calls.clone(),
        fail,
    };
    let provider = provider_with_producers(
        async_runtime,
        exporter.clone(),
        vec![
            producer(vec!["first", "second"], fail.clone()),
            producer(vec![], other_fail.clone()),
            producer(vec!["third"], other_fail.clone()),
        ],
    );
    let counter = provider.meter("sdk").u64_counter("sdk.value").build();
    for value in [7, 3] {
        counter.add(value, &[]);
        provider.force_flush().unwrap();
    }
    fail.store(true, Ordering::Relaxed);
    counter.add(999, &[]);
    provider.force_flush().unwrap();
    fail.store(false, Ordering::Relaxed);
    counter.add(1, &[]);
    provider.force_flush().unwrap();
    fail.store(true, Ordering::Relaxed);
    other_fail.store(true, Ordering::Relaxed);
    counter.add(5, &[]);
    provider.shutdown().unwrap();

    assert_eq!(calls.load(Ordering::Relaxed), 15);
    assert!(provider.force_flush().is_err());
    assert_eq!(calls.load(Ordering::Relaxed), 15);

    let batches = exporter.get_finished_metrics().unwrap();
    assert_eq!(batches.len(), 5);
    let expected_values = if temporality == Temporality::Delta {
        [7, 3, 999, 1, 5]
    } else {
        [7, 10, 1009, 1010, 1015]
    };
    let mut sdk_values = Vec::new();
    for (index, batch) in batches.iter().enumerate() {
        let scopes: Vec<_> = batch.scope_metrics().collect();
        let expected_scopes: &[&str] = if index == 4 {
            &["sdk"]
        } else if index == 2 {
            &["sdk", "third"]
        } else {
            &["sdk", "first", "second", "third"]
        };
        assert_eq!(
            scopes.iter().map(|s| s.scope().name()).collect::<Vec<_>>(),
            expected_scopes,
        );
        assert_eq!(
            batch.resource().get(&Key::new("service.name")),
            Some("external-test".into())
        );
        let sdk_metrics: Vec<_> = scopes[0].metrics().collect();
        assert_eq!(sdk_metrics.len(), 1);
        assert_eq!(sdk_metrics[0].name(), "sdk.value");
        let AggregatedMetrics::U64(MetricData::Sum(sum)) = sdk_metrics[0].data() else {
            panic!("expected SDK counter");
        };
        assert_eq!(sum.temporality(), temporality);
        assert!(sum.is_monotonic());
        let points: Vec<_> = sum.data_points().collect();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].value(), expected_values[index]);
        sdk_values.push(points[0].value());
        for scope in &scopes[1..] {
            let metrics: Vec<_> = scope.metrics().collect();
            assert_eq!(metrics.len(), 1);
            let AggregatedMetrics::U64(MetricData::Gauge(gauge)) = metrics[0].data() else {
                panic!("expected external gauge");
            };
            assert_eq!(gauge.data_points().next().unwrap().value(), 42);
        }
    }
    if temporality == Temporality::Delta {
        assert_eq!(sdk_values.iter().sum::<u64>(), 1015);
    }
}

fn exercise_empty_collection(async_runtime: bool) {
    let exporter = InMemoryMetricExporterBuilder::new()
        .with_temporality(Temporality::Delta)
        .build();
    let fail = Arc::new(AtomicBool::new(false));
    let calls = Arc::new(AtomicUsize::new(0));
    let provider = provider_with_producers(
        async_runtime,
        exporter.clone(),
        vec![Producer {
            scopes: vec!["external"],
            fail: fail.clone(),
            calls: calls.clone(),
        }],
    );
    let counter = provider.meter("sdk").u64_counter("sdk.value").build();
    counter.add(7, &[]);
    provider.force_flush().unwrap();
    assert_eq!(exporter.get_finished_metrics().unwrap().len(), 1);
    fail.store(true, Ordering::Relaxed);
    for _ in 0..2 {
        provider.force_flush().unwrap();
        assert_eq!(exporter.get_finished_metrics().unwrap().len(), 1);
    }
    fail.store(false, Ordering::Relaxed);
    provider.force_flush().unwrap();
    let batches = exporter.get_finished_metrics().unwrap();
    assert_eq!(batches.len(), 2);
    assert_eq!(
        batches[1]
            .scope_metrics()
            .map(|s| s.scope().name())
            .collect::<Vec<_>>(),
        ["external"]
    );
    fail.store(true, Ordering::Relaxed);
    provider.shutdown().unwrap();
    assert_eq!(exporter.get_finished_metrics().unwrap().len(), 2);
    assert_eq!(calls.load(Ordering::Relaxed), 5);
}

#[derive(Clone, Debug, Default)]
struct FailingExporter {
    attempts: Arc<AtomicUsize>,
    shutdowns: Arc<AtomicUsize>,
}

impl PushMetricExporter for FailingExporter {
    async fn export(&self, _metrics: &ResourceMetrics) -> OTelSdkResult {
        self.attempts.fetch_add(1, Ordering::Relaxed);
        Err(OTelSdkError::InternalFailure("export failed".into()))
    }

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        self.shutdowns.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn temporality(&self) -> Temporality {
        Temporality::Delta
    }
}

fn exercise_exporter_error(async_runtime: bool) {
    let exporter = FailingExporter::default();
    let calls = Arc::new(AtomicUsize::new(0));
    let provider = provider_with_producers(
        async_runtime,
        exporter.clone(),
        vec![Producer {
            scopes: vec!["external"],
            calls: calls.clone(),
            fail: Arc::new(AtomicBool::new(true)),
        }],
    );
    let counter = provider.meter("sdk").u64_counter("sdk.value").build();
    counter.add(1, &[]);
    assert!(provider.force_flush().is_err());
    assert_eq!(exporter.attempts.load(Ordering::Relaxed), 1);
    counter.add(2, &[]);
    assert!(provider.shutdown().is_err());
    assert_eq!(exporter.attempts.load(Ordering::Relaxed), 2);
    assert_eq!(exporter.shutdowns.load(Ordering::Relaxed), 1);
    assert_eq!(calls.load(Ordering::Relaxed), 2);
}

#[test]
fn periodic_reader_preserves_metrics_when_producers_fail() {
    for temporality in [Temporality::Delta, Temporality::Cumulative] {
        exercise_reader(false, temporality);
    }
}

#[test]
fn periodic_reader_does_not_replay_stale_metrics_when_producers_fail() {
    exercise_empty_collection(false);
}

#[test]
fn periodic_reader_still_reports_exporter_errors_when_producers_fail() {
    exercise_exporter_error(false);
}

#[cfg(all(
    feature = "experimental_metrics_periodicreader_with_async_runtime",
    feature = "rt-tokio-current-thread"
))]
#[test]
fn async_reader_preserves_metrics_when_producers_fail() {
    for temporality in [Temporality::Delta, Temporality::Cumulative] {
        exercise_reader(true, temporality);
    }
}

#[cfg(all(
    feature = "experimental_metrics_periodicreader_with_async_runtime",
    feature = "rt-tokio-current-thread"
))]
#[test]
fn async_reader_does_not_replay_stale_metrics_when_producers_fail() {
    exercise_empty_collection(true);
}

#[cfg(all(
    feature = "experimental_metrics_periodicreader_with_async_runtime",
    feature = "rt-tokio-current-thread"
))]
#[test]
fn async_reader_still_reports_exporter_errors_when_producers_fail() {
    exercise_exporter_error(true);
}
