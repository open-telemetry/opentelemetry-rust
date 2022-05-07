use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use opentelemetry_api::{Context, InstrumentationLibrary, KeyValue};
use opentelemetry_api::trace::{Link, SamplingDecision, SamplingResult, SpanKind, TraceId, TraceState};
use crate::trace::sampler::jaeger_remote::remote::SamplingStrategyResponse;
use crate::trace::sampler::jaeger_remote::sampler::InitialSamplingConfig;
use crate::trace::sampler::sample_based_on_probability;
use crate::trace::ShouldSample;

use super::rate_limit::LeakyBucket;
use super::per_operation::PerOperationStrategies;

// todo: remove the mutex as probabilistic doesn't require mutable ref
// sampling strategy that sent by remote agents or collectors.
enum Inner {
    // probability to sample between [0.0, 1.0]
    Probabilistic(f64),
    //maxTracesPerSecond
    RateLimiting(LeakyBucket),
    PerOperation(PerOperationStrategies),
}

pub struct SamplingStrategy {
    inner: Mutex<Option<Inner>>,
    default_sampler: Box<dyn ShouldSample + 'static>,
    config: InitialSamplingConfig,
}

impl SamplingStrategy {
    pub(crate) fn new<S>(config: InitialSamplingConfig, default_sampler: S) -> Self
        where S: ShouldSample + 'static {
        SamplingStrategy {
            default_sampler: Box::new(default_sampler),
            inner: Mutex::new(None),
            config,
        }
    }

    pub(crate) fn update(&self, remote_strategy_resp: SamplingStrategyResponse) {
        self.inner.lock().map(|mut old_strategy_opt| {
            // update sample strategy
            // the response should be an union type where
            // - operation_sampling
            // - rate_limiting_sampling
            // - probabilistic_sampling
            // are mutually exclusive.
            *old_strategy_opt = old_strategy_opt
                .take()
                .and_then(|mut old_strategy| {
                    match (&remote_strategy_resp.operation_sampling, &remote_strategy_resp.rate_limiting_sampling, &remote_strategy_resp.probabilistic_sampling, &mut old_strategy) {
                        (Some(op_sampling), None, None, Inner::PerOperation(_)) => {
                            // ops sampling
                            todo!()
                        }
                        (None, Some(rate_limiting), None, Inner::RateLimiting(leaky_bucket)) => {
                            leaky_bucket.update(rate_limiting.max_traces_per_second as f64); // in the future the remote response may support f64
                        }
                        (None, None, Some(probabilistic), Inner::Probabilistic(prob)) => {
                            *prob = probabilistic.sampling_rate;
                        }
                        _ => {
                            // invalid cases, do nothing
                        }
                    }
                    Some(old_strategy)
                }).or({
                // no old strategy, create a new one
                match (remote_strategy_resp.operation_sampling, remote_strategy_resp.rate_limiting_sampling, remote_strategy_resp.probabilistic_sampling) {
                    (Some(op_sampling), None, None) => {
                        // ops sampling
                        todo!()
                    }
                    (None, Some(rate_limiting), None) => {
                        Some(Inner::RateLimiting(LeakyBucket::new(self.config.bucket_size, rate_limiting.max_traces_per_second as f64)))
                    }
                    (None, None, Some(probabilistic)) => {
                        Some(Inner::Probabilistic(probabilistic.sampling_rate))
                    }
                    _ => None
                }
            });
        });
    }

    pub fn should_sample(&self, parent_context: Option<&Context>, trace_id: TraceId, name: &str, span_kind: &SpanKind, attributes: &[KeyValue], links: &[Link], instrumentation_library: &InstrumentationLibrary) -> SamplingResult {
        let default_sampler = &self.default_sampler;
        self.inner.lock().and_then(|mut inner_opt| {
            match inner_opt.as_mut() {
                Some(inner) => {
                    let decision = match inner {
                        Inner::RateLimiting(leaky_bucket) => {
                            if leaky_bucket.should_sample() {
                                SamplingDecision::RecordAndSample
                            } else {
                                SamplingDecision::Drop
                            }
                        }
                        Inner::Probabilistic(prob) => {
                            sample_based_on_probability(prob, trace_id)
                        }
                        Inner::PerOperation(per_operation_strategies) => {
                            todo!()
                        }
                    };
                    Ok(SamplingResult {
                        decision,
                        attributes: Vec::new(),
                        trace_state: TraceState::default(), // todo: propagate state from parent
                    })
                }
                None => {
                    Ok(default_sampler.should_sample(parent_context, trace_id, name, span_kind, attributes, links, instrumentation_library))
                }
            }
        })
            .unwrap_or_else(|_| {
                default_sampler.should_sample(parent_context, trace_id, name, span_kind, attributes, links, instrumentation_library)
            })
    }
}