//! Metrics from sources outside the SDK.
use crate::error::OTelSdkError;
use opentelemetry::otel_warn;
use std::fmt;

use super::data::{ResourceMetrics, ScopeMetrics};
#[cfg(doc)]
use super::reader::MetricReader;

/// Produces metrics for a [MetricReader] from a source outside the SDK, such as a
/// bridge to another metrics library, or measurements already aggregated elsewhere.
///
/// Register a producer with a reader's builder (`with_producer`). Each collection
/// of that reader appends what the producer returns after the SDK's own metrics.
/// The producer chooses the temporality of the data it returns.
///
/// See the [MetricProducer] specification.
///
/// [MetricProducer]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/sdk.md#metricproducer
#[cfg_attr(not(feature = "experimental_metrics_producer"), allow(unreachable_pub))]
pub trait MetricProducer: fmt::Debug + Send + Sync {
    /// Returns the producer's current metrics, grouped by instrumentation scope.
    fn produce(&self) -> Result<Vec<ScopeMetrics>, OTelSdkError>;
}

/// Appends the output of each producer to `rm`. A producer that fails is logged
/// and skipped, so it cannot drop the SDK's own metrics or other producers' output.
pub(crate) fn produce_external(producers: &[Box<dyn MetricProducer>], rm: &mut ResourceMetrics) {
    for producer in producers {
        match producer.produce() {
            Ok(scope_metrics) => rm.scope_metrics.extend(scope_metrics),
            Err(err) => {
                otel_warn!(
                    name: "MetricReader.ProducerFailed",
                    producer = format!("{producer:?}"),
                    error = format!("{err}"),
                );
            }
        }
    }
}

#[cfg(all(test, feature = "experimental_metrics_producer"))]
mod tests {
    use super::*;
    use crate::error::OTelSdkResult;
    use crate::metrics::data::{
        AggregatedMetrics, Histogram, HistogramDataPoint, Metric, MetricData,
    };
    use crate::metrics::pipeline::Pipeline;
    use crate::metrics::reader::MetricReader;
    use crate::metrics::{
        InMemoryMetricExporter, InstrumentKind, ManualReader, PeriodicReader, SdkMeterProvider,
        Temporality,
    };
    use crate::Resource;
    use opentelemetry::metrics::MeterProvider as _;
    use opentelemetry::time::now;
    use opentelemetry::{InstrumentationScope, KeyValue};
    use std::sync::{Arc, Weak};
    use std::time::Duration;

    /// A histogram aggregated outside the SDK.
    #[derive(Debug)]
    struct External;

    impl MetricProducer for External {
        fn produce(&self) -> Result<Vec<ScopeMetrics>, OTelSdkError> {
            let point = HistogramDataPoint::new(
                vec![KeyValue::new("op", "put")],
                vec![10.0, 100.0],
                vec![1, 2, 3],
                1234u64,
            );
            let histogram = Histogram::new(vec![point], now(), now(), Temporality::Cumulative);
            let metric = Metric::new("latency", "", "us", MetricData::from(histogram));
            let scope = InstrumentationScope::builder("external").build();
            Ok(vec![ScopeMetrics::new(scope, vec![metric])])
        }
    }

    #[derive(Debug)]
    struct Failing;

    impl MetricProducer for Failing {
        fn produce(&self) -> Result<Vec<ScopeMetrics>, OTelSdkError> {
            Err(OTelSdkError::InternalFailure("unavailable".into()))
        }
    }

    /// Lets the test collect from a [ManualReader] that the provider also holds.
    #[derive(Debug, Clone)]
    struct SharedReader(Arc<ManualReader>);

    impl MetricReader for SharedReader {
        fn register_pipeline(&self, pipeline: Weak<Pipeline>) {
            self.0.register_pipeline(pipeline)
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

    fn record_sdk_metric(provider: &SdkMeterProvider) {
        provider
            .meter("sdk")
            .u64_counter("requests")
            .build()
            .add(1, &[]);
    }

    /// Checks that `rm` holds the SDK's metrics, then [External]'s and nothing else.
    fn assert_sdk_then_external(rm: &ResourceMetrics) {
        let scopes: Vec<_> = rm.scope_metrics().map(|sm| sm.scope().name()).collect();
        assert_eq!(scopes, ["sdk", "external"]);
        let metric = rm.scope_metrics().nth(1).unwrap().metrics().next().unwrap();
        assert_eq!((metric.name(), metric.unit()), ("latency", "us"));
        let AggregatedMetrics::U64(MetricData::Histogram(histogram)) = metric.data() else {
            panic!("not a u64 histogram: {:?}", metric.data());
        };
        let point = histogram.data_points().next().unwrap();
        assert_eq!(point.bounds().collect::<Vec<_>>(), [10.0, 100.0]);
        assert_eq!(point.bucket_counts().collect::<Vec<_>>(), [1, 2, 3]);
        assert_eq!((point.count(), point.sum()), (6, 1234));
    }

    #[test]
    fn producer_metrics_follow_the_sdk_metrics() {
        let exporter = InMemoryMetricExporter::default();
        let reader = PeriodicReader::builder(exporter.clone())
            .with_producer(Failing)
            .with_producer(External)
            .build();
        let provider = SdkMeterProvider::builder().with_reader(reader).build();
        record_sdk_metric(&provider);

        // The reader reuses its ResourceMetrics: the second export must not carry the
        // first one's producer output too.
        provider.force_flush().unwrap();
        provider.force_flush().unwrap();

        let exports = exporter.get_finished_metrics().unwrap();
        assert_eq!(exports.len(), 2);
        exports.iter().for_each(assert_sdk_then_external);
    }

    #[test]
    fn manual_reader_appends_producer_metrics() {
        let reader = SharedReader(Arc::new(
            ManualReader::builder()
                .with_producer(Failing)
                .with_producer(External)
                .build(),
        ));
        let provider = SdkMeterProvider::builder()
            .with_reader(reader.clone())
            .build();
        record_sdk_metric(&provider);

        // The same ResourceMetrics collected into twice must not accumulate producer
        // output.
        let mut rm = ResourceMetrics {
            resource: Resource::empty(),
            scope_metrics: Vec::new(),
        };
        reader.collect(&mut rm).unwrap();
        reader.collect(&mut rm).unwrap();
        assert_sdk_then_external(&rm);
    }

    #[cfg(all(
        feature = "experimental_metrics_periodicreader_with_async_runtime",
        feature = "rt-tokio-current-thread"
    ))]
    #[test]
    fn async_runtime_periodic_reader_appends_producer_metrics() {
        use crate::metrics::periodic_reader_with_async_runtime::PeriodicReader;
        use crate::runtime;

        let exporter = InMemoryMetricExporter::default();
        let reader = PeriodicReader::builder(exporter.clone(), runtime::TokioCurrentThread)
            .with_producer(Failing)
            .with_producer(External)
            .build();
        let provider = SdkMeterProvider::builder().with_reader(reader).build();
        record_sdk_metric(&provider);

        provider.force_flush().unwrap();
        provider.force_flush().unwrap();

        let exports = exporter.get_finished_metrics().unwrap();
        assert_eq!(exports.len(), 2);
        exports.iter().for_each(assert_sdk_then_external);
    }
}
