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
use opentelemetry::sdk::trace::Config;
use opentelemetry::sdk::Resource;
use opentelemetry::trace::{TraceError, TracerProvider};
use opentelemetry::{global, sdk, KeyValue};
use opentelemetry_semantic_conventions as semcov;
use std::sync::Arc;

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

// pipeline must have transformation config and trace config.
trait HasRequiredConfig {
    fn set_transformation_config<T>(&mut self, f: T)
    where
        T: FnOnce(&mut TransformationConfig);

    fn set_trace_config(&mut self, config: sdk::trace::Config);
}

// To reduce the overhead of copying service name in every spans. We convert resource into jaeger tags
// and store them into process. And set the resource in trace config to empty.
//
// There are multiple ways to set the service name. A `service.name` tag will be always added
// to the process tags.
fn build_config_and_process(
    sdk_resource: sdk::Resource,
    mut config: Option<sdk::trace::Config>,
    service_name_opt: Option<String>,
) -> (sdk::trace::Config, Process) {
    let (config, resource) = if let Some(mut config) = config.take() {
        let resource = if let Some(resource) = config.resource.replace(Arc::new(Resource::empty()))
        {
            sdk_resource.merge(resource)
        } else {
            sdk_resource
        };

        (config, resource)
    } else {
        (Config::default(), sdk_resource)
    };

    let service_name = service_name_opt.unwrap_or_else(|| {
        resource
            .get(semcov::resource::SERVICE_NAME)
            .map(|v| v.to_string())
            .unwrap_or_else(|| "unknown_service".to_string())
    });

    // merge the tags and resource. Resources take priority.
    let mut tags = resource
        .into_iter()
        .filter(|(key, _)| *key != semcov::resource::SERVICE_NAME)
        .map(|(key, value)| KeyValue::new(key, value))
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
    use opentelemetry::sdk::trace::Config;
    use opentelemetry::sdk::Resource;
    use opentelemetry::KeyValue;
    use std::env;
    use std::sync::Arc;

    #[test]
    fn test_set_service_name() {
        let service_name = "halloween_service".to_string();

        // set via builder's service name, it has highest priority
        let (_, process) =
            build_config_and_process(Resource::empty(), None, Some(service_name.clone()));
        assert_eq!(process.service_name, service_name);

        // make sure the tags in resource are moved to process
        let trace_config = Config::default()
            .with_resource(Resource::new(vec![KeyValue::new("test-key", "test-value")]));
        let (config, process) =
            build_config_and_process(Resource::empty(), Some(trace_config), Some(service_name));
        assert_eq!(config.resource, Some(Arc::new(Resource::empty())));
        assert_eq!(process.tags.len(), 2);

        // sdk provided resource can override service name if users didn't provided service name to builder
        let (_, process) = build_config_and_process(
            Resource::new(vec![KeyValue::new("service.name", "halloween_service")]),
            None,
            None,
        );
        assert_eq!(process.service_name, "halloween_service");

        // users can also provided service.name from config's resource, in this case, it will override the
        // sdk provided service name
        let trace_config = Config::default().with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            "override_service",
        )]));
        let (_, process) = build_config_and_process(
            Resource::new(vec![KeyValue::new("service.name", "halloween_service")]),
            Some(trace_config),
            None,
        );

        assert_eq!(process.service_name, "override_service");
        assert_eq!(process.tags.len(), 1);
        assert_eq!(
            process.tags[0],
            KeyValue::new("service.name", "override_service")
        );
    }

    #[test]
    fn test_read_from_env() {
        // OTEL_SERVICE_NAME env var also works
        env::set_var("OTEL_SERVICE_NAME", "test service");
        let builder = new_agent_pipeline();
        let exporter = builder.build_sync_agent_exporter().unwrap();
        assert_eq!(exporter.process.service_name, "test service");
        env::set_var("OTEL_SERVICE_NAME", "")
    }
}

pub(crate) fn install_tracer_provider_and_get_tracer(
    tracer_provider: sdk::trace::TracerProvider,
) -> Result<sdk::trace::Tracer, TraceError> {
    let tracer = tracer_provider.versioned_tracer(
        "opentelemetry-jaeger",
        Some(env!("CARGO_PKG_VERSION")),
        None,
    );
    let _ = global::set_tracer_provider(tracer_provider);
    Ok(tracer)
}
