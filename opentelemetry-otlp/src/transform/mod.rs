mod common;
#[cfg(feature = "metrics")]
mod metrics;
mod resource;
mod traces;

#[cfg(all(feature = "tonic", feature = "metrics"))]
pub(crate) use metrics::tonic::record_to_metric;
#[cfg(all(feature = "tonic", feature = "metrics"))]
pub(crate) use metrics::tonic::sink;
#[cfg(all(feature = "tonic", feature = "metrics"))]
pub(crate) use resource::ResourceWrapper;

#[cfg(all(feature = "tonic", feature = "metrics"))]
use opentelemetry::sdk::InstrumentationLibrary;

// Metrics in OTEL proto format checked from checkpoint with information of resource and instrumentation
// library.
#[cfg(all(feature = "tonic", feature = "metrics"))]
pub(crate) type CheckpointedMetrics = (
    ResourceWrapper,
    InstrumentationLibrary,
    crate::proto::metrics::v1::Metric,
);
