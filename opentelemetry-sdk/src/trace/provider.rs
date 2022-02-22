//! # Trace Provider SDK
//!
//! ## Tracer Creation
//!
//! New `Tracer` instances are always created through a `TracerProvider`.
//!
//! All configuration objects and extension points (span processors,
//! propagators) are provided by the `TracerProvider`. `Tracer` instances do
//! not duplicate this data to avoid that different `Tracer` instances
//! of the `TracerProvider` have different versions of these data.
use crate::resource::{EnvResourceDetector, SdkProvidedResourceDetector};
use crate::trace::{runtime::TraceRuntime, BatchSpanProcessor, SimpleSpanProcessor, Tracer};
use crate::{export::trace::SpanExporter, trace::SpanProcessor};
use crate::{InstrumentationLibrary, Resource};
use opentelemetry_api::{global, trace::TraceResult};
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;

/// Default tracer name if empty string is provided.
const DEFAULT_COMPONENT_NAME: &str = "rust.opentelemetry.io/sdk/tracer";

/// TracerProvider inner type
#[derive(Debug)]
pub(crate) struct TracerProviderInner {
    processors: Vec<Box<dyn SpanProcessor>>,
    config: crate::trace::Config,
}

impl Drop for TracerProviderInner {
    fn drop(&mut self) {
        for processor in &mut self.processors {
            if let Err(err) = processor.shutdown() {
                global::handle_error(err);
            }
        }
    }
}

/// Creator and registry of named `Tracer` instances.
#[derive(Clone, Debug)]
pub struct TracerProvider {
    inner: Arc<TracerProviderInner>,
}

impl Default for TracerProvider {
    fn default() -> Self {
        TracerProvider::builder().build()
    }
}

impl TracerProvider {
    /// Build a new tracer provider
    pub(crate) fn new(inner: Arc<TracerProviderInner>) -> Self {
        TracerProvider { inner }
    }

    /// Create a new `TracerProvider` builder.
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Span processors associated with this provider
    pub fn span_processors(&self) -> &Vec<Box<dyn SpanProcessor>> {
        &self.inner.processors
    }

    /// Config associated with this tracer
    pub fn config(&self) -> &crate::trace::Config {
        &self.inner.config
    }
}

impl opentelemetry_api::trace::TracerProvider for TracerProvider {
    /// This implementation of `TracerProvider` produces `Tracer` instances.
    type Tracer = crate::trace::Tracer;

    /// Create a new versioned `Tracer` instance.
    fn versioned_tracer(
        &self,
        name: impl Into<Cow<'static, str>>,
        version: Option<&'static str>,
        schema_url: Option<&'static str>,
    ) -> Self::Tracer {
        let name = name.into();
        // Use default value if name is invalid empty string
        let component_name = if name.is_empty() {
            Cow::Borrowed(DEFAULT_COMPONENT_NAME)
        } else {
            name
        };
        let instrumentation_lib =
            InstrumentationLibrary::new(component_name, version.map(Into::into));

        Tracer::new(instrumentation_lib, Arc::downgrade(&self.inner), schema_url)
    }

    /// Force flush all remaining spans in span processors and return results.
    fn force_flush(&self) -> Vec<TraceResult<()>> {
        self.span_processors()
            .iter()
            .map(|processor| processor.force_flush())
            .collect()
    }
}

/// Builder for provider attributes.
#[derive(Debug)]
pub struct Builder {
    processors: Vec<Box<dyn SpanProcessor>>,
    config: crate::trace::Config,
    sdk_provided_resource: Resource,
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            processors: Default::default(),
            config: Default::default(),
            sdk_provided_resource: Resource::from_detectors(
                Duration::from_secs(0),
                vec![
                    Box::new(SdkProvidedResourceDetector),
                    Box::new(EnvResourceDetector::new()),
                ],
            ),
        }
    }
}

impl Builder {
    /// The `SpanExporter` that this provider should use.
    pub fn with_simple_exporter<T: SpanExporter + 'static>(self, exporter: T) -> Self {
        let mut processors = self.processors;
        processors.push(Box::new(SimpleSpanProcessor::new(Box::new(exporter))));

        Builder { processors, ..self }
    }

    /// The `SpanExporter` setup using a default `BatchSpanProcessor` that this provider should use.
    pub fn with_batch_exporter<T: SpanExporter + 'static, R: TraceRuntime>(
        self,
        exporter: T,
        runtime: R,
    ) -> Self {
        let batch = BatchSpanProcessor::builder(exporter, runtime).build();
        self.with_span_processor(batch)
    }

    /// The `SpanProcessor` that this provider should use.
    pub fn with_span_processor<T: SpanProcessor + 'static>(self, processor: T) -> Self {
        let mut processors = self.processors;
        processors.push(Box::new(processor));

        Builder { processors, ..self }
    }

    /// The sdk `Config` that this provider will use.
    pub fn with_config(self, config: crate::trace::Config) -> Self {
        Builder { config, ..self }
    }

    /// Return the clone of sdk provided resource.
    ///
    /// See <https://github.com/open-telemetry/opentelemetry-specification/blob/v1.8.0/specification/resource/sdk.md#sdk-provided-resource-attributes>
    /// for details.
    pub fn sdk_provided_resource(&self) -> Resource {
        self.sdk_provided_resource.clone()
    }

    /// Create a new provider from this configuration.
    pub fn build(self) -> TracerProvider {
        let mut config = self.config;
        config.resource = match config.resource {
            None => Some(Arc::new(self.sdk_provided_resource)),
            // User provided resource information has higher priority.
            Some(resource) => {
                if resource.is_empty() {
                    None
                } else {
                    Some(Arc::new(self.sdk_provided_resource.merge(resource)))
                }
            }
        };
        TracerProvider {
            inner: Arc::new(TracerProviderInner {
                processors: self.processors,
                config,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::export::trace::SpanData;
    use crate::trace::provider::TracerProviderInner;
    use crate::trace::{Config, Span, SpanProcessor};
    use crate::Resource;
    use opentelemetry_api::trace::{TraceError, TraceResult, TracerProvider};
    use opentelemetry_api::{Context, Key, KeyValue};
    use std::env;
    use std::sync::Arc;

    #[derive(Debug)]
    struct TestSpanProcessor {
        success: bool,
    }

    impl SpanProcessor for TestSpanProcessor {
        fn on_start(&self, _span: &mut Span, _cx: &Context) {
            unimplemented!()
        }

        fn on_end(&self, _span: SpanData) {
            unimplemented!()
        }

        fn force_flush(&self) -> TraceResult<()> {
            if self.success {
                Ok(())
            } else {
                Err(TraceError::from("cannot export"))
            }
        }

        fn shutdown(&mut self) -> TraceResult<()> {
            self.force_flush()
        }
    }

    #[test]
    fn test_force_flush() {
        let tracer_provider = super::TracerProvider::new(Arc::from(TracerProviderInner {
            processors: vec![
                Box::from(TestSpanProcessor { success: true }),
                Box::from(TestSpanProcessor { success: false }),
            ],
            config: Default::default(),
        }));

        let results = tracer_provider.force_flush();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_tracer_provider_default_resource() {
        // If users didn't provided a resource and there isn't a env var set. Use default one
        let assert_service_name = |provider: super::TracerProvider,
                                   expect: Option<&'static str>| {
            assert_eq!(
                provider.config().resource.as_ref().and_then(|r| r
                    .get(Key::from_static_str("service.name"))
                    .map(|v| v.to_string())),
                expect.map(|s| s.to_string())
            );
        };
        let default_config_provider = super::TracerProvider::builder().build();
        assert_service_name(default_config_provider, Some("unknown_service"));

        // If user didn't provided a resource, try to get a default from env var
        let custom_config_provider = super::TracerProvider::builder()
            .with_config(Config {
                resource: Some(Arc::new(Resource::new(vec![KeyValue::new(
                    "service.name",
                    "test_service",
                )]))),
                ..Default::default()
            })
            .build();
        assert_service_name(custom_config_provider, Some("test_service"));

        // If `OTEL_RESOURCE_ATTRIBUTES` is set, read them automatically
        env::set_var("OTEL_RESOURCE_ATTRIBUTES", "key1=value1, k2, k3=value2");
        let env_resource_provider = super::TracerProvider::builder().build();
        assert_eq!(
            env_resource_provider.config().resource,
            Some(Arc::new(Resource::new(vec![
                KeyValue::new("key1", "value1"),
                KeyValue::new("k3", "value2"),
                KeyValue::new("service.name", "unknown_service"),
            ])))
        );

        // When `OTEL_RESOURCE_ATTRIBUTES` is set and also user provided config
        env::set_var(
            "OTEL_RESOURCE_ATTRIBUTES",
            "my-custom-key=env-val,k2=value2",
        );
        let user_provided_resource_config_provider = super::TracerProvider::builder()
            .with_config(Config {
                resource: Some(Arc::new(Resource::new(vec![KeyValue::new(
                    "my-custom-key",
                    "my-custom-value",
                )]))),
                ..Default::default()
            })
            .build();
        assert_eq!(
            user_provided_resource_config_provider.config().resource,
            Some(Arc::new(Resource::new(vec![
                KeyValue::new("my-custom-key", "my-custom-value"),
                KeyValue::new("k2", "value2"),
                KeyValue::new("service.name", "unknown_service"),
            ])))
        );
        env::remove_var("OTEL_RESOURCE_ATTRIBUTES");

        // If user provided a resource, it takes priority during collision.
        let no_service_name = super::TracerProvider::builder()
            .with_config(Config {
                resource: Some(Arc::new(Resource::empty())),
                ..Default::default()
            })
            .build();

        assert_service_name(no_service_name, None);
    }
}
