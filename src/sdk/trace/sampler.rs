//! # Sampler
use crate::api;

/// Sampling options
#[derive(Clone, Debug)]
pub enum Sampler {
    /// Always sample the trace
    AlwaysOn,
    /// Never sample the trace
    AlwaysOff,
    /// Respects the parent span's sampling decision or delegates a delegate sampler for root spans.
    ParentOrElse(Box<Sampler>),
    /// Sample a given fraction of traces. Fractions >= 1 will always sample. If the parent span is
    /// sampled, then it's child spans will automatically be sampled. Fractions < 0 are treated as
    /// zero, but spans may still be sampled if their parent is.
    Probability(f64),
}

impl api::Sampler for Sampler {
    fn should_sample(
        &self,
        parent_context: Option<&api::SpanContext>,
        trace_id: api::TraceId,
        _name: &str,
        _span_kind: &api::SpanKind,
        _attributes: &[api::KeyValue],
        _links: &[api::Link],
    ) -> api::SamplingResult {
        let decision = match self {
            // Always sample the trace
            Sampler::AlwaysOn => api::SamplingDecision::RecordAndSampled,
            // Never sample the trace
            Sampler::AlwaysOff => api::SamplingDecision::NotRecord,
            // The parent decision if sampled; otherwise the decision of delegate_sampler
            Sampler::ParentOrElse(delegate_sampler) => parent_context.map_or(
                delegate_sampler
                    .should_sample(
                        parent_context,
                        trace_id,
                        _name,
                        _span_kind,
                        _attributes,
                        _links,
                    )
                    .decision,
                |ctx| {
                    if ctx.is_sampled() {
                        api::SamplingDecision::RecordAndSampled
                    } else {
                        api::SamplingDecision::NotRecord
                    }
                },
            ),
            // Match parent or probabilistically sample the trace.
            Sampler::Probability(prob) => {
                if *prob >= 1.0 || parent_context.map(|ctx| ctx.is_sampled()).unwrap_or(false) {
                    api::SamplingDecision::RecordAndSampled
                } else {
                    let prob_upper_bound = (prob.max(0.0) * (1u64 << 63) as f64) as u64;
                    // The trace_id is already randomly generated, so we don't need a new one here
                    let rnd_from_trace_id = (trace_id.to_u128() as u64) >> 1;

                    if rnd_from_trace_id < prob_upper_bound {
                        api::SamplingDecision::RecordAndSampled
                    } else {
                        api::SamplingDecision::NotRecord
                    }
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

#[cfg(test)]
mod tests {
    use crate::api::{self, Sampler as _};
    use crate::sdk::Sampler;
    use rand::Rng;

    #[rustfmt::skip]
    fn sampler_data() -> Vec<(&'static str, Sampler, f64, bool, bool)> {
        vec![
            // Span w/o a parent
            ("never_sample", Sampler::AlwaysOff, 0.0, false, false),
            ("always_sample", Sampler::AlwaysOn, 1.0, false, false),
            ("probability_-1",  Sampler::Probability(-1.0), 0.0,  false, false),
            ("probability_.25", Sampler::Probability(0.25), 0.25, false, false),
            ("probability_.50", Sampler::Probability(0.50), 0.5,  false, false),
            ("probability_.75", Sampler::Probability(0.75), 0.75, false, false),
            ("probability_2.0", Sampler::Probability(2.0),  1.0,  false, false),

            // Spans w/o a parent delegate
            ("delegate_to_always_on", Sampler::ParentOrElse(Box::new(Sampler::AlwaysOn)), 1.0, false, false),
            ("delegate_to_always_off", Sampler::ParentOrElse(Box::new(Sampler::AlwaysOff)), 0.0, false, false),
            ("delegate_to_probability_-1", Sampler::ParentOrElse(Box::new(Sampler::Probability(-1.0))), 0.0, false, false),
            ("delegate_to_probability_.25", Sampler::ParentOrElse(Box::new(Sampler::Probability(0.25))), 0.25, false, false),
            ("delegate_to_probability_.50", Sampler::ParentOrElse(Box::new(Sampler::Probability(0.50))), 0.50, false, false),
            ("delegate_to_probability_.75", Sampler::ParentOrElse(Box::new(Sampler::Probability(0.75))), 0.75, false, false),
            ("delegate_to_probability_2.0", Sampler::ParentOrElse(Box::new(Sampler::Probability(2.0))), 1.0, false, false),

            // Spans with a parent that is *not* sampled act like spans w/o a parent
            ("unsampled_parent_with_probability_-1",  Sampler::Probability(-1.0), 0.0, true, false),
            ("unsampled_parent_with_probability_.25", Sampler::Probability(0.25), 0.25, true, false),
            ("unsampled_parent_with_probability_.50", Sampler::Probability(0.50), 0.5, true, false),
            ("unsampled_parent_with_probability_.75", Sampler::Probability(0.75), 0.75, true, false),
            ("unsampled_parent_with_probability_2.0", Sampler::Probability(2.0),  1.0, true, false),
            ("unsampled_parent_or_else_with_always_on", Sampler::ParentOrElse(Box::new(Sampler::AlwaysOn)), 0.0, true, false),
            ("unsampled_parent_or_else_with_always_off", Sampler::ParentOrElse(Box::new(Sampler::AlwaysOff)), 0.0, true, false),
            ("unsampled_parent_or_else_with_probability", Sampler::ParentOrElse(Box::new(Sampler::Probability(0.25))), 0.0, true, false),

            // Spans with a parent that is sampled, will always sample, regardless of the probability
            ("sampled_parent_with_probability_-1",  Sampler::Probability(-1.0), 1.0, true, true),
            ("sampled_parent_with_probability_.25", Sampler::Probability(0.25), 1.0, true, true),
            ("sampled_parent_with_probability_2.0", Sampler::Probability(2.0),  1.0, true, true),

            // Spans with a parent that is sampled, will always sample, regardless of the delegate sampler
            ("sampled_parent_or_else_with_always_on", Sampler::ParentOrElse(Box::new(Sampler::AlwaysOn)), 1.0, true, true),
            ("sampled_parent_or_else_with_always_off", Sampler::ParentOrElse(Box::new(Sampler::AlwaysOff)), 1.0, true, true),
            ("sampled_parent_or_else_with_probability_.25", Sampler::ParentOrElse(Box::new(Sampler::Probability(0.25))), 1.0, true, true),

            // Spans with a sampled parent, but when using the NeverSample Sampler, aren't sampled
            ("sampled_parent_span_with_never_sample", Sampler::AlwaysOff, 0.0, true, true),
        ]
    }

    #[test]
    fn sampling() {
        let total = 10_000;
        let mut rng = rand::thread_rng();
        for (name, sampler, expectation, parent, sample_parent) in sampler_data() {
            let mut sampled = 0;
            for _ in 0..total {
                let parent_context = if parent {
                    let trace_flags = if sample_parent {
                        api::TRACE_FLAG_SAMPLED
                    } else {
                        0
                    };
                    Some(api::SpanContext::new(
                        api::TraceId::from_u128(1),
                        api::SpanId::from_u64(1),
                        trace_flags,
                        false,
                    ))
                } else {
                    None
                };
                let trace_id = api::TraceId::from_u128(rng.gen());
                if sampler
                    .should_sample(
                        parent_context.as_ref(),
                        trace_id,
                        name,
                        &api::SpanKind::Internal,
                        &[],
                        &[],
                    )
                    .decision
                    == api::SamplingDecision::RecordAndSampled
                {
                    sampled += 1;
                }
            }
            let mut tolerance = 0.0;
            let got = sampled as f64 / total as f64;

            if expectation > 0.0 && expectation < 1.0 {
                // See https://en.wikipedia.org/wiki/Binomial_proportion_confidence_interval
                let z = 4.75342; // This should succeed 99.9999% of the time
                tolerance = z * (got * (1.0 - got) / total as f64).sqrt();
            }

            let diff = (got - expectation).abs();
            assert!(
                diff <= tolerance,
                "{} got {:?} (diff: {}), expected {} (w/tolerance: {})",
                name,
                got,
                diff,
                expectation,
                tolerance
            );
        }
    }
}
