use crate::exporter::agent::{AgentAsyncClientUdp, AgentSyncClientUdp};
use crate::exporter::config::{
    build_config_and_process,
    common::{HasRequiredConfig, TransformationConfig},
    install_tracer_provider_and_get_tracer,
};
use crate::exporter::uploader::{AsyncUploader, SyncUploader, Uploader};
use crate::{Error, Exporter, JaegerTraceRuntime};
use opentelemetry::sdk;
use opentelemetry::sdk::trace::{Config, TracerProvider};
use opentelemetry::trace::TraceError;
use std::borrow::BorrowMut;
use std::{env, net};

/// The max size of UDP packet we want to send, synced with jaeger-agent
const UDP_PACKET_MAX_LENGTH: usize = 65_000;

/// The hostname for the Jaeger agent.
/// e.g. "localhost"
const ENV_AGENT_HOST: &str = "OTEL_EXPORTER_JAEGER_AGENT_HOST";

/// The port for the Jaeger agent.
/// e.g. 6832
const ENV_AGENT_PORT: &str = "OTEL_EXPORTER_JAEGER_AGENT_PORT";

/// Default agent endpoint if none is provided
const DEFAULT_AGENT_ENDPOINT: &str = "127.0.0.1:6831";

/// AgentPipeline config and build a exporter targeting a jaeger agent using UDP as transport layer protocol.
///
/// ## UDP packet max length
/// The exporter will use UDP to communicate with the agent. Depends on your platform, UDP protocol
/// may cut off if the packet is too long. See [UDP packet size] for details. Users can utilise [`with_max_packet_size`]
/// and [`with_auto_split_batch`] to avoid spans don't lose because the packet is too long.
///
/// [`with_auto_split_batch`]: AgentPipeline::with_auto_split_batch
/// [`with_max_packet_size`]: AgentPipeline::with_max_packet_size
/// [UDP packet size]: https://stackoverflow.com/questions/1098897/what-is-the-largest-safe-udp-packet-size-on-the-internet
///
/// ## Environment variables
/// The following environment variables are available to configure the agent exporter.
///
/// - `OTEL_EXPORTER_JAEGER_AGENT_HOST`, set the host of the agent. If the `OTEL_EXPORTER_JAEGER_AGENT_HOST`
/// is not set, the value will be ignored.
/// - `OTEL_EXPORTER_JAEGER_AGENT_PORT`, set the port of the agent. If the `OTEL_EXPORTER_JAEGER_AGENT_HOST`
/// is not set, the exporter will use 127.0.0.1 as the host.
#[derive(Debug)]
pub struct AgentPipeline {
    common_config: TransformationConfig,
    trace_config: Option<sdk::trace::Config>,
    agent_endpoint: Result<Vec<net::SocketAddr>, crate::Error>,
    max_packet_size: usize,
    auto_split_batch: bool,
}

impl Default for AgentPipeline {
    fn default() -> Self {
        let mut pipeline = AgentPipeline {
            common_config: Default::default(),
            trace_config: Default::default(),
            agent_endpoint: Ok(vec![DEFAULT_AGENT_ENDPOINT.parse().unwrap()]),
            max_packet_size: UDP_PACKET_MAX_LENGTH,
            auto_split_batch: false,
        };

        if let (Ok(host), Ok(port)) = (env::var(ENV_AGENT_HOST), env::var(ENV_AGENT_PORT)) {
            pipeline = pipeline.with_endpoint(format!("{}:{}", host.trim(), port.trim()));
        } else if let Ok(port) = env::var(ENV_AGENT_PORT) {
            pipeline = pipeline.with_endpoint(format!("127.0.0.1:{}", port.trim()))
        }
        pipeline
    }
}

// implement the seal trait
impl HasRequiredConfig for AgentPipeline {
    fn set_transformation_config<T>(&mut self, f: T)
    where
        T: FnOnce(&mut TransformationConfig),
    {
        f(self.common_config.borrow_mut())
    }

    fn set_trace_config(&mut self, config: Config) {
        self.trace_config = Some(config)
    }
}

/// Start a new pipeline to configure a exporter that target a jaeger agent.
///
/// See details for each configurations at [`AgentPipeline`]
///
/// [`AgentPipeline`]: crate::config::agent::AgentPipeline
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
    pub fn with_endpoint<T: net::ToSocketAddrs>(self, agent_endpoint: T) -> Self {
        AgentPipeline {
            agent_endpoint: agent_endpoint
                .to_socket_addrs()
                .map(|addrs| addrs.collect())
                .map_err(|io_err| crate::Error::ConfigError {
                    pipeline_name: "agent",
                    config_name: "endpoint",
                    reason: io_err.to_string(),
                }),
            ..self
        }
    }

    /// Assign the max packet size in bytes.
    ///
    /// If the application is generating a lot of spans or each spans contains a lot of events/tags
    /// it can result in spans loss because of the UDP size limit. Increase the `max_packet_size` can medicate the problem.
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
    /// Note that if one span is too large to export, other spans within the
    /// same batch may or may not be exported. In this case, exporter will
    /// return errors as we cannot split spans.
    ///
    /// Default to be `false`.
    pub fn with_auto_split_batch(mut self, should_auto_split: bool) -> Self {
        self.auto_split_batch = should_auto_split;
        self
    }

    /// Build a `TracerProvider` using a blocking exporter and configurations from the pipeline.
    ///
    /// The exporter will send each span to the agent upon the span ends.
    pub fn build_simple(mut self) -> Result<TracerProvider, TraceError> {
        let mut builder = sdk::trace::TracerProvider::builder();

        let (config, process) = build_config_and_process(
            builder.sdk_provided_resource(),
            self.trace_config.take(),
            self.common_config.service_name.take(),
        );
        let exporter = Exporter::new(
            process.into(),
            self.common_config.export_instrument_library,
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
        let mut builder = sdk::trace::TracerProvider::builder();

        let export_instrument_library = self.common_config.export_instrument_library;
        // build sdk trace config and jaeger process.
        // some attributes like service name has attributes like service name
        let (config, process) = build_config_and_process(
            builder.sdk_provided_resource(),
            self.trace_config.take(),
            self.common_config.service_name.take(),
        );
        let uploader = self.build_async_agent_uploader(runtime.clone())?;
        let exporter = Exporter::new(process.into(), export_instrument_library, uploader);

        builder = builder.with_batch_exporter(exporter, runtime);
        builder = builder.with_config(config);

        Ok(builder.build())
    }

    /// Similar to [`build_simple`][AgentPipeline::build_simple] but also returns a tracer from the
    /// tracer provider.
    ///
    /// The tracer name is `opentelemetry-jaeger`. The tracer version will be the version of this crate.
    pub fn install_simple(self) -> Result<sdk::trace::Tracer, TraceError> {
        let tracer_provider = self.build_simple()?;
        install_tracer_provider_and_get_tracer(tracer_provider)
    }

    /// Similar to [`build_batch`][AgentPipeline::build_batch] but also returns a tracer from the
    /// tracer provider.
    ///
    /// The tracer name is `opentelemetry-jaeger`. The tracer version will be the version of this crate.
    pub fn install_batch<R>(self, runtime: R) -> Result<sdk::trace::Tracer, TraceError>
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
        let builder = sdk::trace::TracerProvider::builder();
        let export_instrument_library = self.common_config.export_instrument_library;
        // build sdk trace config and jaeger process.
        // some attributes like service name has attributes like service name
        let (_, process) = build_config_and_process(
            builder.sdk_provided_resource(),
            self.trace_config.take(),
            self.common_config.service_name.take(),
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
        let builder = sdk::trace::TracerProvider::builder();
        let (_, process) = build_config_and_process(
            builder.sdk_provided_resource(),
            self.trace_config.take(),
            self.common_config.service_name.take(),
        );
        Ok(Exporter::new(
            process.into(),
            self.common_config.export_instrument_library,
            self.build_sync_agent_uploader()?,
        ))
    }

    fn build_async_agent_uploader<R>(self, runtime: R) -> Result<Box<dyn Uploader>, TraceError>
    where
        R: JaegerTraceRuntime,
    {
        let agent = AgentAsyncClientUdp::new(
            self.agent_endpoint?.as_slice(),
            self.max_packet_size,
            runtime,
            self.auto_split_batch,
        )
        .map_err::<Error, _>(Into::into)?;
        Ok(Box::new(AsyncUploader::Agent(agent)))
    }

    fn build_sync_agent_uploader(self) -> Result<Box<dyn Uploader>, TraceError> {
        let agent = AgentSyncClientUdp::new(
            self.agent_endpoint?.as_slice(),
            self.max_packet_size,
            self.auto_split_batch,
        )
        .map_err::<Error, _>(Into::into)?;
        Ok(Box::new(SyncUploader::Agent(agent)))
    }
}
