use crate::PipelineBuilder;
use std::env;
#[cfg(feature = "collector_client")]
use std::time::Duration;

/// The hostname for the Jaeger agent.
/// e.g. "localhost"
const ENV_AGENT_HOST: &str = "OTEL_EXPORTER_JAEGER_AGENT_HOST";

/// The port for the Jaeger agent.
/// e.g. 6832
const ENV_AGENT_PORT: &str = "OTEL_EXPORTER_JAEGER_AGENT_PORT";

/// HTTP endpoint for Jaeger collector.
/// e.g. "http://localhost:14250"
#[cfg(feature = "collector_client")]
const ENV_ENDPOINT: &str = "OTEL_EXPORTER_JAEGER_ENDPOINT";

/// Timeout for Jaeger collector.
#[cfg(feature = "collector_client")]
pub(crate) const ENV_TIMEOUT: &str = "OTEL_EXPORTER_JAEGER_TIMEOUT";

/// Default of 10s
#[cfg(feature = "collector_client")]
pub(crate) const DEFAULT_COLLECTOR_TIMEOUT: Duration = Duration::from_secs(10);

/// Username to send as part of "Basic" authentication to the collector endpoint.
#[cfg(feature = "collector_client")]
const ENV_USER: &str = "OTEL_EXPORTER_JAEGER_USER";

/// Password to send as part of "Basic" authentication to the collector endpoint.
#[cfg(feature = "collector_client")]
const ENV_PASSWORD: &str = "OTEL_EXPORTER_JAEGER_PASSWORD";

/// Assign builder attributes from env
pub(crate) fn assign_attrs(mut builder: PipelineBuilder) -> PipelineBuilder {
    if let (Ok(host), Ok(port)) = (env::var(ENV_AGENT_HOST), env::var(ENV_AGENT_PORT)) {
        builder = builder.with_agent_endpoint(format!("{}:{}", host.trim(), port.trim()));
    }

    #[cfg(feature = "collector_client")]
    {
        if let Some(timeout) = env::var(ENV_TIMEOUT).ok().filter(|var| !var.is_empty()) {
            let timeout = match timeout.parse() {
                Ok(timeout) => Duration::from_millis(timeout),
                Err(e) => {
                    eprintln!("{} malformed defaulting to 10000: {}", ENV_TIMEOUT, e);
                    DEFAULT_COLLECTOR_TIMEOUT
                }
            };
            builder = builder.with_collector_timeout(timeout);
        }
        if let Some(endpoint) = env::var(ENV_ENDPOINT).ok().filter(|var| !var.is_empty()) {
            builder = builder.with_collector_endpoint(endpoint);
        }

        if let Some(user) = env::var(ENV_USER).ok().filter(|var| !var.is_empty()) {
            builder = builder.with_collector_username(user);
        }
        if let Some(password) = env::var(ENV_PASSWORD).ok().filter(|var| !var.is_empty()) {
            builder = builder.with_collector_password(password);
        }
    }

    builder
}
