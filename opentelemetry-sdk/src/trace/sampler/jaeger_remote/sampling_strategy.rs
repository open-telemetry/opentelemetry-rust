use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use opentelemetry_api::trace::{SamplingDecision, TraceId};
use crate::trace::sampler::jaeger_remote::remote::SamplingStrategyResponse;
use crate::trace::sampler::sample_based_on_probability;

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
    inner: Arc<Mutex<Inner>>,
}

impl SamplingStrategy {
    pub fn update(&self, remote_strategy_resp: SamplingStrategyResponse) {
        self.inner.lock().map(|mut old_strategy| {
            // update sample strategy
            // the response should be an union type where
            // - operation_sampling
            // - rate_limiting_sampling
            // - probabilistic_sampling
            // are mutually exclusive.
            match (remote_strategy_resp.operation_sampling, remote_strategy_resp.rate_limiting_sampling, remote_strategy_resp.probabilistic_sampling, old_strategy.deref_mut()) {
                (Some(op_sampling), None, None, Inner::PerOperation(_)) => {
                    // ops sampling
                }
                (None, Some(rate_limiting), None, Inner::RateLimiting(leaky_bucket)) => {
                    leaky_bucket.update(rate_limiting.max_traces_per_second as f64) // in the future the remote response may support f64
                }
                (None, None, Some(probabilistic), Inner::Probabilistic(prob)) => {
                    *prob = probabilistic.sampling_rate;
                }
                _ => {
                    // invalid cases, do nothing
                }
            }
        });
    }

    pub fn should_sample(&self, trace_id: TraceId, name: &str) -> Option<SamplingDecision> {
        self.inner.lock().and_then(|mut inner| {
            let decision = match inner.deref_mut() {
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
            Ok(Some(decision))
        })
            .unwrap_or(None)
    }
}