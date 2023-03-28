#[cfg(feature = "metrics")]
mod metrics;

#[cfg(all(feature = "grpc-tonic", feature = "metrics"))]
pub(crate) use metrics::tonic::sink;
