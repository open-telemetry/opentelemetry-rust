use crate::trace::sampler::jaeger_remote::remote::SamplingStrategyResponse;
use crate::trace::sampler::jaeger_remote::sampling_strategy::Inner;
use crate::trace::{Sampler, ShouldSample, TraceRuntime};
use futures_util::{stream, StreamExt as _};
use http::Uri;
use opentelemetry_api::trace::{Link, OrderMap, SamplingResult, SpanKind, TraceError, TraceId};
use opentelemetry_api::{global, Context, InstrumentationLibrary, Key, Value};
use opentelemetry_http::HttpClient;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

const DEFAULT_REMOTE_SAMPLER_ENDPOINT: &str = "http://localhost:5778/sampling";

/// builder of JaegerRemoteSampler.
/// See [Sampler::jaeger_remote] for details.
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

    /// Change how often the SDK should fetch the sampling strategy from remote servers
    ///
    /// By default it fetches every 5 minutes.
    ///
    /// A shorter interval have a performance overhead and should be avoid.
    pub fn with_update_interval(self, interval: Duration) -> Self {
        Self {
            update_interval: interval,
            ..self
        }
    }

    /// The endpoint of remote servers.
    ///
    /// By default it's `http://localhost:5778/sampling`.
    ///
    /// If the service name is provided as part of the
    pub fn with_endpoint<Str: Into<String>>(self, endpoint: Str) -> Self {
        Self {
            endpoint: endpoint.into(),
            ..self
        }
    }

    /// The size of the leaky bucket.
    ///
    /// It's used when sampling strategy is rate limiting.
    pub fn with_leaky_bucket_size(self, size: f64) -> Self {
        Self {
            leaky_bucket_size: size,
            ..self
        }
    }

    /// Build a jaeger remote sampler.
    ///
    /// Return errors when the endpoint provided is invalid(e.g, service name is empty)
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
            .unwrap_or_else(|_| url::Url::parse(DEFAULT_REMOTE_SAMPLER_ENDPOINT).unwrap());
        endpoint
            .query_pairs_mut()
            .append_pair("service", service_name);

        Uri::from_str(endpoint.as_str()).map_err(|_err| "invalid service name".to_string())
    }
}

/// Sampler that fetches the sampling configuration from remotes.
///
/// Note that the backend doesn't need to be Jaeger so long as it supports jaeger remote sampling
/// protocol.
#[derive(Clone, Debug)]
pub struct JaegerRemoteSampler {
    inner: Arc<Inner>,
    default_sampler: Arc<dyn ShouldSample + 'static>,
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
        let (shutdown_tx, shutdown_rx) = futures_channel::mpsc::channel(1);
        let inner = Arc::new(Inner::new(leaky_bucket_size, shutdown_tx));
        let sampler = JaegerRemoteSampler {
            inner,
            default_sampler: Arc::new(default_sampler),
        };
        Self::run_update_task(
            runtime,
            sampler.inner.clone(),
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
        strategy: Arc<Inner>,
        update_timeout: Duration,
        client: C,
        shutdown: futures_channel::mpsc::Receiver<()>,
        endpoint: Uri,
    ) where
        R: TraceRuntime,
        C: HttpClient + 'static,
    {
        // todo: review if we need 'static here
        let interval = runtime.interval(update_timeout);
        runtime.spawn(Box::pin(async move {
            // either update or shutdown
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
                        Err(err_msg) => global::handle_error(TraceError::Other(err_msg.into())),
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
        endpoint: Uri,
    ) -> Result<SamplingStrategyResponse, String>
    where
        C: HttpClient,
    {
        let request = http::Request::get(endpoint)
            .header("Content-Type", "application/json")
            .body(Vec::new())
            .unwrap();

        let resp = client
            .send(request)
            .await
            .map_err(|err| format!("the request is failed to send {}", err))?;

        // process failures
        if resp.status() != http::StatusCode::OK {
            return Err(format!(
                "the http response code is not 200 but {}",
                resp.status()
            ));
        }

        // deserialize the response
        serde_json::from_slice(&resp.body()[..])
            .map_err(|err| format!("cannot deserialize the response, {}", err))
    }
}

impl ShouldSample for JaegerRemoteSampler {
    fn should_sample(
        &self,
        parent_context: Option<&Context>,
        trace_id: TraceId,
        name: &str,
        span_kind: &SpanKind,
        attributes: &OrderMap<Key, Value>,
        links: &[Link],
        instrumentation_library: &InstrumentationLibrary,
    ) -> SamplingResult {
        self.inner
            .should_sample(parent_context, trace_id, name)
            .unwrap_or_else(|| {
                self.default_sampler.should_sample(
                    parent_context,
                    trace_id,
                    name,
                    span_kind,
                    attributes,
                    links,
                    instrumentation_library,
                )
            })
    }
}
