//! Interfaces for exporting metrics

use opentelemetry::InstrumentationScope;

use crate::{error::OTelSdkResult, Resource};
use std::{
    fmt::Debug,
    slice::Iter,
    time::{Duration, SystemTime},
};

use super::{
    data::{AggregatedMetrics, Sum},
    pipeline::InstrumentSync,
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
    iter: std::collections::hash_map::Iter<'a, InstrumentationScope, Vec<InstrumentSync>>,
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
    // for  optimization purposes
    aggr: AggregatedMetrics,
    iter: Iter<'a, InstrumentSync>,
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
    pub(crate) fn new(
        resource: &'a Resource,
        iter: std::collections::hash_map::Iter<'a, InstrumentationScope, Vec<InstrumentSync>>,
    ) -> Self {
        Self {
            resource,
            scope_metrics: ScopeMetricsLendingIter { iter },
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
    pub fn next_scope_metric(&mut self) -> Option<ScopeMetrics<'_>> {
        self.iter.next().map(|(scope, instruments)| ScopeMetrics {
            scope,
            metrics: MetricsLendingIter {
                // doesn't matter what we initialize this with,
                // it's purpose is to be reused between collections
                aggr: AggregatedMetrics::F64(super::data::MetricData::Sum(Sum {
                    is_monotonic: true,
                    data_points: Vec::new(),
                    start_time: SystemTime::now(),
                    time: SystemTime::now(),
                    temporality: Temporality::Cumulative,
                })),
                iter: instruments.iter(),
            },
        })
    }
}

impl MetricsLendingIter<'_> {
    /// Advances the iterator and returns the next value.    
    pub fn next_metric(&mut self) -> Option<Metric<'_>> {
        loop {
            let inst = self.iter.next()?;
            let (len, data) = inst.comp_agg.call(Some(&mut self.aggr));
            if len > 0 {
                if let Some(new_aggr) = data {
                    self.aggr = new_aggr;
                }
                return Some(Metric {
                    instrument: &inst.info,
                    data: &self.aggr,
                });
            }
        }
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
