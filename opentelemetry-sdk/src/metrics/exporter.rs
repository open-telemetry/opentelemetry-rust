//! Interfaces for exporting metrics
use std::time::Duration;

use async_trait::async_trait;

use opentelemetry::metrics::Result;

use crate::metrics::{data::ResourceMetrics, reader::TemporalitySelector};

/// Exporter handles the delivery of metric data to external receivers.
///
/// This is the final component in the metric push pipeline.
#[async_trait]
pub trait PushMetricsExporter: TemporalitySelector + Send + Sync + 'static {
    /// Export serializes and transmits metric data to a receiver.
    ///
    /// All retry logic must be contained in this function. The SDK does not
    /// implement any retry logic. All errors returned by this function are
    /// considered unrecoverable and will be reported to a configured error
    /// Handler.
    async fn export(&self, metrics: &mut ResourceMetrics, timeout: Duration) -> Result<()>;

    /// Flushes any metric data held by an exporter.
    async fn force_flush(&self) -> Result<()>;

    /// Releases any held computational resources.
    ///
    /// After Shutdown is called, calls to Export will perform no operation and
    /// instead will return an error indicating the shutdown state.
    fn shutdown(&self) -> Result<()>;
}
