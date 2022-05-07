use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::time::Duration;
use futures_util::{stream, StreamExt as _};
use opentelemetry_api::{Context, InstrumentationLibrary, KeyValue};
use opentelemetry_api::trace::{Link, SamplingResult, SpanKind, TraceContextExt, TraceId, TraceState};
use crate::trace::{ShouldSample, TraceRuntime};
use opentelemetry_http::HttpClient;
use crate::trace::sampler::jaeger_remote::remote::SamplingStrategyResponse;
use crate::trace::sampler::jaeger_remote::sampling_strategy::SamplingStrategy;

#[non_exhaustive]
pub struct InitialSamplingConfig {
    // leaky bucket sampler initial config
    pub bucket_size: f64,
    pub span_per_sec: f64,
}

impl Default for InitialSamplingConfig {
    fn default() -> Self {
        InitialSamplingConfig {
            bucket_size: 20.0,
            span_per_sec: 1.0,
        }
    }
}

/// Sampler that fetches the sampling configuration from remotes.
///
/// Note that the backend doesn't need to be Jaeger so long as it supports jaeger remote sampling
/// protocol.
pub struct JaegerRemoteSampler {
    strategy: Arc<SamplingStrategy>,
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
    fn new<C, R, S>(runtime: R, update_timeout: Duration, client: C, endpoint: http::Uri, default_sampler: S, initial_setup: InitialSamplingConfig) -> Self
        where R: TraceRuntime,
              C: HttpClient + 'static,
              S: ShouldSample + 'static {
        let strategy = Arc::new(SamplingStrategy::new(initial_setup, default_sampler));
        let (shutdown_tx, shutdown_rx) = futures_channel::mpsc::channel(1);
        let sampler = JaegerRemoteSampler {
            strategy,
            update_timeout: update_timeout.clone(),
            endpoint: endpoint.clone(),
            shutdown: shutdown_tx,
        };
        Self::run_update_task(runtime, sampler.strategy.clone(), update_timeout, client, shutdown_rx, endpoint);
        sampler
    }

    // start a updating thread/task
    fn run_update_task<C, R>(runtime: R, strategy: Arc<SamplingStrategy>, update_timeout: Duration, client: C, shutdown: futures_channel::mpsc::Receiver<()>, endpoint: http::Uri)
        where R: TraceRuntime,
              C: HttpClient + 'static { // todo: review if we need 'static here
        let interval = runtime.interval(update_timeout);
        runtime.spawn(Box::pin(async move {
            let mut update = Box::pin(stream::select(shutdown.map(|_| false), interval.map(|_| true)));
            while let Some(should_update) = update.next().await {
                if should_update {
                    // poll next available configuration or shutdown
                    // send request
                    match Self::request_new_strategy(&client, endpoint.clone()).await {
                        Ok(remote_strategy_resp) => {
                            strategy.update(remote_strategy_resp)
                        }
                        Err(()) => {}
                    };
                } else {
                    // shutdown
                    break;
                }
            }
        }));
    }

    async fn request_new_strategy<C>(client: &C, endpoint: http::Uri) -> Result<SamplingStrategyResponse, ()>
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
        self.strategy.should_sample(parent_context, trace_id, name, span_kind, attributes, links, instrumentation_library)
    }
}

fn warn<E: Error>(_err: E) -> () {
    // todo: print warning information
    ()
}