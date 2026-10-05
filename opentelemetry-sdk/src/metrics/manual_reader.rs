use opentelemetry::otel_debug;
use std::time::Duration;
use std::{
    fmt,
    sync::{Mutex, Weak},
};

use crate::{
    error::{OTelSdkError, OTelSdkResult},
    metrics::Temporality,
};

use super::{
    data::ResourceMetrics,
    pipeline::Pipeline,
    reader::{produce_external, MetricProducer, MetricReader, SdkProducer},
};

/// A simple [MetricReader] that allows an application to read metrics on demand.
///
/// See [ManualReaderBuilder] for configuration options.
///
/// # Example
///
/// ```
/// use opentelemetry_sdk::metrics::ManualReader;
///
/// // can specify additional reader configuration
/// let reader = ManualReader::builder().build();
/// # drop(reader)
/// ```
pub struct ManualReader {
    inner: Mutex<ManualReaderInner>,
    temporality: Temporality,
    external_producers: Vec<Box<dyn MetricProducer>>,
}

impl Default for ManualReader {
    fn default() -> Self {
        ManualReader::builder().build()
    }
}

impl fmt::Debug for ManualReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ManualReader")
    }
}

#[derive(Debug)]
struct ManualReaderInner {
    sdk_producer: Option<Weak<dyn SdkProducer>>,
    is_shutdown: bool,
}

impl ManualReader {
    /// Configuration for this reader
    pub fn builder() -> ManualReaderBuilder {
        ManualReaderBuilder::default()
    }

    /// A [MetricReader] which is directly called to collect metrics.
    pub(crate) fn new(
        temporality: Temporality,
        external_producers: Vec<Box<dyn MetricProducer>>,
    ) -> Self {
        ManualReader {
            inner: Mutex::new(ManualReaderInner {
                sdk_producer: None,
                is_shutdown: false,
            }),
            temporality,
            external_producers,
        }
    }
}

impl MetricReader for ManualReader {
    ///  Register a pipeline which enables the caller to read metrics from the SDK
    ///  on demand.
    fn register_pipeline(&self, pipeline: Weak<Pipeline>) {
        let _ = self.inner.lock().map(|mut inner| {
            // Only register once. If producer is already set, do nothing.
            if inner.sdk_producer.is_none() {
                inner.sdk_producer = Some(pipeline);
            } else {
                otel_debug!(
                    name: "ManualReader.DuplicateRegistration",
                    message = "The pipeline is already registered to the Reader. Registering pipeline multiple times is not allowed.");
            }
        });
    }

    /// Gathers all metrics from the SDK, calling any
    /// callbacks necessary and returning the results.
    ///
    /// Returns an error if called after shutdown.
    fn collect(&self, rm: &mut ResourceMetrics) -> OTelSdkResult {
        let producer = {
            let inner = self
                .inner
                .lock()
                .map_err(|_| OTelSdkError::InternalFailure("Failed to lock pipeline".into()))?;
            inner
                .sdk_producer
                .as_ref()
                .and_then(|producer| producer.upgrade())
                .ok_or_else(|| {
                    OTelSdkError::InternalFailure("reader is shut down or not registered".into())
                })?
        };
        producer.produce(rm)?;

        produce_external(&self.external_producers, rm);
        Ok(())
    }

    /// ForceFlush is a no-op, it always returns nil.
    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    /// Closes any connections and frees any resources used by the reader.
    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        let mut inner = self
            .inner
            .lock()
            .map_err(|e| OTelSdkError::InternalFailure(format!("Failed to acquire lock: {e}")))?;

        // Any future call to collect will now return an error.
        inner.sdk_producer = None;
        inner.is_shutdown = true;

        Ok(())
    }

    fn temporality(&self, kind: super::InstrumentKind) -> Temporality {
        kind.temporality_preference(self.temporality)
    }
}

/// Configuration for a [ManualReader]
#[derive(Default)]
pub struct ManualReaderBuilder {
    temporality: Temporality,
    producers: Vec<Box<dyn MetricProducer>>,
}

impl fmt::Debug for ManualReaderBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ManualReaderBuilder")
    }
}

impl ManualReaderBuilder {
    /// New manual builder configuration
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the [Temporality] of the exporter.
    pub fn with_temporality(mut self, temporality: Temporality) -> Self {
        self.temporality = temporality;
        self
    }

    /// Registers an external [`MetricProducer`] with this reader.
    ///
    /// The producer supplies pre-aggregated metrics that are collected under
    /// the SDK [`Resource`](crate::Resource) alongside metrics collected from SDK instruments.
    pub fn with_producer(mut self, producer: impl MetricProducer + 'static) -> Self {
        self.producers.push(Box::new(producer));
        self
    }

    /// Create a new [ManualReader] from this configuration.
    pub fn build(self) -> ManualReader {
        ManualReader::new(self.temporality, self.producers)
    }
}

#[cfg(test)]
mod tests {
    use super::ManualReader;
    use crate::{
        error::{OTelSdkError, OTelSdkResult},
        metrics::{
            data::{
                AggregatedMetrics, Gauge, GaugeDataPoint, Metric, MetricData, ResourceMetrics,
                ScopeMetrics,
            },
            pipeline::Pipeline,
            reader::{MetricProducer, MetricReader},
            InstrumentKind, SdkMeterProvider, Temporality,
        },
        Resource,
    };
    use opentelemetry::{metrics::MeterProvider, InstrumentationScope, Key, KeyValue};
    use std::{
        sync::{
            atomic::{AtomicBool, AtomicUsize, Ordering},
            Arc, Weak,
        },
        time::{Duration, SystemTime},
    };

    #[derive(Debug)]
    struct TestMetricProducer {
        scope_name: &'static str,
        calls: Arc<AtomicUsize>,
        fail: Arc<AtomicBool>,
    }

    impl MetricProducer for TestMetricProducer {
        fn produce(&self, resource: &Resource) -> Result<Vec<ScopeMetrics>, OTelSdkError> {
            assert_eq!(
                resource.get(&Key::new("service.name")),
                Some("test-service".into())
            );
            self.calls.fetch_add(1, Ordering::Relaxed);
            if self.fail.load(Ordering::Relaxed) {
                return Err(OTelSdkError::InternalFailure(
                    "external source unavailable".into(),
                ));
            }
            let gauge = Gauge::builder(
                vec![GaugeDataPoint::builder(42_u64).build()],
                SystemTime::now(),
            )
            .build();
            Ok(vec![ScopeMetrics::builder()
                .with_scope(InstrumentationScope::builder(self.scope_name).build())
                .with_metrics(vec![Metric::builder(
                    "external.value",
                    MetricData::from(gauge).into(),
                )
                .build()])
                .build()])
        }
    }

    #[derive(Clone, Debug)]
    struct SharedReader(Arc<ManualReader>);

    impl MetricReader for SharedReader {
        fn register_pipeline(&self, pipeline: Weak<Pipeline>) {
            self.0.register_pipeline(pipeline);
        }

        fn collect(&self, rm: &mut ResourceMetrics) -> OTelSdkResult {
            self.0.collect(rm)
        }

        fn force_flush(&self) -> OTelSdkResult {
            self.0.force_flush()
        }

        fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult {
            self.0.shutdown_with_timeout(timeout)
        }

        fn temporality(&self, kind: InstrumentKind) -> Temporality {
            self.0.temporality(kind)
        }
    }

    #[test]
    fn collects_sdk_delta_metrics_despite_external_producer_failures() {
        let calls = Arc::new(AtomicUsize::new(0));
        let fail = Arc::new(AtomicBool::new(false));
        let reader = SharedReader(Arc::new(
            ManualReader::builder()
                .with_temporality(Temporality::Delta)
                .with_producer(TestMetricProducer {
                    scope_name: "first",
                    calls: calls.clone(),
                    fail: fail.clone(),
                })
                .with_producer(TestMetricProducer {
                    scope_name: "second",
                    calls: calls.clone(),
                    fail: Arc::new(AtomicBool::new(false)),
                })
                .build(),
        ));
        let mut rm = ResourceMetrics::default();
        assert!(reader.collect(&mut rm).is_err());
        assert_eq!(calls.load(Ordering::Relaxed), 0);
        let meter_provider = SdkMeterProvider::builder()
            .with_resource(
                Resource::builder_empty()
                    .with_attribute(KeyValue::new("service.name", "test-service"))
                    .build(),
            )
            .with_reader(reader.clone())
            .build();
        let counter = meter_provider.meter("sdk").u64_counter("sdk.value").build();

        let mut sdk_values = Vec::new();
        for (value, expected_calls) in [(7, 2), (3, 4)] {
            counter.add(value, &[]);
            reader.collect(&mut rm).unwrap();
            assert_eq!(calls.load(Ordering::Relaxed), expected_calls);
            let scopes: Vec<_> = rm.scope_metrics().collect();
            assert_eq!(
                scopes.iter().map(|s| s.scope().name()).collect::<Vec<_>>(),
                ["sdk", "first", "second"]
            );
            let sdk_metric = scopes[0].metrics().next().unwrap();
            let AggregatedMetrics::U64(MetricData::Sum(sum)) = sdk_metric.data() else {
                panic!("expected SDK counter");
            };
            assert_eq!(sum.temporality(), Temporality::Delta);
            let sdk_value = sum.data_points().next().unwrap().value();
            assert_eq!(sdk_value, value);
            sdk_values.push(sdk_value);
            for scope in &scopes[1..] {
                let metric = scope.metrics().next().unwrap();
                let AggregatedMetrics::U64(MetricData::Gauge(gauge)) = metric.data() else {
                    panic!("expected external gauge");
                };
                assert_eq!(gauge.data_points().next().unwrap().value(), 42);
            }
        }

        fail.store(true, Ordering::Relaxed);
        counter.add(999, &[]);
        reader.collect(&mut rm).unwrap();
        assert_eq!(calls.load(Ordering::Relaxed), 6);
        assert_eq!(
            rm.scope_metrics()
                .map(|s| s.scope().name())
                .collect::<Vec<_>>(),
            ["sdk", "second"]
        );
        let sdk_metric = rm.scope_metrics().next().unwrap().metrics().next().unwrap();
        let AggregatedMetrics::U64(MetricData::Sum(sum)) = sdk_metric.data() else {
            panic!("expected SDK counter");
        };
        sdk_values.push(sum.data_points().next().unwrap().value());
        assert_eq!(sdk_values, [7, 3, 999]);
        assert_eq!(sdk_values.iter().sum::<u64>(), 1009);
        meter_provider.shutdown().unwrap();
        assert!(reader.collect(&mut rm).is_err());
        assert_eq!(calls.load(Ordering::Relaxed), 6);
    }
}
