//! Jaeger remote sampler
//! Note that you don't necessary need a jaeger backend to run it. Opentelemetry collector also supports
//! Jaeger remote sampling protocol.
//!
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::time::Duration;
use opentelemetry_api::{Context, global, InstrumentationLibrary, KeyValue};
use opentelemetry_api::trace::{Link, SamplingResult, SpanKind, TraceContextExt, TraceError, TraceId, TraceState};
use crate::trace::ShouldSample;
use opentelemetry_http::HttpClient;
use sampling_strategy::SamplingStrategy;

mod sampling_strategy;
mod rate_limit;
mod per_operation;
mod remote;

// todo: need test the sampling APIs(integration test?)
// todo: for probabilistic sampling, we should use RwLocks(Not available in futures), or AtomicNumber?

/// Sampler that fetches the sampling configuration from remotes.
///
/// Note that the backend doesn't need to be Jaeger so long as it supports jaeger remote sampling
/// protocol.
pub struct JaegerRemoteSampler {
    default_sampler: Box<dyn ShouldSample>,
    current_strategy: SamplingStrategy,
    update_timeout: Duration,
    // contains endpoint and service name
    endpoint: http::Uri,
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
    fn setup<C, R>(runtime: R, current_strategy: SamplingStrategy, update_timeout: Duration, client: C, shutdown: futures_channel::mpsc::Receiver<()>, endpoint: http::Uri)
        where R: crate::runtime::Runtime,
              C: HttpClient {
        let interval = runtime.interval(update_timeout);
        runtime.spawn(Box::pin(async move {
            // poll next available configuration or shutdown
            // send request
            match Self::request_new_strategy(client, endpoint).await {
                Ok(remote_strategy_resp) => {
                    // update sample strategy
                    // the response should be an union type where
                    // - operation_sampling
                    // - rate_limiting_sampling
                    // - probabilistic_sampling
                    // are mutually exclusive.
                    let strategy = match (remote_strategy_resp.operation_sampling, remote_strategy_resp.rate_limiting_sampling, remote_strategy_resp.probabilistic_sampling) {
                        (Some(op_sampling), None, None) => {
                            // ops sampling
                        }
                        (None, Some(rate_limiting), None) => {
                            // rate limiting
                        }
                        (None, None, Some(probabilistic)) => {
                            // probabilistic
                        }
                        _ => {
                            // invalid cases
                        }
                    }
                }
                Err(()) => {

                }
            }
        }))
    }

    async fn request_new_strategy<C>(client: C, endpoint: http::Uri) -> Result<remote::SamplingStrategyResponse, ()>
        where C: HttpClient {
        let request = http::Request::get(endpoint)
            .header("Content-Type", "application/json")
            .body(vec![])
            .unwrap();

        let resp = client.send(request).await.map_err(|err| {
            ()
        })?;

        // process failures
        if resp.status() != http::StatusCode::OK {
            return Err(());
        }

        // deserialize the response
        Ok(serde_json::from_slice(&resp.body()[..]).map_err(|err| {
            ()
        })?)
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

fn warn<E: Error>(_err: E) -> () {
    // todo: print warning information
    ()
}

#[cfg(test)]
mod tests {}