use crate::trace::sampler::jaeger_remote::remote::SamplingStrategyResponse;
use crate::trace::sampler::jaeger_remote::sampling_strategy::SamplingStrategy;
use crate::trace::{Sampler, ShouldSample, TraceRuntime};
use futures_util::{stream, StreamExt as _};
use http::Uri;
use opentelemetry_api::trace::{Link, SamplingResult, SpanKind, TraceError, TraceId, TraceState};
use opentelemetry_api::{Context, InstrumentationLibrary, KeyValue};
use opentelemetry_http::HttpClient;
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

const DEFAULT_REMOTE_SAMPLER_ENDPOINT: &str = "http://localhost:5778/sampling";

#[derive(Debug)]
pub struct JaegerRemoteSamplerBuilder<C, S, R>
where
    R: TraceRuntime,
    C: HttpClient + 'static,
    S: ShouldSample + 'static,
{
    pub(crate) update_interval: Duration,
    pub(crate) client: C,
    pub(crate) endpoint: String,
    pub(crate) default_sampler: S,
    pub(crate) leaky_bucket_size: f64,
    pub(crate) runtime: R,
    pub(crate) service_name: String,
}

impl<C, S, R> JaegerRemoteSamplerBuilder<C, S, R>
where
    C: HttpClient + 'static,
    S: ShouldSample + 'static,
    R: TraceRuntime,
{
    pub(crate) fn new<Svc>(
        runtime: R,
        http_client: C,
        default_sampler: S,
        service_name: Svc,
    ) -> Self
    where
        Svc: Into<String>,
    {
        JaegerRemoteSamplerBuilder {
            runtime,
            update_interval: Duration::from_secs(60 * 5),
            client: http_client,
            endpoint: DEFAULT_REMOTE_SAMPLER_ENDPOINT.to_string(),
            default_sampler,
            leaky_bucket_size: 100.0,
            service_name: service_name.into(),
        }
    }

    pub fn with_update_interval(self, interval: Duration) -> Self {
        Self {
            update_interval: interval,
            ..self
        }
    }

    pub fn with_endpoint<STR: Into<String>>(self, endpoint: STR) -> Self {
        Self {
            endpoint: endpoint.into(),
            ..self
        }
    }

    pub fn with_leaky_bucket_size(self, size: f64) -> Self {
        Self {
            leaky_bucket_size: size,
            ..self
        }
    }

    pub fn build(self) -> Result<Sampler, TraceError> {
        let endpoint = Self::get_endpoint(&self.endpoint, &self.service_name)
            .map_err(|err_str| TraceError::Other(err_str.into()))?;

        Ok(Sampler::JaegerRemote(JaegerRemoteSampler::new(
            self.runtime,
            self.update_interval,
            self.client,
            endpoint,
            self.default_sampler,
            self.leaky_bucket_size,
        )))
    }

    fn get_endpoint(endpoint: &str, service_name: &str) -> Result<Uri, String> {
        if endpoint.is_empty() || service_name.is_empty() {
            return Err("endpoint and service name cannot be empty".to_string());
        }
        let mut endpoint = url::Url::parse(endpoint)
            .unwrap_or(url::Url::parse(DEFAULT_REMOTE_SAMPLER_ENDPOINT).unwrap());
        endpoint
            .query_pairs_mut()
            .append_pair("service", service_name);

        Uri::from_str(endpoint.as_str()).map_err(|err| "invalid service name".to_string())
    }
}

/// Sampler that fetches the sampling configuration from remotes.
///
/// Note that the backend doesn't need to be Jaeger so long as it supports jaeger remote sampling
/// protocol.
#[derive(Clone)]
pub struct JaegerRemoteSampler {
    strategy: Arc<SamplingStrategy>,
    shutdown: futures_channel::mpsc::Sender<()>,
}

impl Drop for JaegerRemoteSampler {
    fn drop(&mut self) {
        println!("shut down");
        self.shutdown.try_send(()); // best effort to shutdown the updating thread/task
    }
}

impl Debug for JaegerRemoteSampler {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl JaegerRemoteSampler {
    fn new<C, R, S>(
        runtime: R,
        update_timeout: Duration,
        client: C,
        endpoint: Uri,
        default_sampler: S,
        leaky_bucket_size: f64,
    ) -> Self
    where
        R: TraceRuntime,
        C: HttpClient + 'static,
        S: ShouldSample + 'static,
    {
        let strategy = Arc::new(SamplingStrategy::new(default_sampler, leaky_bucket_size));
        let (shutdown_tx, shutdown_rx) = futures_channel::mpsc::channel(1);
        let sampler = JaegerRemoteSampler {
            strategy,
            shutdown: shutdown_tx,
        };
        Self::run_update_task(
            runtime,
            sampler.strategy.clone(),
            update_timeout,
            client,
            shutdown_rx,
            endpoint,
        );
        sampler
    }

    // start a updating thread/task
    fn run_update_task<C, R>(
        runtime: R,
        strategy: Arc<SamplingStrategy>,
        update_timeout: Duration,
        client: C,
        shutdown: futures_channel::mpsc::Receiver<()>,
        endpoint: http::Uri,
    ) where
        R: TraceRuntime,
        C: HttpClient + 'static,
    {
        // todo: review if we need 'static here
        let interval = runtime.interval(update_timeout);
        runtime.spawn(Box::pin(async move {
            let mut update = Box::pin(stream::select(
                shutdown.map(|_| false),
                interval.map(|_| true),
            ));
            while let Some(should_update) = update.next().await {
                if should_update {
                    // poll next available configuration or shutdown
                    // send request
                    match Self::request_new_strategy(&client, endpoint.clone()).await {
                        Ok(remote_strategy_resp) => strategy.update(remote_strategy_resp),
                        Err(()) => {}
                    };
                } else {
                    // shutdown
                    break;
                }
            }
        }));
    }

    async fn request_new_strategy<C>(
        client: &C,
        endpoint: http::Uri,
    ) -> Result<SamplingStrategyResponse, ()>
    where
        C: HttpClient,
    {
        let request = http::Request::get(endpoint)
            .header("Content-Type", "application/json")
            .body(Vec::new())
            .unwrap();

        let resp = client.send(request).await.map_err(|err| todo!())?;

        // process failures
        if resp.status() != http::StatusCode::OK {
            return Err(());
        }

        // deserialize the response
        Ok(serde_json::from_slice(&resp.body()[..]).map_err(|err| todo!())?)
    }
}

impl ShouldSample for JaegerRemoteSampler {
    fn should_sample(
        &self,
        parent_context: Option<&Context>,
        trace_id: TraceId,
        name: &str,
        span_kind: &SpanKind,
        attributes: &[KeyValue],
        links: &[Link],
        instrumentation_library: &InstrumentationLibrary,
    ) -> SamplingResult {
        self.strategy.should_sample(
            parent_context,
            trace_id,
            name,
            span_kind,
            attributes,
            links,
            instrumentation_library,
        )
    }
}

fn warn<E: Error>(_err: E) -> () {
    // todo: print warning information
    ()
}
