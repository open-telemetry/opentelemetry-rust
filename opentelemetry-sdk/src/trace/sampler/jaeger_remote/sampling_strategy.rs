use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use opentelemetry_api::{Context, InstrumentationLibrary, KeyValue};
use opentelemetry_api::trace::{Link, SamplingDecision, SamplingResult, SpanKind, TraceContextExt, TraceId, TraceState};
use crate::trace::sampler::jaeger_remote::remote::{OperationSamplingStrategy, PerOperationSamplingStrategies, SamplingStrategyResponse};
use crate::trace::sampler::sample_based_on_probability;
use crate::trace::ShouldSample;

use super::rate_limit::LeakyBucket;

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
    // initial configuration for leaky bucket
    leaky_bucket_size: f64,
}

impl SamplingStrategy {
    pub(crate) fn new<S>(default_sampler: S, leaky_bucket_size: f64) -> Self
        where S: ShouldSample + 'static {
        SamplingStrategy {
            default_sampler: Box::new(default_sampler),
            inner: Mutex::new(None),
            leaky_bucket_size,
        }
    }

    pub(crate) fn update(&self, remote_strategy_resp: SamplingStrategyResponse) {
        self.inner.lock().map(|mut old_strategy_opt| {
            *old_strategy_opt = match old_strategy_opt.take() {
                Some(mut old_strategy) => {
                    // update sample strategy
                    // the response should be an union type where
                    // - operation_sampling
                    // - rate_limiting_sampling
                    // - probabilistic_sampling
                    // are mutually exclusive.
                    match (remote_strategy_resp.operation_sampling, remote_strategy_resp.rate_limiting_sampling, remote_strategy_resp.probabilistic_sampling, &mut old_strategy) {
                        (Some(remote_ops_sampling), None, None, Inner::PerOperation(per_ops_sampling)) => {
                            per_ops_sampling.update(remote_ops_sampling);
                        }
                        (None, Some(rate_limiting), None, Inner::RateLimiting(leaky_bucket)) => {
                            leaky_bucket.update(rate_limiting.max_traces_per_second as f64); // in the future the remote response may support f64
                        }
                        (None, None, Some(remote_prob), Inner::Probabilistic(prob)) => {
                            *prob = remote_prob.sampling_rate;
                        }
                        _ => {
                            // invalid cases, do nothing
                        }
                    }
                    Some(old_strategy)
                }
                None => {
                    // initiate sample strategy
                    match (remote_strategy_resp.operation_sampling, remote_strategy_resp.rate_limiting_sampling, remote_strategy_resp.probabilistic_sampling) {
                        (Some(op_sampling), None, None) => {
                            // ops sampling
                            let mut per_ops_sampling = PerOperationStrategies::default();
                            per_ops_sampling.update(op_sampling);
                            Some(Inner::PerOperation(per_ops_sampling))
                        }
                        (None, Some(rate_limiting), None) => {
                            Some(Inner::RateLimiting(LeakyBucket::new(self.leaky_bucket_size, rate_limiting.max_traces_per_second as f64)))
                        }
                        (None, None, Some(probabilistic)) => {
                            Some(Inner::Probabilistic(probabilistic.sampling_rate))
                        }
                        _ => None
                    }
                }
            };
        }).unwrap_or_else(|e| {
            // add warning log
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
                            sample_based_on_probability(&per_operation_strategies.get_probability(name), trace_id)
                        }
                    };
                    Ok(SamplingResult {
                        decision,
                        attributes: Vec::new(),
                        trace_state: match parent_context {
                            Some(ctx) => ctx.span().span_context().trace_state().clone(),
                            None => TraceState::default(),
                        },
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

#[derive(Default)]
pub(crate) struct PerOperationStrategies {
    default_prob: f64,
    default_lower_bound: f64,
    operation_prob: HashMap<String, f64>,
    // todo: switch to Dashmap if the outside mutex is removed
    default_upper_bound: f64,
}

impl PerOperationStrategies {
    pub(crate) fn update(&mut self, remote_strategies: PerOperationSamplingStrategies) {
        self.default_prob = remote_strategies.default_sampling_probability as f64;
        self.default_lower_bound = remote_strategies.default_lower_bound_traces_per_second as f64;
        self.default_upper_bound = remote_strategies.default_upper_bound_traces_per_second as f64;

        self.operation_prob = remote_strategies.per_operation_strategies.into_iter().map(|op_strategy| {
            (op_strategy.operation, op_strategy.probabilistic_sampling.sampling_rate)
        }).collect();
    }

    pub(crate) fn get_probability(&self, operation: &str) -> f64 {
        let prob = *(self.operation_prob.get(operation).unwrap_or(&self.default_prob));
        prob.clamp(self.default_lower_bound, self.default_upper_bound)
    }
}