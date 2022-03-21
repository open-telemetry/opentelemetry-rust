use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use dashmap::Map;
use opentelemetry_api::{Context, InstrumentationLibrary, KeyValue};
use opentelemetry_api::trace::{Link, SamplingDecision, SamplingResult, SpanKind, TraceId};
use crate::trace::Sampler;
use crate::trace::sampler::sample_based_on_probability;

use super::rate_limit::LeakyBucket;
use super::per_operation::PerOperationStrategies;

// sampling strategy that sent by remote agents or collectors.
enum Inner {
    // not yet available, should use the default sampler
    None,
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
                _ => {}
            };
            Ok(Some(decision))
        })
            .unwrap_or(None)
    }
}