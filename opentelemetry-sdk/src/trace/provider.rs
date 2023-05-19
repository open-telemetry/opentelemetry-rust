//! # Trace Provider SDK
//!
//! ## Tracer Creation
//!
//! New [`Tracer`] instances are always created through a [`TracerProvider`].
//!
//! All configuration objects and extension points (span processors,
//! propagators) are provided by the [`TracerProvider`]. [`Tracer`] instances do
//! not duplicate this data to avoid that different [`Tracer`] instances
//! of the [`TracerProvider`] have different versions of these data.
use crate::runtime::MessageRuntime;
use crate::trace::{BatchMessage, BatchSpanProcessor, SimpleSpanProcessor, Tracer};
use crate::{export::trace::SpanExporter, trace::SpanProcessor};
use crate::{InstrumentationLibrary, Resource};
use once_cell::sync::OnceCell;
use opentelemetry_api::{global, trace::TraceResult};
use std::borrow::Cow;
use std::sync::Arc;

/// Default tracer name if empty string is provided.
const DEFAULT_COMPONENT_NAME: &str = "rust.opentelemetry.io/sdk/tracer";
static PROVIDER_RESOURCE: OnceCell<Resource> = OnceCell::new();

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

/// Creator and registry of named [`Tracer`] instances.
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

    /// Create a new [`TracerProvider`] builder.
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

    /// Force flush all remaining spans in span processors and return results.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry_api::global;
    /// use opentelemetry_sdk::trace::TracerProvider;
    ///
    /// fn init_tracing() -> TracerProvider {
    ///     let provider = TracerProvider::default();
    ///
    ///     // Set provider to be used as global tracer provider
    ///     let _ = global::set_tracer_provider(provider.clone());
    ///
    ///     provider
    /// }
    ///
    /// fn main() {
    ///     let provider = init_tracing();
    ///
    ///     // create spans..
    ///
    ///     // force all spans to flush
    ///     for result in provider.force_flush() {
    ///         if let Err(err) = result {
    ///             // .. handle flush error
    ///         }
    ///     }
    ///
    ///     // create more spans..
    ///
    ///     // dropping provider and shutting down global provider ensure all
    ///     // remaining spans are exported
    ///     drop(provider);
    ///     global::shutdown_tracer_provider();
    /// }
    /// ```
    pub fn force_flush(&self) -> Vec<TraceResult<()>> {
        self.span_processors()
            .iter()
            .map(|processor| processor.force_flush())
            .collect()
    }
}

impl opentelemetry_api::trace::TracerProvider for TracerProvider {
    /// This implementation of `TracerProvider` produces `Tracer` instances.
    type Tracer = crate::trace::Tracer;

    /// Create a new versioned `Tracer` instance.
    fn versioned_tracer(
        &self,
        name: impl Into<Cow<'static, str>>,
        version: Option<impl Into<Cow<'static, str>>>,
        schema_url: Option<impl Into<Cow<'static, str>>>,
        attributes: Option<Vec<opentelemetry_api::KeyValue>>,
    ) -> Self::Tracer {
        // Use default value if name is invalid empty string
        let name = name.into();
        let component_name = if name.is_empty() {
            Cow::Borrowed(DEFAULT_COMPONENT_NAME)
        } else {
            name
        };
        let instrumentation_lib =
            InstrumentationLibrary::new(component_name, version, schema_url, attributes);

        Tracer::new(instrumentation_lib, Arc::downgrade(&self.inner))
    }
}

/// Builder for provider attributes.
#[derive(Debug, Default)]
pub struct Builder {
    processors: Vec<Box<dyn SpanProcessor>>,
    config: crate::trace::Config,
}

impl Builder {
    /// The `SpanExporter` that this provider should use.
    pub fn with_simple_exporter<T: SpanExporter + 'static>(self, exporter: T) -> Self {
        let mut processors = self.processors;
        processors.push(Box::new(SimpleSpanProcessor::new(Box::new(exporter))));

        Builder { processors, ..self }
    }

    /// The [`SpanExporter`] setup using a default [`BatchSpanProcessor`] that this provider should use.
    pub fn with_batch_exporter<T: SpanExporter + 'static, R: MessageRuntime<BatchMessage>>(
        self,
        exporter: T,
        runtime: R,
    ) -> Self {
        let batch = BatchSpanProcessor::builder(exporter, runtime).build();
        self.with_span_processor(batch)
    }

    /// The [`SpanProcessor`] that this provider should use.
    pub fn with_span_processor<T: SpanProcessor + 'static>(self, processor: T) -> Self {
        let mut processors = self.processors;
        processors.push(Box::new(processor));

        Builder { processors, ..self }
    }

    /// The sdk [`crate::trace::Config`] that this provider will use.
    pub fn with_config(self, config: crate::trace::Config) -> Self {
        Builder { config, ..self }
    }

    /// Create a new provider from this configuration.
    pub fn build(self) -> TracerProvider {
        let mut config = self.config;

        // Standard config will contain an owned [`Resource`] (either sdk default or use supplied)
        // we can optimize the common case with a static ref to avoid cloning the underlying
        // resource data for each span.
        //
        // For the uncommon case where there are multiple tracer providers with different resource
        // configurations, users can optionally provide their own borrowed static resource.
        if matches!(config.resource, Cow::Owned(_)) {
            config.resource = match PROVIDER_RESOURCE.try_insert(config.resource.into_owned()) {
                Ok(static_resource) => Cow::Borrowed(static_resource),
                Err((prev, new)) => {
                    if prev == &new {
                        Cow::Borrowed(prev)
                    } else {
                        Cow::Owned(new)
                    }
                }
            }
        }

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
    use opentelemetry_api::trace::{TraceError, TraceResult};
    use opentelemetry_api::{Context, Key, KeyValue};
    use std::borrow::Cow;
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
                provider
                    .config()
                    .resource
                    .get(Key::from_static_str("service.name"))
                    .map(|v| v.to_string()),
                expect.map(|s| s.to_string())
            );
        };
        let default_config_provider = super::TracerProvider::builder().build();
        assert_service_name(default_config_provider, Some("unknown_service"));

        // If user didn't provided a resource, try to get a default from env var
        let custom_config_provider = super::TracerProvider::builder()
            .with_config(Config {
                resource: Cow::Owned(Resource::new(vec![KeyValue::new(
                    "service.name",
                    "test_service",
                )])),
                ..Default::default()
            })
            .build();
        assert_service_name(custom_config_provider, Some("test_service"));

        // If `OTEL_RESOURCE_ATTRIBUTES` is set, read them automatically
        env::set_var("OTEL_RESOURCE_ATTRIBUTES", "key1=value1, k2, k3=value2");
        let env_resource_provider = super::TracerProvider::builder().build();
        assert_eq!(
            env_resource_provider.config().resource,
            Cow::Owned(Resource::new(vec![
                KeyValue::new("key1", "value1"),
                KeyValue::new("k3", "value2"),
                KeyValue::new("service.name", "unknown_service"),
            ]))
        );

        // When `OTEL_RESOURCE_ATTRIBUTES` is set and also user provided config
        env::set_var(
            "OTEL_RESOURCE_ATTRIBUTES",
            "my-custom-key=env-val,k2=value2",
        );
        let user_provided_resource_config_provider = super::TracerProvider::builder()
            .with_config(Config {
                resource: Cow::Owned(Resource::default().merge(&mut Resource::new(vec![
                    KeyValue::new("my-custom-key", "my-custom-value"),
                ]))),
                ..Default::default()
            })
            .build();
        assert_eq!(
            user_provided_resource_config_provider.config().resource,
            Cow::Owned(Resource::new(vec![
                KeyValue::new("my-custom-key", "my-custom-value"),
                KeyValue::new("k2", "value2"),
                KeyValue::new("service.name", "unknown_service"),
            ]))
        );
        env::remove_var("OTEL_RESOURCE_ATTRIBUTES");

        // If user provided a resource, it takes priority during collision.
        let no_service_name = super::TracerProvider::builder()
            .with_config(Config {
                resource: Cow::Owned(Resource::empty()),
                ..Default::default()
            })
            .build();

        assert_service_name(no_service_name, None);
    }
}
