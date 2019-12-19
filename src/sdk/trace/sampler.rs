//! # Sampler
//!
use crate::api;

/// Sampling options
#[derive(Clone, Debug)]
pub enum Sampler {
    /// Always sample the trace
    Always,
    /// Never sample the trace
    Never,
    /// Sample if the parent span is sampled
    Parent,
    /// Sample a given fraction of traces. Fractions >= 1 will always sample.
    /// If the parent span is sampled, then it's child spans will automatically
    /// be sampled. Fractions <0 are treated as zero, but spans may still be
    /// sampled if their parent is.
    Probability(f64),
}

impl api::Sampler for Sampler {
    fn should_sample(
        &self,
        parent_context: Option<&api::SpanContext>,
        _trace_id: u128,
        _span_id: u64,
        _name: &str,
        _span_kind: &api::SpanKind,
        _attributes: &[api::KeyValue],
        _links: &[api::Link],
    ) -> api::SamplingResult {
        let decision = match self {
            // Always sample the trace
            Sampler::Always => api::SamplingDecision::RecordAndSampled,
            // Never sample the trace
            Sampler::Never => api::SamplingDecision::NotRecord,
            // Sample if the parent span is sampled
            Sampler::Parent => {
                if parent_context.map(|ctx| ctx.is_sampled()).unwrap_or(false) {
                    api::SamplingDecision::RecordAndSampled
                } else {
                    api::SamplingDecision::NotRecord
                }
            }
            // Match parent or probabilistically sample the trace.
            Sampler::Probability(prob) => {
                if parent_context.map(|ctx| ctx.is_sampled()).unwrap_or(false) && *prob > 0.0 {
                    if *prob > 1.0 || *prob > rand::random() {
                        api::SamplingDecision::RecordAndSampled
                    } else {
                        api::SamplingDecision::NotRecord
                    }
                } else {
                    api::SamplingDecision::NotRecord
                }
            }
        };

        api::SamplingResult {
            decision,
            // No extra attributes ever set by the SDK samplers.
            attributes: Vec::new(),
        }
    }
}
