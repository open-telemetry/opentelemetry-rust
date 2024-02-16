use std::borrow::BorrowMut;
use std::convert::TryFrom;
use std::env;
use std::sync::Arc;
#[cfg(feature = "collector_client")]
use std::time::Duration;

use http::Uri;

use opentelemetry::trace::TraceError;
#[cfg(feature = "collector_client")]
use opentelemetry_http::HttpClient;
use opentelemetry_sdk::trace::{BatchConfig, BatchSpanProcessor, Config, Tracer, TracerProvider};

#[cfg(feature = "collector_client")]
use crate::config::collector::http_client::CollectorHttpClient;
#[cfg(feature = "collector_client")]
use crate::exporter::collector::AsyncHttpClient;
#[cfg(feature = "wasm_collector_client")]
use crate::exporter::collector::WasmCollector;
use crate::exporter::config::{
    build_config_and_process, install_tracer_provider_and_get_tracer, HasRequiredConfig,
    TransformationConfig,
};
use crate::exporter::uploader::{AsyncUploader, Uploader};
use crate::{Exporter, JaegerTraceRuntime};

#[cfg(feature = "collector_client")]
mod http_client;

/// HTTP endpoint for Jaeger collector.
/// e.g. "http://localhost:14250"
const ENV_ENDPOINT: &str = "OTEL_EXPORTER_JAEGER_ENDPOINT";

const DEFAULT_ENDPOINT: &str = "http://localhost:14250/api/trace";

/// Timeout for Jaeger collector.
#[cfg(feature = "collector_client")]
const ENV_TIMEOUT: &str = "OTEL_EXPORTER_JAEGER_TIMEOUT";

/// Default of 10s
#[cfg(feature = "collector_client")]
const DEFAULT_COLLECTOR_TIMEOUT: Duration = Duration::from_secs(10);

/// Username to send as part of "Basic" authentication to the collector endpoint.
const ENV_USERNAME: &str = "OTEL_EXPORTER_JAEGER_USER";

/// Password to send as part of "Basic" authentication to the collector endpoint.
const ENV_PASSWORD: &str = "OTEL_EXPORTER_JAEGER_PASSWORD";

/// CollectorPipeline config and build a exporter targeting a jaeger collector using HTTP protocol.
///
/// ## Environment variables
///
/// - `OTEL_EXPORTER_JAEGER_ENDPOINT`: set the endpoint of the collector. Usually starts with `http://` or `https://`
///
/// - `OTEL_EXPORTER_JAEGER_TIMEOUT`: set the timeout of the http client timeout. It only applies to build in http clients.
///
/// - `OTEL_EXPORTER_JAEGER_USER`: set the username. Part of the authentication for the collector. It only applies to build in http clients.
///
/// - `OTEL_EXPORTER_JAEGER_PASSWORD`: set the password. Part of the authentication for the collector. It only applies to build in http clients.
///
/// ## Built-in http clients
/// To help user setup the exporter, `opentelemetry-jaeger` provides the following build in http client
/// implementation and relative configurations.
///
/// - [hyper], requires `hyper_collector_client` feature enabled, use [`with_hyper`][CollectorPipeline::with_hyper] function to setup.
/// - [isahc], requires `isahc_collector_client` feature enabled, use [`with_isahc`][CollectorPipeline::with_isahc] function to setup.
/// - [reqwest], requires `reqwest_collector_client` feature enabled,  use [`with_reqwest`][CollectorPipeline::with_reqwest] function to setup.
/// - [reqwest blocking client], requires `reqwest_blocking_collector_client` feature enabled, use [`with_reqwest_blocking`][CollectorPipeline::with_reqwest_blocking] function to setup.
///
/// Additionally you can enable https
///
/// Note that the functions to setup build in http clients override each other. That means if you have a pipeline with the following setup
///
/// ```no_run
/// # use opentelemetry::trace::TraceError;
/// # #[cfg(all(feature="reqwest_collector_client", feature="surf_collector_client"))]
/// let tracer = opentelemetry_jaeger::new_collector_pipeline()
///         .with_surf()
///         .with_reqwest()
///         .install_batch(opentelemetry_sdk::runtime::Tokio)
/// #       .unwrap();
/// ```
///
/// The pipeline will use [reqwest] http client.
///
/// [reqwest]: reqwest::Client
/// [reqwest blocking client]: reqwest::blocking::Client
#[derive(Debug)]
pub struct CollectorPipeline {
    transformation_config: TransformationConfig,
    trace_config: Option<Config>,
    batch_config: Option<BatchConfig>,

    #[cfg(feature = "collector_client")]
    collector_timeout: Duration,
    // only used by builtin http clients.
    collector_endpoint: Option<String>,
    collector_username: Option<String>,
    collector_password: Option<String>,

    client_config: ClientConfig,
}

impl Default for CollectorPipeline {
    fn default() -> Self {
        Self {
            #[cfg(feature = "collector_client")]
            collector_timeout: DEFAULT_COLLECTOR_TIMEOUT,
            collector_endpoint: None,
            collector_username: None,
            collector_password: None,
            client_config: ClientConfig::default(),
            transformation_config: Default::default(),
            trace_config: Default::default(),
            batch_config: Some(Default::default()),
        }
    }
}

// implement the seal trait
impl HasRequiredConfig for CollectorPipeline {
    fn set_transformation_config<T>(&mut self, f: T)
    where
        T: FnOnce(&mut TransformationConfig),
    {
        f(self.transformation_config.borrow_mut())
    }

    fn set_trace_config(&mut self, config: Config) {
        self.trace_config = Some(config)
    }

    fn set_batch_config(&mut self, config: BatchConfig) {
        self.batch_config = Some(config)
    }
}

#[derive(Debug)]
enum ClientConfig {
    #[cfg(feature = "collector_client")]
    Http { client_type: CollectorHttpClient },
    #[cfg(feature = "wasm_collector_client")]
    Wasm, // no config is available for wasm for now. But we can add in the future
}

#[allow(clippy::derivable_impls)]
impl Default for ClientConfig {
    fn default() -> Self {
        // as long as collector is enabled, we will in favor of it
        #[cfg(feature = "collector_client")]
        {
            ClientConfig::Http {
                client_type: CollectorHttpClient::None,
            }
        }
        // when collector_client is disabled and wasm_collector_client is enabled
        #[cfg(not(feature = "collector_client"))]
        ClientConfig::Wasm
    }
}

/// Start a new pipeline to configure a exporter that target a jaeger collector.
///
/// See details for each configurations at [`CollectorPipeline`].
///
/// [`CollectorPipeline`]: crate::config::collector::CollectorPipeline
#[cfg(feature = "collector_client")]
pub fn new_collector_pipeline() -> CollectorPipeline {
    CollectorPipeline::default()
}

/// Similar to [`new_collector_pipeline`] but the exporter is configured to run with wasm.
#[cfg(feature = "wasm_collector_client")]
#[allow(clippy::field_reassign_with_default)] // make sure when collector_cilent and wasm_collector_client are both set. We will create a wasm type client
pub fn new_wasm_collector_pipeline() -> CollectorPipeline {
    let mut pipeline = CollectorPipeline::default();
    pipeline.client_config = ClientConfig::Wasm;
    pipeline
}

impl CollectorPipeline {
    /// Set the http client timeout.
    ///
    /// This function only applies to build in http clients.
    ///
    /// Default to be 10s.
    #[cfg(feature = "collector_client")]
    pub fn with_timeout(self, collector_timeout: Duration) -> Self {
        Self {
            collector_timeout,
            ..self
        }
    }

    /// Set the collector endpoint.
    ///
    /// E.g. "http://localhost:14268/api/traces"
    pub fn with_endpoint<T: Into<String>>(self, collector_endpoint: T) -> Self {
        Self {
            collector_endpoint: Some(collector_endpoint.into()),
            ..self
        }
    }

    /// Set the username used in authentication to communicate with the collector.
    ///
    /// *Note* that if the password is not set by calling `with_password` or set `OTEL_EXPORTER_JAEGER_PASSWORD`
    /// environment variables. The username will be ignored.
    ///
    /// This function only applies to build in http clients.
    pub fn with_username<S: Into<String>>(self, collector_username: S) -> Self {
        Self {
            collector_username: Some(collector_username.into()),
            ..self
        }
    }

    /// Set the password used in authentication to communicate with the collector.
    ///
    /// *Note* that if the username is not set by calling `with_username` or set `OTEL_EXPORTER_JAEGER_USER`
    /// environment variables. The username will be ignored.
    ///
    /// This function only applies to build in http clients.
    pub fn with_password<S: Into<String>>(self, collector_password: S) -> Self {
        Self {
            collector_password: Some(collector_password.into()),
            ..self
        }
    }

    /// Get collector's username set in the builder. Default to be the value of
    /// `OTEL_EXPORTER_JAEGER_USER` environment variable.
    ///
    /// If users uses custom http client. This function can help retrieve the value of
    /// `OTEL_EXPORTER_JAEGER_USER` environment variable.
    pub fn collector_username(&self) -> Option<String> {
        self.collector_username.clone()
    }

    /// Get the collector's password set in the builder. Default to be the value of
    /// `OTEL_EXPORTER_JAEGER_PASSWORD` environment variable.
    ///
    /// If users uses custom http client. This function can help retrieve the value of
    /// `OTEL_EXPORTER_JAEGER_PASSWORD` environment variable.
    pub fn collector_password(&self) -> Option<String> {
        self.collector_password.clone()
    }

    /// Custom http client used to send spans.
    ///
    /// **Note** that all configuration other than the [`endpoint`][CollectorPipeline::with_endpoint] are not
    /// applicable to custom clients.
    #[cfg(feature = "collector_client")]
    pub fn with_http_client<T: HttpClient + 'static>(mut self, client: T) -> Self {
        self.client_config = match self.client_config {
            ClientConfig::Http { .. } => ClientConfig::Http {
                client_type: CollectorHttpClient::Custom(Box::new(client)),
            },
            // noop for wasm
            #[cfg(feature = "wasm_collector_client")]
            ClientConfig::Wasm => ClientConfig::Wasm,
        };
        self
    }

    /// Use isahc http client in the exporter.
    #[cfg(feature = "isahc_collector_client")]
    pub fn with_isahc(self) -> Self {
        Self {
            client_config: ClientConfig::Http {
                client_type: CollectorHttpClient::Isahc,
            },
            ..self
        }
    }

    /// Use reqwest http client in the exporter.
    #[cfg(feature = "reqwest_collector_client")]
    pub fn with_reqwest(self) -> Self {
        Self {
            client_config: ClientConfig::Http {
                client_type: CollectorHttpClient::Reqwest,
            },
            ..self
        }
    }

    /// Use reqwest blocking http client in the exporter.
    #[cfg(feature = "reqwest_blocking_collector_client")]
    pub fn with_reqwest_blocking(self) -> Self {
        Self {
            client_config: ClientConfig::Http {
                client_type: CollectorHttpClient::ReqwestBlocking,
            },
            ..self
        }
    }

    /// Use hyper http client in the exporter.
    #[cfg(feature = "hyper_collector_client")]
    pub fn with_hyper(self) -> Self {
        Self {
            client_config: ClientConfig::Http {
                client_type: CollectorHttpClient::Hyper,
            },
            ..self
        }
    }

    /// Set the service name of the application. It generally is the name of application.
    /// Critically, Jaeger backend depends on `Span.Process.ServiceName` to identify the service
    /// that produced the spans.
    ///
    /// Opentelemetry allows set the service name using multiple methods.
    /// This functions takes priority over all other methods.
    ///
    /// If the service name is not set. It will default to be `unknown_service`.
    pub fn with_service_name<T: Into<String>>(mut self, service_name: T) -> Self {
        self.set_transformation_config(|config| {
            config.service_name = Some(service_name.into());
        });
        self
    }

    /// Config whether to export information of instrumentation library.
    ///
    /// It's required to [report instrumentation library as span tags].
    /// However it does have a overhead on performance, performance sensitive applications can
    /// use this function to opt out reporting instrumentation library.
    ///
    /// Default to be `true`.
    ///
    /// [report instrumentation library as span tags]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/sdk_exporters/non-otlp.md#instrumentationscope
    pub fn with_instrumentation_library_tags(mut self, should_export: bool) -> Self {
        self.set_transformation_config(|config| {
            config.export_instrument_library = should_export;
        });
        self
    }

    /// Assign the opentelemetry SDK configurations for the exporter pipeline.
    ///
    /// For mapping between opentelemetry configurations and Jaeger spans. Please refer [the spec].
    ///
    /// [the spec]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/sdk_exporters/jaeger.md#mappings
    /// # Examples
    /// Set service name via resource.
    /// ```rust
    /// use opentelemetry::KeyValue;
    /// use opentelemetry_sdk::{Resource, trace::Config};
    ///
    /// let pipeline = opentelemetry_jaeger::new_collector_pipeline()
    ///                 .with_trace_config(
    ///                       Config::default()
    ///                         .with_resource(Resource::new(vec![KeyValue::new("service.name", "my-service")]))
    ///                 );
    ///
    /// ```
    pub fn with_trace_config(mut self, config: Config) -> Self {
        self.set_trace_config(config);
        self
    }

    /// Assign the batch span processor for the exporter pipeline.
    ///
    /// # Examples
    /// Set max queue size.
    /// ```rust
    /// use opentelemetry_sdk::trace::BatchConfigBuilder;
    ///
    /// let pipeline = opentelemetry_jaeger::new_collector_pipeline()
    ///                 .with_batch_processor_config(
    ///                       BatchConfigBuilder::default()
    ///                         .with_max_queue_size(200)
    ///                         .build()
    ///                 );
    ///
    /// ```
    pub fn with_batch_processor_config(mut self, config: BatchConfig) -> Self {
        self.set_batch_config(config);
        self
    }

    /// Build a `TracerProvider` using a async exporter and configurations from the pipeline.
    ///
    /// The exporter will collect spans in a batch and send them to the agent.
    ///
    /// It's possible to lose spans up to a batch when the application shuts down. So users should
    /// use [`shut_down_tracer_provider`] to block the shut down process until
    /// all remaining spans have been sent.
    ///
    /// Commonly used runtime are provided via `rt-tokio`, `rt-tokio-current-thread`, `rt-async-std`
    /// features.
    ///
    /// [`shut_down_tracer_provider`]: opentelemetry::global::shutdown_tracer_provider
    // todo: we don't need JaegerTraceRuntime, we only need otel runtime
    pub fn build_batch<R: JaegerTraceRuntime>(
        mut self,
        runtime: R,
    ) -> Result<TracerProvider, TraceError> {
        let mut builder = TracerProvider::builder();
        // build sdk trace config and jaeger process.
        // some attributes like service name has attributes like service name
        let export_instrument_library = self.transformation_config.export_instrument_library;
        let (config, process) = build_config_and_process(
            self.trace_config.take(),
            self.transformation_config.service_name.take(),
        );
        let batch_config = self.batch_config.take();
        let uploader = self.build_uploader::<R>()?;
        let exporter = Exporter::new(process.into(), export_instrument_library, uploader);
        let batch_processor = BatchSpanProcessor::builder(exporter, runtime)
            .with_batch_config(batch_config.unwrap_or_default())
            .build();

        builder = builder.with_span_processor(batch_processor);
        builder = builder.with_config(config);

        Ok(builder.build())
    }

    /// Similar to [`build_batch`][CollectorPipeline::build_batch] but also returns a tracer from the
    /// tracer provider.
    ///
    /// The tracer name is `opentelemetry-jaeger`. The tracer version will be the version of this crate.
    pub fn install_batch<R: JaegerTraceRuntime>(self, runtime: R) -> Result<Tracer, TraceError> {
        let tracer_provider = self.build_batch(runtime)?;
        install_tracer_provider_and_get_tracer(tracer_provider)
    }

    /// Build an jaeger exporter targeting a jaeger collector.
    pub fn build_collector_exporter<R>(mut self) -> Result<crate::Exporter, TraceError>
    where
        R: JaegerTraceRuntime,
    {
        let export_instrument_library = self.transformation_config.export_instrument_library;
        let (_, process) = build_config_and_process(
            self.trace_config.take(),
            self.transformation_config.service_name.take(),
        );
        let uploader = self.build_uploader::<R>()?;
        let exporter = Exporter::new(process.into(), export_instrument_library, uploader);
        Ok(exporter)
    }

    fn build_uploader<R>(self) -> Result<Arc<dyn Uploader>, crate::Error>
    where
        R: JaegerTraceRuntime,
    {
        let endpoint = self.resolve_endpoint()?;
        let username = self.resolve_username();
        let password = self.resolve_password();
        #[cfg(feature = "collector_client")]
        let timeout = self.resolve_timeout();
        match self.client_config {
            #[cfg(feature = "collector_client")]
            ClientConfig::Http { client_type } => {
                let client = client_type.build_client(username, password, timeout)?;

                let collector = AsyncHttpClient::new(endpoint, client);
                Ok(Arc::new(AsyncUploader::<R>::Collector(collector)))
            }
            #[cfg(feature = "wasm_collector_client")]
            ClientConfig::Wasm => {
                let collector = WasmCollector::new(endpoint, username, password)
                    .map_err::<crate::Error, _>(Into::into)?;
                Ok(Arc::new(AsyncUploader::<R>::WasmCollector(collector)))
            }
        }
    }

    fn resolve_env_var(env_var: &'static str) -> Option<String> {
        env::var(env_var).ok().filter(|var| !var.is_empty())
    }

    // if provided value from environment variable or the builder is invalid, return error
    fn resolve_endpoint(&self) -> Result<Uri, crate::Error> {
        let endpoint_from_env = Self::resolve_env_var(ENV_ENDPOINT)
            .map(|endpoint| {
                Uri::try_from(endpoint.as_str()).map_err::<crate::Error, _>(|err| {
                    crate::Error::ConfigError {
                        pipeline_name: "collector",
                        config_name: "collector_endpoint",
                        reason: format!("invalid uri from environment variable, {}", err),
                    }
                })
            })
            .transpose()?;

        Ok(match endpoint_from_env {
            Some(endpoint) => endpoint,
            None => {
                if let Some(endpoint) = &self.collector_endpoint {
                    Uri::try_from(endpoint.as_str()).map_err::<crate::Error, _>(|err| {
                        crate::Error::ConfigError {
                            pipeline_name: "collector",
                            config_name: "collector_endpoint",
                            reason: format!("invalid uri from the builder, {}", err),
                        }
                    })?
                } else {
                    Uri::try_from(DEFAULT_ENDPOINT).unwrap() // default endpoint should always valid
                }
            }
        })
    }

    #[cfg(feature = "collector_client")]
    fn resolve_timeout(&self) -> Duration {
        match Self::resolve_env_var(ENV_TIMEOUT) {
            Some(timeout) => match timeout.parse() {
                Ok(timeout) => Duration::from_millis(timeout),
                Err(e) => {
                    eprintln!("{} malformed default to 10s: {}", ENV_TIMEOUT, e);
                    self.collector_timeout
                }
            },
            None => self.collector_timeout,
        }
    }

    fn resolve_username(&self) -> Option<String> {
        Self::resolve_env_var(ENV_USERNAME).or_else(|| self.collector_username.clone())
    }

    fn resolve_password(&self) -> Option<String> {
        Self::resolve_env_var(ENV_PASSWORD).or_else(|| self.collector_password.clone())
    }
}

#[cfg(test)]
#[cfg(feature = "rt-tokio")]
mod tests {
    use opentelemetry_sdk::runtime::Tokio;

    use crate::config::collector::http_client::test_http_client;

    use super::*;

    #[test]
    fn test_set_collector_endpoint() {
        let invalid_uri = new_collector_pipeline()
            .with_endpoint("127.0.0.1:14268/api/traces")
            .with_http_client(test_http_client::TestHttpClient)
            .build_uploader::<Tokio>();
        assert!(invalid_uri.is_err());
        assert_eq!(
            format!("{:?}", invalid_uri.err().unwrap()),
            "ConfigError { pipeline_name: \"collector\", config_name: \"collector_endpoint\", reason: \"invalid uri from the builder, invalid format\" }",
        );

        let valid_uri = new_collector_pipeline()
            .with_http_client(test_http_client::TestHttpClient)
            .with_endpoint("http://127.0.0.1:14268/api/traces")
            .build_uploader::<Tokio>();

        assert!(valid_uri.is_ok());
    }

    // Ignore this test as it is flaky and the opentelemetry-jaeger is on-track for deprecation
    #[ignore]
    #[test]
    fn test_collector_exporter() {
        let exporter = new_collector_pipeline()
            .with_endpoint("http://127.0.0.1:14268/api/traces")
            .with_http_client(test_http_client::TestHttpClient)
            .build_collector_exporter::<Tokio>();
        assert!(exporter.is_ok());
    }

    #[test]
    fn test_resolve_endpoint() {
        struct TestCase<'a> {
            description: &'a str,
            env_var: &'a str,
            builder_endpoint: Option<&'a str>,
            expected_result: Result<Uri, crate::Error>,
        }
        let test_cases = vec![
            TestCase {
                description: "Positive: Endpoint from environment variable exists",
                env_var: "http://example.com",
                builder_endpoint: None,
                expected_result: Ok(Uri::try_from("http://example.com").unwrap()),
            },
            TestCase {
                description: "Positive: Endpoint from builder",
                env_var: "",
                builder_endpoint: Some("http://example.com"),
                expected_result: Ok(Uri::try_from("http://example.com").unwrap()),
            },
            TestCase {
                description: "Negative: Invalid URI from environment variable",
                env_var: "invalid random uri",
                builder_endpoint: None,
                expected_result: Err(crate::Error::ConfigError {
                    pipeline_name: "collector",
                    config_name: "collector_endpoint",
                    reason: "invalid uri from environment variable, invalid uri character"
                        .to_string(),
                }),
            },
            TestCase {
                description: "Negative: Invalid URI from builder",
                env_var: "",
                builder_endpoint: Some("invalid random uri"),
                expected_result: Err(crate::Error::ConfigError {
                    pipeline_name: "collector",
                    config_name: "collector_endpoint",
                    reason: "invalid uri from the builder, invalid uri character".to_string(),
                }),
            },
            TestCase {
                description: "Positive: Default endpoint (no environment variable set)",
                env_var: "",
                builder_endpoint: None,
                expected_result: Ok(Uri::try_from(DEFAULT_ENDPOINT).unwrap()),
            },
        ];
        for test_case in test_cases {
            env::set_var(ENV_ENDPOINT, test_case.env_var);
            let builder = CollectorPipeline {
                collector_endpoint: test_case.builder_endpoint.map(|s| s.to_string()),
                ..Default::default()
            };
            let result = builder.resolve_endpoint();
            match test_case.expected_result {
                Ok(expected) => {
                    assert_eq!(result.unwrap(), expected, "{}", test_case.description);
                }
                Err(expected_err) => {
                    assert!(
                        result.is_err(),
                        "{}, expected error, get {}",
                        test_case.description,
                        result.unwrap()
                    );
                    match (result.unwrap_err(), expected_err) {
                        (
                            crate::Error::ConfigError {
                                pipeline_name: result_pipeline_name,
                                config_name: result_config_name,
                                reason: result_reason,
                            },
                            crate::Error::ConfigError {
                                pipeline_name: expected_pipeline_name,
                                config_name: expected_config_name,
                                reason: expected_reason,
                            },
                        ) => {
                            assert_eq!(
                                result_pipeline_name, expected_pipeline_name,
                                "{}",
                                test_case.description
                            );
                            assert_eq!(
                                result_config_name, expected_config_name,
                                "{}",
                                test_case.description
                            );
                            assert_eq!(result_reason, expected_reason, "{}", test_case.description);
                        }
                        _ => panic!("we don't expect collector to return other error"),
                    }
                }
            }
            env::remove_var(ENV_ENDPOINT);
        }
    }

    #[test]
    fn test_resolve_timeout() {
        struct TestCase<'a> {
            description: &'a str,
            env_var: &'a str,
            builder_var: Option<Duration>,
            expected_duration: Duration,
        }
        let test_cases = vec![
            TestCase {
                description: "Valid environment variable",
                env_var: "5000",
                builder_var: None,
                expected_duration: Duration::from_millis(5000),
            },
            TestCase {
                description: "Invalid environment variable",
                env_var: "invalid",
                builder_var: None,
                expected_duration: DEFAULT_COLLECTOR_TIMEOUT,
            },
            TestCase {
                description: "Missing environment variable",
                env_var: "",
                builder_var: Some(Duration::from_millis(5000)),
                expected_duration: Duration::from_millis(5000),
            },
        ];
        for test_case in test_cases {
            env::set_var(ENV_TIMEOUT, test_case.env_var);
            let mut builder = CollectorPipeline::default();
            if let Some(timeout) = test_case.builder_var {
                builder = builder.with_timeout(timeout);
            }
            let result = builder.resolve_timeout();
            assert_eq!(
                result, test_case.expected_duration,
                "{}",
                test_case.description
            );
            env::remove_var(ENV_TIMEOUT);
        }
    }
}
