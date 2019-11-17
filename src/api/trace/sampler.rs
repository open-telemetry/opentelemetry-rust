//! # OpenTelemetry Sampler Interface

/// Sampling options
#[derive(Debug)]
pub enum Sampler {
    /// Always sample the trace
    Always,
    /// Never sample the trace
    Never,
    /// Sample a given fraction of traces. Fractions >= 1 will always sample.
    /// If the parent span is sampled, then it's child spans will automatically
    /// be sampled. Fractions <0 are treated as zero, but spans may still be
    /// sampled if their parent is.
    Probability(f64),
}
