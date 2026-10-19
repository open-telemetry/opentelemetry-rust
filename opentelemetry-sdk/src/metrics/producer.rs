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
    use crate::metrics::data::{
        AggregatedMetrics, Histogram, HistogramDataPoint, Metric, MetricData,
    };
    use crate::metrics::{InMemoryMetricExporter, PeriodicReader, SdkMeterProvider, Temporality};
    use opentelemetry::metrics::MeterProvider as _;
    use opentelemetry::time::now;
    use opentelemetry::{InstrumentationScope, KeyValue};

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

    #[test]
    fn producer_metrics_follow_the_sdk_metrics() {
        let exporter = InMemoryMetricExporter::default();
        let reader = PeriodicReader::builder(exporter.clone())
            .with_producer(Failing)
            .with_producer(External)
            .build();
        let provider = SdkMeterProvider::builder().with_reader(reader).build();
        provider
            .meter("sdk")
            .u64_counter("requests")
            .build()
            .add(1, &[]);

        // The reader reuses its ResourceMetrics: the second export must not carry the
        // first one's producer output too.
        provider.force_flush().unwrap();
        provider.force_flush().unwrap();

        let exports = exporter.get_finished_metrics().unwrap();
        assert_eq!(exports.len(), 2);
        for rm in &exports {
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
    }
}
