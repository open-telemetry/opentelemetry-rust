//! Interfaces for reading and producing metrics
use crate::error::{OTelSdkError, OTelSdkResult};
use opentelemetry::otel_warn;
use std::time::Duration;
use std::{fmt, sync::Weak};

use super::{
    data::{ResourceMetrics, ScopeMetrics},
    instrument::InstrumentKind,
    pipeline::Pipeline,
    Temporality,
};
use crate::Resource;

/// The interface used between the SDK and an exporter.
///
/// Control flow is bi-directional through the `MetricReader`, since the SDK
/// initiates `force_flush` and `shutdown` while the reader initiates
/// collection. The `register_pipeline` method here informs the metric reader
/// that it can begin reading, signaling the start of bi-directional control
/// flow.
///
/// Typically, push-based exporters that are periodic will implement
/// `MetricExporter` themselves and construct a `PeriodicReader` to satisfy this
/// interface.
///
/// Pull-based exporters will typically implement `MetricReader` themselves,
/// since they read on demand.
pub trait MetricReader: fmt::Debug + Send + Sync + 'static {
    /// Registers a [MetricReader] with a [Pipeline].
    ///
    /// The pipeline argument allows the `MetricReader` to signal the sdk to collect
    /// and send aggregated metric measurements.
    fn register_pipeline(&self, pipeline: Weak<Pipeline>);

    /// Gathers and returns all metric data related to the [MetricReader] from the
    /// SDK and stores it in the provided [ResourceMetrics] reference.
    ///
    /// Built-in readers log external [`MetricProducer`] errors and retain
    /// metrics from the SDK and other producers instead of failing collection.
    ///
    /// An error is returned if this is called after shutdown.
    fn collect(&self, rm: &mut ResourceMetrics) -> OTelSdkResult;

    /// Flushes all metric measurements held in an export pipeline.
    ///
    /// There is no guaranteed that all telemetry be flushed or all resources have
    /// been released on error.
    fn force_flush(&self) -> OTelSdkResult;

    /// Flushes all metric measurements held in an export pipeline and releases any
    /// held computational resources.
    ///
    /// There is no guaranteed that all telemetry be flushed or all resources have
    /// been released on error.
    ///
    /// After `shutdown` is called, calls to `collect` will perform no operation and
    /// instead will return an error indicating the shutdown state.
    fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult;

    /// shutdown with default timeout
    fn shutdown(&self) -> OTelSdkResult {
        self.shutdown_with_timeout(Duration::from_secs(5))
    }

    /// The output temporality, a function of instrument kind.
    /// This SHOULD be obtained from the exporter.
    ///
    /// If not configured, the Cumulative temporality SHOULD be used.
    fn temporality(&self, kind: InstrumentKind) -> Temporality;
}

/// Produces metrics for a [MetricReader].
pub(crate) trait SdkProducer: fmt::Debug + Send + Sync {
    /// Returns aggregated metrics from a single collection.
    fn produce(&self, rm: &mut ResourceMetrics) -> OTelSdkResult;
}

/// Produces pre-aggregated metrics from an external source.
///
/// A metric reader invokes registered producers during collection and
/// exports their metrics under the [`Resource`] configured on the
/// [`SdkMeterProvider`](crate::metrics::SdkMeterProvider). The resource is
/// provided to the producer for context; the producer returns only
/// [`ScopeMetrics`].
///
/// This interface is intended for bridging data that has already been
/// aggregated. For new instrumentation, prefer the OpenTelemetry metrics API.
/// External data bypasses SDK views and aggregation. Producers must supply
/// valid metric data, including timestamps and a temporality supported by the
/// exporter; the reader does not convert temporality.
///
/// Producers are called synchronously, so implementations should return
/// promptly and must not call collection, flush, or shutdown on their own
/// reader or provider. Implementations must be safe to call concurrently.
///
/// All registered producers are attempted even if one fails. Built-in readers
/// report producer errors through internal diagnostic warnings (with the
/// `internal-logs` feature enabled) and omit only the failed producer's data
/// for that collection. Metrics from the SDK and successful producers are
/// still collected and exported, including during shutdown.
///
/// Producer errors alone do not cause collection, flush, or shutdown to fail.
/// A successful operation does not guarantee that every producer succeeded.
/// SDK collection and export errors are still returned to the caller.
///
/// # Example
///
/// ```
/// use opentelemetry::InstrumentationScope;
/// use opentelemetry_sdk::{
///     error::OTelSdkError,
///     metrics::{
///         data::{Gauge, GaugeDataPoint, Metric, MetricData, ScopeMetrics},
///         MetricProducer, PeriodicReader,
///         exporter::PushMetricExporter,
///     },
///     Resource,
/// };
/// use std::time::SystemTime;
///
/// #[derive(Debug)]
/// struct ExternalTemperature;
///
/// impl MetricProducer for ExternalTemperature {
///     fn produce(&self, _resource: &Resource) -> Result<Vec<ScopeMetrics>, OTelSdkError> {
///         let gauge = Gauge::builder(
///             vec![GaugeDataPoint::builder(23.5_f64).build()],
///             SystemTime::now(),
///         ).build();
///         Ok(vec![ScopeMetrics::builder()
///             .with_scope(InstrumentationScope::builder("external-bridge").build())
///             .with_metrics(vec![
///                 Metric::builder("temperature", MetricData::from(gauge).into())
///                     .with_unit("Cel")
///                     .build(),
///             ])
///             .build()])
///     }
/// }
///
/// # fn reader<E: PushMetricExporter>(exporter: E) {
/// // Use any push exporter, including an OTLP MetricExporter.
/// let reader = PeriodicReader::builder(exporter)
///     .with_producer(ExternalTemperature)
///     .build();
/// let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
///     .with_reader(reader)
///     .build();
/// # provider.shutdown().unwrap();
/// # }
/// ```
pub trait MetricProducer: fmt::Debug + Send + Sync + 'static {
    /// Returns aggregated metrics from an external source.
    fn produce(&self, resource: &Resource) -> Result<Vec<ScopeMetrics>, OTelSdkError>;
}

pub(crate) fn produce_external(producers: &[Box<dyn MetricProducer>], rm: &mut ResourceMetrics) {
    for (producer_index, producer) in producers.iter().enumerate() {
        match producer.produce(&rm.resource) {
            Ok(scope_metrics) => rm.scope_metrics.extend(scope_metrics),
            Err(error) => {
                otel_warn!(
                    name: "MetricProducer.ProduceFailed",
                    producer_index = producer_index,
                    error = error.to_string(),
                    message = "External producer failed. Metrics from the SDK and other producers will still be collected.",
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{produce_external, MetricProducer};
    use crate::{
        error::OTelSdkError,
        metrics::data::{ResourceMetrics, ScopeMetrics},
        Resource,
    };
    use opentelemetry::{InstrumentationScope, Key, KeyValue};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[derive(Debug)]
    struct TestProducer {
        calls: Arc<AtomicUsize>,
        scope_name: &'static str,
        fail: bool,
    }

    impl MetricProducer for TestProducer {
        fn produce(&self, resource: &Resource) -> Result<Vec<ScopeMetrics>, OTelSdkError> {
            assert_eq!(
                resource.get(&Key::new("service.name")),
                Some("test-service".into())
            );
            self.calls.fetch_add(1, Ordering::Relaxed);
            if self.fail {
                return Err(OTelSdkError::InternalFailure(self.scope_name.to_string()));
            }
            Ok(vec![ScopeMetrics {
                scope: InstrumentationScope::builder(self.scope_name).build(),
                metrics: Vec::new(),
            }])
        }
    }

    fn resource_metrics() -> ResourceMetrics {
        ResourceMetrics {
            resource: Resource::builder_empty()
                .with_attribute(KeyValue::new("service.name", "test-service"))
                .build(),
            scope_metrics: Vec::new(),
        }
    }

    #[test]
    fn external_producers_receive_resource_and_append_scopes() {
        let calls = Arc::new(AtomicUsize::new(0));
        let producers: Vec<Box<dyn MetricProducer>> = vec![
            Box::new(TestProducer {
                calls: calls.clone(),
                scope_name: "first",
                fail: false,
            }),
            Box::new(TestProducer {
                calls: calls.clone(),
                scope_name: "second",
                fail: false,
            }),
        ];
        let mut rm = resource_metrics();

        produce_external(&producers, &mut rm);

        assert_eq!(calls.load(Ordering::Relaxed), 2);
        assert_eq!(rm.scope_metrics.len(), 2);
        assert_eq!(rm.scope_metrics[0].scope.name(), "first");
        assert_eq!(rm.scope_metrics[1].scope.name(), "second");
    }

    #[test]
    fn external_producer_failures_do_not_discard_collected_metrics() {
        let calls = Arc::new(AtomicUsize::new(0));
        let producers: Vec<Box<dyn MetricProducer>> = vec![
            Box::new(TestProducer {
                calls: calls.clone(),
                scope_name: "first failure",
                fail: true,
            }),
            Box::new(TestProducer {
                calls: calls.clone(),
                scope_name: "success",
                fail: false,
            }),
            Box::new(TestProducer {
                calls: calls.clone(),
                scope_name: "second failure",
                fail: true,
            }),
        ];
        let mut rm = resource_metrics();
        rm.scope_metrics.push(ScopeMetrics {
            scope: InstrumentationScope::builder("sdk").build(),
            metrics: Vec::new(),
        });

        produce_external(&producers, &mut rm);

        assert_eq!(calls.load(Ordering::Relaxed), 3);
        assert_eq!(
            rm.scope_metrics()
                .map(|s| s.scope().name())
                .collect::<Vec<_>>(),
            ["sdk", "success"]
        );
    }
}
