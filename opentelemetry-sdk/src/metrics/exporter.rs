//! Interfaces for exporting metrics

use opentelemetry::InstrumentationScope;

use crate::{error::OTelSdkResult, Resource};
use std::{fmt::Debug, slice::Iter, time::Duration};

use super::{
    data::AggregatedMetrics,
    reader::{MetricsData, ResourceMetricsData, ScopeMetricsData},
    InstrumentInfo, Temporality,
};

/// A collection of [`BatchScopeMetrics`] and the associated [Resource] that created them.
#[derive(Debug)]
pub struct ResourceMetrics<'a> {
    /// The entity that collected the metrics.
    pub resource: &'a Resource,
    /// The collection of metrics with unique [InstrumentationScope]s.
    pub scope_metrics: ScopeMetricsLendingIter<'a>,
}

/// Iterator over libraries instrumentation scopes ([`InstrumentationScope`]) together with metrics.
/// Doesn't implement standard [`Iterator`], because it returns borrowed items. AKA "LendingIterator".
pub struct ScopeMetricsLendingIter<'a> {
    iter: Iter<'a, ScopeMetricsData>,
}

/// A collection of metrics produced by a [`InstrumentationScope`] meter.
#[derive(Debug)]
pub struct ScopeMetrics<'a> {
    /// The [InstrumentationScope] that the meter was created with.
    pub scope: &'a InstrumentationScope,
    /// The list of aggregations created by the meter.
    pub metrics: MetricsLendingIter<'a>,
}

/// Iterator over aggregations created by the meter.
/// Doesn't implement standard [`Iterator`], because it returns borrowed items. AKA "LendingIterator".
pub struct MetricsLendingIter<'a> {
    iter: Iter<'a, MetricsData>,
}

/// A collection of one or more aggregated time series from an [Instrument].
///
/// [Instrument]: crate::metrics::Instrument
#[derive(Debug)]
pub struct Metric<'a> {
    /// The name of the instrument that created this data.
    pub instrument: &'a InstrumentInfo,
    /// The aggregated data from an instrument.
    pub data: &'a AggregatedMetrics,
}

impl<'a> ResourceMetrics<'a> {
    pub(crate) fn new(rm: &'a ResourceMetricsData) -> Self {
        Self {
            resource: &rm.resource,
            scope_metrics: ScopeMetricsLendingIter {
                iter: rm.scope_metrics.iter(),
            },
        }
    }
}

impl<'a> ScopeMetrics<'a> {
    fn new(sm: &'a ScopeMetricsData) -> Self {
        Self {
            scope: &sm.scope,
            metrics: MetricsLendingIter {
                iter: sm.metrics.iter(),
            },
        }
    }
}

impl Debug for ScopeMetricsLendingIter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BatchScopeMetrics").finish()
    }
}

impl ScopeMetricsLendingIter<'_> {
    /// Advances the iterator and returns the next value.
    pub fn next(&mut self) -> Option<ScopeMetrics<'_>> {
        self.iter.next().map(ScopeMetrics::new)
    }
}

impl MetricsLendingIter<'_> {
    /// Advances the iterator and returns the next value.
    pub fn next(&mut self) -> Option<Metric<'_>> {
        self.iter.next().map(|metric| Metric {
            instrument: &metric.instrument,
            data: &metric.data,
        })
    }
}

impl Debug for MetricsLendingIter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BatchMetrics").finish()
    }
}

/// Exporter handles the delivery of metric data to external receivers.
///
/// This is the final component in the metric push pipeline.
pub trait PushMetricExporter: Send + Sync + 'static {
    /// Export serializes and transmits metric data to a receiver.
    ///
    /// All retry logic must be contained in this function. The SDK does not
    /// implement any retry logic. All errors returned by this function are
    /// considered unrecoverable and will be logged.
    fn export(
        &self,
        metrics: ResourceMetrics<'_>,
    ) -> impl std::future::Future<Output = OTelSdkResult> + Send;

    /// Flushes any metric data held by an exporter.
    fn force_flush(&self) -> OTelSdkResult;

    /// Releases any held computational resources.
    ///
    /// After Shutdown is called, calls to Export will perform no operation and
    /// instead will return an error indicating the shutdown state.
    fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult;

    /// Shutdown with the default timeout of 5 seconds.
    fn shutdown(&self) -> OTelSdkResult {
        self.shutdown_with_timeout(Duration::from_secs(5))
    }

    /// Access the [Temporality] of the MetricExporter.
    fn temporality(&self) -> Temporality;
}
