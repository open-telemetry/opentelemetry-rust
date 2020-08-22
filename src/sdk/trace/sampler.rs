//! # OpenTelemetry ShouldSample Interface
//!
//! ## Sampling
//!
//! Sampling is a mechanism to control the noise and overhead introduced by
//! OpenTelemetry by reducing the number of samples of traces collected and
//! sent to the backend.
//!
//! Sampling may be implemented on different stages of a trace collection.
//! OpenTelemetry SDK defines a `ShouldSample` interface that can be used at
//! instrumentation points by libraries to check the sampling `SamplingDecision`
//! early and optimize the amount of telemetry that needs to be collected.
//!
//! All other sampling algorithms may be implemented on SDK layer in exporters,
//! or even out of process in Agent or Collector.
//!
//! The OpenTelemetry API has two properties responsible for the data collection:
//!
//! * `is_recording` method on a `Span`. If `true` the current `Span` records
//!   tracing events (attributes, events, status, etc.), otherwise all tracing
//!   events are dropped. Users can use this property to determine if expensive
//!   trace events can be avoided. `SpanProcessor`s will receive
//!   all spans with this flag set. However, `SpanExporter`s will
//!   not receive them unless the `Sampled` flag was set.
//! * `Sampled` flag in `trace_flags` on `SpanContext`. This flag is propagated
//!   via the `SpanContext` to child Spans. For more details see the [W3C
//!   specification](https://w3c.github.io/trace-context/). This flag indicates
//!   that the `Span` has been `sampled` and will be exported. `SpanProcessor`s
//!   and `SpanExporter`s will receive spans with the `Sampled` flag set for
//!   processing.
//!
//! The flag combination `Sampled == false` and `is_recording` == true` means
//! that the current `Span` does record information, but most likely the child
//! `Span` will not.
//!
//! The flag combination `Sampled == true` and `is_recording == false` could
//! cause gaps in the distributed trace, and because of this OpenTelemetry API
//! MUST NOT allow this combination.

use crate::api;

/// The `ShouldSample` interface allows implementations to provide samplers
/// which will return a sampling `SamplingResult` based on information that
/// is typically available just before the `Span` was created.
pub trait ShouldSample: Send + Sync + std::fmt::Debug {
    /// Returns the `SamplingDecision` for a `Span` to be created.
    #[allow(clippy::too_many_arguments)]
    fn should_sample(
        &self,
        parent_context: Option<&api::SpanContext>,
        trace_id: api::TraceId,
        name: &str,
        span_kind: &api::SpanKind,
        attributes: &[api::KeyValue],
        links: &[api::Link],
    ) -> SamplingResult;
}

/// The result of sampling logic for a given `Span`.
#[derive(Clone, Debug, PartialEq)]
pub struct SamplingResult {
    /// `SamplingDecision` reached by this result
    pub decision: SamplingDecision,
    /// Extra attributes added by this result
    pub attributes: Vec<api::KeyValue>,
}

/// Decision about whether or not to sample
#[derive(Clone, Debug, PartialEq)]
pub enum SamplingDecision {
    /// `is_recording() == false`, span will not be recorded and all events and
    /// attributes will be dropped.
    NotRecord,
    /// `is_recording() == true`, but `Sampled` flag MUST NOT be set.
    Record,
    /// `is_recording() == true` AND `Sampled` flag` MUST be set.
    RecordAndSampled,
}

/// Sampling options
#[derive(Clone, Debug)]
pub enum Sampler {
    /// Always sample the trace
    AlwaysOn,
    /// Never sample the trace
    AlwaysOff,
    /// Respects the parent span's sampling decision or delegates a delegate sampler for root spans.
    ParentBased(Box<Sampler>),
    /// Sample a given fraction of traces. Fractions >= 1 will always sample. If the parent span is
    /// sampled, then it's child spans will automatically be sampled. Fractions < 0 are treated as
    /// zero, but spans may still be sampled if their parent is.
    Probability(f64),
}

impl ShouldSample for Sampler {
    fn should_sample(
        &self,
        parent_context: Option<&api::SpanContext>,
        trace_id: api::TraceId,
        name: &str,
        span_kind: &api::SpanKind,
        attributes: &[api::KeyValue],
        links: &[api::Link],
    ) -> SamplingResult {
        let decision = match self {
            // Always sample the trace
            Sampler::AlwaysOn => SamplingDecision::RecordAndSampled,
            // Never sample the trace
            Sampler::AlwaysOff => SamplingDecision::NotRecord,
            // The parent decision if sampled; otherwise the decision of delegate_sampler
            Sampler::ParentBased(delegate_sampler) => parent_context.map_or(
                delegate_sampler
                    .should_sample(parent_context, trace_id, name, span_kind, attributes, links)
                    .decision,
                |ctx| {
                    if ctx.is_sampled() {
                        SamplingDecision::RecordAndSampled
                    } else {
                        SamplingDecision::NotRecord
                    }
                },
            ),
            // Probabilistically sample the trace.
            Sampler::Probability(prob) => {
                if *prob >= 1.0 {
                    SamplingDecision::RecordAndSampled
                } else {
                    let prob_upper_bound = (prob.max(0.0) * (1u64 << 63) as f64) as u64;
                    // The trace_id is already randomly generated, so we don't need a new one here
                    let rnd_from_trace_id = (trace_id.to_u128() as u64) >> 1;

                    if rnd_from_trace_id < prob_upper_bound {
                        SamplingDecision::RecordAndSampled
                    } else {
                        SamplingDecision::NotRecord
                    }
                }
            }
        };

        SamplingResult {
            decision,
            // No extra attributes ever set by the SDK samplers.
            attributes: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::api;
    use crate::sdk::{Sampler, SamplingDecision, ShouldSample};
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
            ("delegate_to_always_on", Sampler::ParentBased(Box::new(Sampler::AlwaysOn)), 1.0, false, false),
            ("delegate_to_always_off", Sampler::ParentBased(Box::new(Sampler::AlwaysOff)), 0.0, false, false),
            ("delegate_to_probability_-1", Sampler::ParentBased(Box::new(Sampler::Probability(-1.0))), 0.0, false, false),
            ("delegate_to_probability_.25", Sampler::ParentBased(Box::new(Sampler::Probability(0.25))), 0.25, false, false),
            ("delegate_to_probability_.50", Sampler::ParentBased(Box::new(Sampler::Probability(0.50))), 0.50, false, false),
            ("delegate_to_probability_.75", Sampler::ParentBased(Box::new(Sampler::Probability(0.75))), 0.75, false, false),
            ("delegate_to_probability_2.0", Sampler::ParentBased(Box::new(Sampler::Probability(2.0))), 1.0, false, false),

            // Spans with a parent that is *not* sampled act like spans w/o a parent
            ("unsampled_parent_with_probability_-1",  Sampler::Probability(-1.0), 0.0, true, false),
            ("unsampled_parent_with_probability_.25", Sampler::Probability(0.25), 0.25, true, false),
            ("unsampled_parent_with_probability_.50", Sampler::Probability(0.50), 0.5, true, false),
            ("unsampled_parent_with_probability_.75", Sampler::Probability(0.75), 0.75, true, false),
            ("unsampled_parent_with_probability_2.0", Sampler::Probability(2.0),  1.0, true, false),
            ("unsampled_parent_or_else_with_always_on", Sampler::ParentBased(Box::new(Sampler::AlwaysOn)), 0.0, true, false),
            ("unsampled_parent_or_else_with_always_off", Sampler::ParentBased(Box::new(Sampler::AlwaysOff)), 0.0, true, false),
            ("unsampled_parent_or_else_with_probability", Sampler::ParentBased(Box::new(Sampler::Probability(0.25))), 0.0, true, false),

            // A probability sampler with a parent that is sampled will ignore the parent
            ("sampled_parent_with_probability_-1",  Sampler::Probability(-1.0), 0.0, true, true),
            ("sampled_parent_with_probability_.25", Sampler::Probability(0.25), 0.25, true, true),
            ("sampled_parent_with_probability_2.0", Sampler::Probability(2.0),  1.0, true, true),

            // Spans with a parent that is sampled, will always sample, regardless of the delegate sampler
            ("sampled_parent_or_else_with_always_on", Sampler::ParentBased(Box::new(Sampler::AlwaysOn)), 1.0, true, true),
            ("sampled_parent_or_else_with_always_off", Sampler::ParentBased(Box::new(Sampler::AlwaysOff)), 1.0, true, true),
            ("sampled_parent_or_else_with_probability_.25", Sampler::ParentBased(Box::new(Sampler::Probability(0.25))), 1.0, true, true),

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
                    == SamplingDecision::RecordAndSampled
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
