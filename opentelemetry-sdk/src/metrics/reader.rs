//! Interfaces for reading and producing metrics
use opentelemetry::InstrumentationScope;

use crate::error::OTelSdkResult;
use crate::Resource;
use std::time::Duration;
use std::{fmt, sync::Weak};

use super::data::Metric;
use super::{pipeline::Pipeline, InstrumentKind, Temporality};

/// A collection of [ScopeMetricsData] and the associated [Resource] that created them.
#[derive(Debug)]
pub struct ResourceMetricsData {
    /// The entity that collected the metrics.
    pub resource: Resource,
    /// The collection of metrics with unique [InstrumentationScope]s.
    pub scope_metrics: Vec<ScopeMetricsData>,
}

/// A collection of metrics produced by a meter.
#[derive(Debug, Default)]
pub struct ScopeMetricsData {
    /// The [InstrumentationScope] that the meter was created with.
    pub scope: InstrumentationScope,
    /// The list of aggregations created by the meter.
    pub metrics: Vec<Metric>,
}

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
    /// SDK and stores it in the provided [ResourceMetricsData] reference.
    ///
    /// An error is returned if this is called after shutdown.
    fn collect(&self, rm: &mut ResourceMetricsData) -> OTelSdkResult;

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
    fn produce(&self, rm: &mut ResourceMetricsData) -> OTelSdkResult;
}
