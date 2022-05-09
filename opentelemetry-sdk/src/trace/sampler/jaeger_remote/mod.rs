//! Jaeger remote sampler
//! Note that you don't necessary need a jaeger backend to run it. Opentelemetry collector also supports
//! Jaeger remote sampling protocol.
//!
mod sampling_strategy;
mod rate_limit;
mod remote;
mod sampler;

pub use sampler::{JaegerRemoteSampler, JaegerRemoteSamplerBuilder};

#[cfg(test)]
mod tests {}