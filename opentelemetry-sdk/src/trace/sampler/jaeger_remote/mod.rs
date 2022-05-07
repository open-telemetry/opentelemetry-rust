//! Jaeger remote sampler
//! Note that you don't necessary need a jaeger backend to run it. Opentelemetry collector also supports
//! Jaeger remote sampling protocol.
//!
mod sampling_strategy;
mod rate_limit;
mod per_operation;
mod remote;
mod sampler;

// todo: for probabilistic sampling, we should use RwLocks(Not available in futures), or AtomicNumber?


#[cfg(test)]
mod tests {}