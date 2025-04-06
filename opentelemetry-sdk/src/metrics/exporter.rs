//! Interfaces for exporting metrics

use opentelemetry::InstrumentationScope;

use crate::{error::OTelSdkResult, Resource};
use std::{fmt::Debug, slice::Iter, time::Duration};

use super::{
    data::{Metric, ResourceMetrics, ScopeMetrics},
    Temporality,
};

/// A collection of [`BatchScopeMetrics`] and the associated [Resource] that created them.
#[derive(Debug)]
pub struct ResourceMetricsRef<'a> {
    /// The entity that collected the metrics.
    pub resource: &'a Resource,
    /// The collection of metrics with unique [InstrumentationScope]s.
    pub scope_metrics: BatchScopeMetrics<'a>,
}

/// Iterator over libraries instrumentation scopes ([`InstrumentationScope`]) together with metrics.
pub struct BatchScopeMetrics<'a> {
    iter: Iter<'a, ScopeMetrics>,
}

/// A collection of metrics produced by a [`InstrumentationScope`] meter.
#[derive(Debug)]
pub struct ScopeMetricsRef<'a> {
    /// The [InstrumentationScope] that the meter was created with.
    pub scope: &'a InstrumentationScope,
    /// The list of aggregations created by the meter.
    pub metrics: BatchMetrics<'a>,
}

/// Iterator over aggregations created by the meter.
pub struct BatchMetrics<'a> {
    iter: Iter<'a, Metric>,
}

impl<'a> ResourceMetricsRef<'a> {
    pub(crate) fn new(rm: &'a ResourceMetrics) -> Self {
        Self {
            resource: &rm.resource,
            scope_metrics: BatchScopeMetrics {
                iter: rm.scope_metrics.iter(),
            },
        }
    }
}

impl<'a> ScopeMetricsRef<'a> {
    fn new(sm: &'a ScopeMetrics) -> Self {
        Self {
            scope: &sm.scope,
            metrics: BatchMetrics {
                iter: sm.metrics.iter(),
            },
        }
    }
}

impl Debug for BatchScopeMetrics<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BatchScopeMetrics").finish()
    }
}

impl<'a> Iterator for BatchScopeMetrics<'a> {
    type Item = ScopeMetricsRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(ScopeMetricsRef::new)
    }
}

impl<'a> Iterator for BatchMetrics<'a> {
    type Item = &'a Metric;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl Debug for BatchMetrics<'_> {
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
        metrics: ResourceMetricsRef<'_>,
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
