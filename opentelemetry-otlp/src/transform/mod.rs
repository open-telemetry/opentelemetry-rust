#[cfg(feature = "metrics")]
mod metrics;
mod resource;

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
    opentelemetry_proto::tonic::metrics::v1::Metric,
);
