use std::borrow::BorrowMut;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::{env, net};

use opentelemetry::trace::TraceError;
use opentelemetry_sdk::trace::{BatchSpanProcessor, Tracer};
use opentelemetry_sdk::trace
    ::{BatchConfig, Config, TracerProvider};

use crate::exporter::agent::{AgentAsyncClientUdp, AgentSyncClientUdp};
use crate::exporter::config::{
    build_config_and_process, install_tracer_provider_and_get_tracer, HasRequiredConfig,
    TransformationConfig,
};
use crate::exporter::uploader::{AsyncUploader, SyncUploader, Uploader};
use crate::{Error, Exporter, JaegerTraceRuntime};

/// The max size of UDP packet we want to send, synced with jaeger-agent
const UDP_PACKET_MAX_LENGTH: usize = 65_000;

/// The hostname for the Jaeger agent.
/// e.g. "localhost"
const ENV_AGENT_HOST: &str = "OTEL_EXPORTER_JAEGER_AGENT_HOST";

/// The port for the Jaeger agent.
/// e.g. 6832
const ENV_AGENT_PORT: &str = "OTEL_EXPORTER_JAEGER_AGENT_PORT";

/// Default agent host if none is provided
const DEFAULT_AGENT_ENDPOINT_HOST: &str = "127.0.0.1";

/// Default agent port if none is provided
const DEFAULT_AGENT_ENDPOINT_PORT: &str = "6831";

/// Deprecation Notice:
/// Ingestion of OTLP is now supported in Jaeger please check [crates.io] for more details.
///
/// AgentPipeline config and build a exporter targeting a jaeger agent using UDP as transport layer protocol.
///
/// ## UDP packet max length
/// The exporter uses UDP to communicate with the agent. UDP requests may be rejected if it's too long.
/// See [UDP packet size] for details.
///
/// Users can utilise [`with_max_packet_size`] and [`with_auto_split_batch`] to avoid spans loss or UDP requests failure.
///
/// The default `max_packet_size` is `65000`([why 65000]?). If your platform has a smaller limit on UDP packet.
/// You will need to adjust the `max_packet_size` accordingly.
///
/// Set `auto_split_batch` to true will config the exporter to split the batch based on `max_packet_size`
/// automatically. Note that it has a performance overhead as every batch could require multiple requests to export.
///
/// For example, OSX UDP packet limit is 9216 by default. You can configure the pipeline as following
/// to avoid UDP packet breaches the limit.
///
/// ```no_run
/// # use opentelemetry_sdk::runtime::Tokio;
/// # fn main() {
///     let tracer = opentelemetry_jaeger::new_agent_pipeline()
///         .with_endpoint("localhost:6831")
///         .with_service_name("my_app")
///         .with_max_packet_size(9_216)
///         .with_auto_split_batch(true)
///         .install_batch(Tokio).unwrap();
/// # }
/// ```
///
/// [`with_auto_split_batch`]: AgentPipeline::with_auto_split_batch
/// [`with_max_packet_size`]: AgentPipeline::with_max_packet_size
/// [UDP packet size]: https://stackoverflow.com/questions/1098897/what-is-the-largest-safe-udp-packet-size-on-the-internet
/// [why 65000]: https://serverfault.com/questions/246508/how-is-the-mtu-is-65535-in-udp-but-ethernet-does-not-allow-frame-size-more-than
/// [crates.io]: https://crates.io/crates/opentelemetry-jaeger
///
/// ## Environment variables
/// The following environment variables are available to configure the agent exporter.
///
/// - `OTEL_EXPORTER_JAEGER_AGENT_HOST`, set the host of the agent. If the `OTEL_EXPORTER_JAEGER_AGENT_HOST`
/// is not set, the value will be ignored.
/// - `OTEL_EXPORTER_JAEGER_AGENT_PORT`, set the port of the agent. If the `OTEL_EXPORTER_JAEGER_AGENT_HOST`
/// is not set, the exporter will use 127.0.0.1 as the host.
#[derive(Debug)]
#[deprecated(
    since = "0.21.0",
    note = "Please migrate to opentelemetry-otlp exporter."
)]
pub struct AgentPipeline {
    transformation_config: TransformationConfig,
    trace_config: Option<Config>,
    batch_config: Option<BatchConfig>,
    agent_endpoint: Option<String>,
    max_packet_size: usize,
    auto_split_batch: bool,
}

impl Default for AgentPipeline {
    fn default() -> Self {
        AgentPipeline {
            transformation_config: Default::default(),
            trace_config: Default::default(),
            batch_config: Some(Default::default()),
            agent_endpoint: Some(format!(
                "{DEFAULT_AGENT_ENDPOINT_HOST}:{DEFAULT_AGENT_ENDPOINT_PORT}"
            )),
            max_packet_size: UDP_PACKET_MAX_LENGTH,
            auto_split_batch: false,
        }
    }
}

// implement the seal trait
impl HasRequiredConfig for AgentPipeline {
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

/// Start a new pipeline to configure a exporter that target a jaeger agent.
///
/// See details for each configurations at [`AgentPipeline`]
///
/// Deprecation Notice:
/// Ingestion of OTLP is now supported in Jaeger please check [crates.io] for more details.
///
/// [`AgentPipeline`]: crate::config::agent::AgentPipeline
/// [crates.io]: https://crates.io/crates/opentelemetry-jaeger
#[deprecated(
    since = "0.21.0",
    note = "Please migrate to opentelemetry-otlp exporter."
)]
pub fn new_agent_pipeline() -> AgentPipeline {
    AgentPipeline::default()
}

impl AgentPipeline {
    /// set the endpoint of the agent.
    ///
    /// It usually composed by host ip and the port number.
    /// Any valid socket address can be used.
    ///
    /// Default to be `127.0.0.1:6831`.
    pub fn with_endpoint<T: Into<String>>(self, agent_endpoint: T) -> Self {
        AgentPipeline {
            agent_endpoint: Some(agent_endpoint.into()),
            ..self
        }
    }

    /// Assign the max packet size in bytes.
    ///
    /// It should be consistent with the limit of platforms. Otherwise, UDP requests maybe reject with
    /// error like `thrift agent failed with transport error` or `thrift agent failed with message too long`.
    ///
    /// The exporter will cut off spans if the batch is long. To avoid this, set [auto_split_batch](AgentPipeline::with_auto_split_batch) to `true`
    /// to split a batch into multiple UDP packets.
    ///
    /// Default to be `65000`.
    pub fn with_max_packet_size(self, max_packet_size: usize) -> Self {
        AgentPipeline {
            max_packet_size,
            ..self
        }
    }

    /// Config whether to auto split batches.
    ///
    /// When auto split is set to `true`, the exporter will try to split the
    /// batch into smaller ones so that there will be minimal data loss. It
    /// will impact the performance.
    ///
    /// Note that if the length of one serialized span is longer than the `max_packet_size`.
    /// The exporter will return an error as it cannot export the span. Use jaeger collector
    /// instead of jaeger agent may be help in this case as the exporter will use HTTP to communicate
    /// with jaeger collector.
    ///
    /// Default to be `false`.
    pub fn with_auto_split_batch(mut self, should_auto_split: bool) -> Self {
        self.auto_split_batch = should_auto_split;
        self
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
    /// let pipeline = opentelemetry_jaeger::new_agent_pipeline()
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
    /// If a simple span processor is used by [`install_simple`][AgentPipeline::install_simple]
    /// or [`build_simple`][AgentPipeline::install_simple], then this config will not be ignored.
    ///
    /// # Examples
    /// Set max queue size.
    /// ```rust
    /// use opentelemetry_sdk::trace::BatchConfigBuilder;
    ///
    /// let pipeline = opentelemetry_jaeger::new_agent_pipeline()
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

    /// Build a `TracerProvider` using a blocking exporter and configurations from the pipeline.
    ///
    /// The exporter will send each span to the agent upon the span ends.
    pub fn build_simple(mut self) -> Result<TracerProvider, TraceError> {
        let mut builder = TracerProvider::builder();

        let (config, process) = build_config_and_process(
            self.trace_config.take(),
            self.transformation_config.service_name.take(),
        );
        let exporter = Exporter::new(
            process.into(),
            self.transformation_config.export_instrument_library,
            self.build_sync_agent_uploader()?,
        );

        builder = builder.with_simple_exporter(exporter);
        builder = builder.with_config(config);

        Ok(builder.build())
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
    pub fn build_batch<R>(mut self, runtime: R) -> Result<TracerProvider, TraceError>
    where
        R: JaegerTraceRuntime,
    {
        let mut builder = TracerProvider::builder();

        let export_instrument_library = self.transformation_config.export_instrument_library;
        // build sdk trace config and jaeger process.
        // some attributes like service name has attributes like service name
        let (config, process) = build_config_and_process(
            self.trace_config.take(),
            self.transformation_config.service_name.take(),
        );
        let batch_config = self.batch_config.take();
        let uploader = self.build_async_agent_uploader(runtime.clone())?;
        let exporter = Exporter::new(process.into(), export_instrument_library, uploader);
        let batch_processor = BatchSpanProcessor::builder(exporter, runtime)
            .with_batch_config(batch_config.unwrap_or_default())
            .build();

        builder = builder.with_span_processor(batch_processor);
        builder = builder.with_config(config);

        Ok(builder.build())
    }

    /// Similar to [`build_simple`][AgentPipeline::build_simple] but also returns a tracer from the
    /// tracer provider.
    ///
    /// The tracer name is `opentelemetry-jaeger`. The tracer version will be the version of this crate.
    pub fn install_simple(self) -> Result<Tracer, TraceError> {
        let tracer_provider = self.build_simple()?;
        install_tracer_provider_and_get_tracer(tracer_provider)
    }

    /// Similar to [`build_batch`][AgentPipeline::build_batch] but also returns a tracer from the
    /// tracer provider.
    ///
    /// The tracer name is `opentelemetry-jaeger`. The tracer version will be the version of this crate.
    pub fn install_batch<R>(self, runtime: R) -> Result<Tracer, TraceError>
    where
        R: JaegerTraceRuntime,
    {
        let tracer_provider = self.build_batch(runtime)?;
        install_tracer_provider_and_get_tracer(tracer_provider)
    }

    /// Build an jaeger exporter targeting a jaeger agent and running on the async runtime.
    pub fn build_async_agent_exporter<R>(
        mut self,
        runtime: R,
    ) -> Result<crate::Exporter, TraceError>
    where
        R: JaegerTraceRuntime,
    {
        let export_instrument_library = self.transformation_config.export_instrument_library;
        // build sdk trace config and jaeger process.
        // some attributes like service name has attributes like service name
        let (_, process) = build_config_and_process(
            self.trace_config.take(),
            self.transformation_config.service_name.take(),
        );
        let uploader = self.build_async_agent_uploader(runtime)?;
        Ok(Exporter::new(
            process.into(),
            export_instrument_library,
            uploader,
        ))
    }

    /// Build an jaeger exporter targeting a jaeger agent and running on the sync runtime.
    pub fn build_sync_agent_exporter(mut self) -> Result<crate::Exporter, TraceError> {
        let (_, process) = build_config_and_process(
            self.trace_config.take(),
            self.transformation_config.service_name.take(),
        );
        Ok(Exporter::new(
            process.into(),
            self.transformation_config.export_instrument_library,
            self.build_sync_agent_uploader()?,
        ))
    }

    fn build_async_agent_uploader<R>(self, runtime: R) -> Result<Arc<dyn Uploader>, TraceError>
    where
        R: JaegerTraceRuntime,
    {
        let agent = AgentAsyncClientUdp::new(
            self.max_packet_size,
            runtime,
            self.auto_split_batch,
            self.resolve_endpoint()?,
        )
        .map_err::<Error, _>(Into::into)?;
        Ok(Arc::new(AsyncUploader::Agent(
            futures_util::lock::Mutex::new(agent),
        )))
    }

    fn build_sync_agent_uploader(self) -> Result<Arc<dyn Uploader>, TraceError> {
        let agent = AgentSyncClientUdp::new(
            self.max_packet_size,
            self.auto_split_batch,
            self.resolve_endpoint()?,
        )
        .map_err::<Error, _>(Into::into)?;
        Ok(Arc::new(SyncUploader::Agent(std::sync::Mutex::new(agent))))
    }

    // resolve the agent endpoint from the environment variables or the builder
    // if only one of the environment variables is set, the other one will be set to the default value
    // if no environment variable is set, the builder value will be used.
    fn resolve_endpoint(self) -> Result<Vec<net::SocketAddr>, TraceError> {
        let endpoint_str = match (env::var(ENV_AGENT_HOST), env::var(ENV_AGENT_PORT)) {
            (Ok(host), Ok(port)) => format!("{}:{}", host.trim(), port.trim()),
            (Ok(host), _) => format!("{}:{DEFAULT_AGENT_ENDPOINT_PORT}", host.trim()),
            (_, Ok(port)) => format!("{DEFAULT_AGENT_ENDPOINT_HOST}:{}", port.trim()),
            (_, _) => self.agent_endpoint.unwrap_or(format!(
                "{DEFAULT_AGENT_ENDPOINT_HOST}:{DEFAULT_AGENT_ENDPOINT_PORT}"
            )),
        };
        endpoint_str
            .to_socket_addrs()
            .map(|addrs| addrs.collect())
            .map_err(|io_err| {
                Error::ConfigError {
                    pipeline_name: "agent",
                    config_name: "endpoint",
                    reason: io_err.to_string(),
                }
                .into()
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::config::agent::AgentPipeline;

    #[test]
    fn set_socket_address() {
        let test_cases = vec![
            // invalid inputs
            ("invalid_endpoint", false),
            ("0.0.0.0.0:9123", false),
            ("127.0.0.1", false), // port is needed
            // valid inputs
            ("[::0]:9123", true),
            ("127.0.0.1:1001", true),
        ];
        for (socket_str, is_ok) in test_cases.into_iter() {
            let resolved_endpoint = AgentPipeline::default()
                .with_endpoint(socket_str)
                .resolve_endpoint();
            assert_eq!(
                resolved_endpoint.is_ok(),
                // if is_ok is true, use socket_str, otherwise use the default endpoint
                is_ok,
                "endpoint string {}",
                socket_str
            );
        }
    }
}
