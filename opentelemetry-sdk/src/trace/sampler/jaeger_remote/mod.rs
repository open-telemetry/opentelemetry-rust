//! Jaeger remote sampler
//! Note that you don't necessary need a jaeger backend to run it. Opentelemetry collector also supports
//! Jaeger remote sampling protocol.
//!
use std::fmt::{Debug, Formatter};
use std::time::Duration;
use opentelemetry_api::{Context, InstrumentationLibrary, KeyValue};
use opentelemetry_api::trace::{Link, SamplingResult, SpanKind, TraceContextExt, TraceId, TraceState};
use crate::trace::ShouldSample;
use opentelemetry_http::HttpClient;
use sampling_strategy::SamplingStrategy;

mod sampling_strategy;
mod rate_limit;
mod per_operation;

// todo: need proto generated file.
// todo: need test the sampling APIs(integration test?)
// todo: for probabilistic sampling, we should use RwLocks(Not available in futures), or AtomicNumber?

/// Sampler that fetches the sampling configuration from remotes.
pub struct JaegerRemoteSampler {
    default_sampler: Box<dyn ShouldSample>,
    current_strategy: SamplingStrategy,
    update_timeout: Duration,
    shutdown: futures_channel::mpsc::Sender<()>,
}

impl Drop for JaegerRemoteSampler {
    fn drop(&mut self) {
        self.shutdown.try_send(()); // best effort to shutdown the updating thread/task
    }
}

impl Debug for JaegerRemoteSampler {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl JaegerRemoteSampler
{
    // start a updating thread/task
    fn setup<C, R>(runtime: R, sample_strategy: SamplingStrategy, update_timeout: Duration, client: C, shutdown: futures_channel::mpsc::Receiver<()>)
        where R: crate::runtime::Runtime,
              C: HttpClient {
        let interval = runtime.interval(update_timeout);
        runtime.spawn(Box::pin(async move {
            // poll next available configuration or shutdown
            // send request
            // deserialize it into struct
            // update sample strategy
        }))
    }
}

impl ShouldSample for JaegerRemoteSampler {
    fn should_sample(&self, parent_context: Option<&Context>, trace_id: TraceId, name: &str, span_kind: &SpanKind, attributes: &[KeyValue], links: &[Link], instrumentation_library: &InstrumentationLibrary) -> SamplingResult {
        self.current_strategy.should_sample(trace_id, name)
            .map(|decision| SamplingResult {
                decision,
                attributes: Vec::new(),
                // all sampler in SDK will not modify trace state.
                trace_state: match parent_context {
                    Some(ctx) => ctx.span().span_context().trace_state().clone(),
                    None => TraceState::default(),
                },
            })
            // if the remote sampler cannot generate a result. Fall back to default sample
            // it can happen when the lock is poisoned or when the sampler is just initialized
            .unwrap_or_else(|| {
                self.default_sampler.should_sample(parent_context, trace_id, name, span_kind, attributes, links, instrumentation_library)
            })
    }
}

#[cfg(test)]
mod tests {}