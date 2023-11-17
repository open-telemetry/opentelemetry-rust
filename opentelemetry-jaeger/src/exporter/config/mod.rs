//! Configurations to build a jaeger exporter.
//!
//! The jaeger exporter can send spans to [jaeger agent] or [jaeger collector]. The agent is usually
//! deployed along with the application like a sidecar. The collector is usually deployed a stand alone
//! application and receive spans from multiple sources. The exporter will use UDP to send spans to
//! agents and use HTTP/TCP to send spans to collectors. See [jaeger deployment guide] for more details.
//!
//! [jaeger agent]: https://www.jaegertracing.io/docs/1.31/deployment/#agent
//! [jaeger collector]: https://www.jaegertracing.io/docs/1.31/deployment/#collector
//! [jaeger deployment guide]: https://www.jaegertracing.io/docs/1.31/deployment

use crate::Process;
use opentelemetry::{global, trace::TraceError, KeyValue};
use opentelemetry_sdk::trace::{BatchConfig, Config, Tracer, TracerProvider};
use opentelemetry_semantic_conventions as semcov;

/// Config a exporter that sends the spans to a [jaeger agent](https://www.jaegertracing.io/docs/1.31/deployment/#agent).
pub mod agent;
/// Config a exporter that bypass the agent and send spans directly to [jaeger collector](https://www.jaegertracing.io/docs/1.31/deployment/#collector).
#[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
pub mod collector;

// configurations and overrides on how to transform OTLP spans to Jaeger spans.
#[derive(Debug)]
struct TransformationConfig {
    export_instrument_library: bool,
    service_name: Option<String>,
}

impl Default for TransformationConfig {
    fn default() -> Self {
        TransformationConfig {
            export_instrument_library: true,
            service_name: None,
        }
    }
}

// pipeline must have transformation config, trace config and batch config.
trait HasRequiredConfig {
    fn set_transformation_config<T>(&mut self, f: T)
    where
        T: FnOnce(&mut TransformationConfig);

    fn set_trace_config(&mut self, config: Config);

    fn set_batch_config(&mut self, config: BatchConfig);
}

// To reduce the overhead of copying service name in every spans. We convert resource into jaeger tags
// and store them into process. And set the resource in trace config to empty.
//
// There are multiple ways to set the service name. A `service.name` tag will be always added
// to the process tags.
fn build_config_and_process(
    config: Option<Config>,
    service_name_opt: Option<String>,
) -> (Config, Process) {
    let config = config.unwrap_or_default();

    let service_name = service_name_opt.unwrap_or_else(|| {
        config
            .resource
            .get(semcov::resource::SERVICE_NAME.into())
            .map(|v| v.to_string())
            .unwrap_or_else(|| "unknown_service".to_string())
    });

    // merge the tags and resource. Resources take priority.
    let mut tags = config
        .resource
        .iter()
        .filter(|(key, _)| key.as_str() != semcov::resource::SERVICE_NAME)
        .map(|(key, value)| KeyValue::new(key.clone(), value.clone()))
        .collect::<Vec<KeyValue>>();

    tags.push(KeyValue::new(
        semcov::resource::SERVICE_NAME,
        service_name.clone(),
    ));

    (config, Process { service_name, tags })
}

#[cfg(test)]
mod tests {
    use crate::exporter::config::build_config_and_process;
    use crate::new_agent_pipeline;
    use opentelemetry::KeyValue;
    use opentelemetry_sdk::{trace::Config, Resource};
    use std::env;

    #[test]
    fn test_set_service_name() {
        let service_name = "halloween_service".to_string();

        // set via builder's service name, it has highest priority
        let (_, process) = build_config_and_process(None, Some(service_name.clone()));
        assert_eq!(process.service_name, service_name);

        // make sure the tags in resource are moved to process
        let trace_config = Config::default()
            .with_resource(Resource::new(vec![KeyValue::new("test-key", "test-value")]));
        let (_, process) = build_config_and_process(Some(trace_config), Some(service_name));
        assert_eq!(process.tags.len(), 2);
    }

    #[tokio::test]
    async fn test_read_from_env() {
        // OTEL_SERVICE_NAME env var also works
        env::set_var("OTEL_SERVICE_NAME", "test service");
        let builder = new_agent_pipeline();
        let exporter = builder.build_sync_agent_exporter().unwrap();
        assert_eq!(exporter.process.service_name, "test service");
        env::set_var("OTEL_SERVICE_NAME", "")
    }
}

pub(crate) fn install_tracer_provider_and_get_tracer(
    tracer_provider: TracerProvider,
) -> Result<Tracer, TraceError> {
    let tracer = opentelemetry::trace::TracerProvider::versioned_tracer(
        &tracer_provider,
        "opentelemetry-jaeger",
        Some(env!("CARGO_PKG_VERSION")),
        Some(semcov::SCHEMA_URL),
        None,
    );
    let _ = global::set_tracer_provider(tracer_provider);
    Ok(tracer)
}
