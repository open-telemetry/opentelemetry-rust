/// # Trace Provider SDK
///
/// The `TracerProvider` handles the creation and management of [`Tracer`] instances and coordinates
/// span processing. It serves as the central configuration point for tracing, ensuring consistency
/// across all [`Tracer`] instances it creates.
///
/// ## Tracer Creation
///
/// New [`Tracer`] instances are always created through a `TracerProvider`. These `Tracer`s share
/// a common configuration, which includes the [`Resource`], span processors, sampling strategies,
/// and span limits. This avoids the need for each `Tracer` to maintain its own version of these
/// configurations, ensuring uniform behavior across all instances.
///
/// ## Cloning and Shutdown
///
/// The `TracerProvider` is designed to be clonable. Cloning a `TracerProvider`  creates a
/// new reference to the same provider, not a new instance. Dropping the last reference
/// to the `TracerProvider` will automatically trigger its shutdown. During shutdown, the provider
/// will flush all remaining spans, ensuring they are passed to the configured processors.
/// Users can also manually trigger shutdown using the [`shutdown`](TracerProvider::shutdown)
/// method, which will ensure the same behavior.
///
/// Once shut down, the `TracerProvider` transitions into a disabled state. In this state, further
/// operations on its associated `Tracer` instances will result in no-ops, ensuring that no spans
/// are processed or exported after shutdown.
///
/// ## Span Processing and Force Flush
///
/// The `TracerProvider` manages the lifecycle of span processors, which are responsible for
/// collecting, processing, and exporting spans. The [`force_flush`](TracerProvider::force_flush) method
/// invoked at any time will trigger an immediate flush of all pending spans (if any) to the exporters.
/// This will block the user thread till all the spans are passed to exporters.
///
/// # Examples
///
/// ```
/// use opentelemetry::global;
/// use opentelemetry_sdk::trace::TracerProvider;
/// use opentelemetry::trace::Tracer;
///
/// fn init_tracing() -> TracerProvider {
///     let provider = TracerProvider::default();
///
///     // Set the provider to be used globally
///     let _ = global::set_tracer_provider(provider.clone());
///
///     provider
/// }
///
/// fn main() {
///     let provider = init_tracing();
///
///     // create tracer..
///     let tracer = global::tracer("example/client");
///
///     // create span...
///     let span = tracer
///         .span_builder("test_span")
///         .start(&tracer);
///
///     // Explicitly shut down the provider
///     provider.shutdown();
/// }
/// ```
use crate::runtime::RuntimeChannel;
use crate::trace::{
    BatchSpanProcessor, Config, RandomIdGenerator, Sampler, SimpleSpanProcessor, SpanLimits, Tracer,
};
use crate::{export::trace::SpanExporter, trace::SpanProcessor};
use crate::{InstrumentationLibrary, Resource};
use once_cell::sync::{Lazy, OnceCell};
use opentelemetry::trace::TraceError;
use opentelemetry::{otel_debug, trace::TraceResult};
use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Default tracer name if empty string is provided.
const DEFAULT_COMPONENT_NAME: &str = "rust.opentelemetry.io/sdk/tracer";
static PROVIDER_RESOURCE: OnceCell<Resource> = OnceCell::new();

// a no nop tracer provider used as placeholder when the provider is shutdown
static NOOP_TRACER_PROVIDER: Lazy<TracerProvider> = Lazy::new(|| TracerProvider {
    inner: Arc::new(TracerProviderInner {
        processors: Vec::new(),
        config: Config {
            // cannot use default here as the default resource is not empty
            sampler: Box::new(Sampler::ParentBased(Box::new(Sampler::AlwaysOn))),
            id_generator: Box::<RandomIdGenerator>::default(),
            span_limits: SpanLimits::default(),
            resource: Cow::Owned(Resource::empty()),
        },
        is_shutdown: AtomicBool::new(true),
    }),
});

/// TracerProvider inner type
#[derive(Debug)]
pub(crate) struct TracerProviderInner {
    processors: Vec<Box<dyn SpanProcessor>>,
    config: crate::trace::Config,
    is_shutdown: AtomicBool,
}

impl TracerProviderInner {
    /// Crate-private shutdown method to be called both from explicit shutdown
    /// and from Drop when the last reference is released.
    pub(crate) fn shutdown(&self) -> Vec<TraceError> {
        let mut errs = vec![];
        for processor in &self.processors {
            if let Err(err) = processor.shutdown() {
                // Log at debug level because:
                //  - The error is also returned to the user for handling (if applicable)
                //  - Or the error occurs during `TracerProviderInner::Drop` as part of telemetry shutdown,
                //    which is non-actionable by the user
                otel_debug!(name: "TracerProvider.Drop.ShutdownError",
                        error = format!("{err}"));
                errs.push(err);
            }
        }
        errs
    }
}

impl Drop for TracerProviderInner {
    fn drop(&mut self) {
        if !self.is_shutdown.load(Ordering::Relaxed) {
            let _ = self.shutdown(); // errors are handled within shutdown
        } else {
            otel_debug!(
                name: "TracerProvider.Drop.AlreadyShutdown"
            );
        }
    }
}

/// Creator and registry of named [`Tracer`] instances.
///
/// `TracerProvider` is a container holding pointers to `SpanProcessor` and other components.
/// Cloning a `TracerProvider` instance and dropping it will not stop span processing. To stop span processing, users
/// must either call the `shutdown` method explicitly or allow the last reference to the `TracerProvider`
/// to be dropped. When the last reference is dropped, the shutdown process will be automatically triggered
/// to ensure proper cleanup.
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
    pub(crate) fn new(inner: TracerProviderInner) -> Self {
        TracerProvider {
            inner: Arc::new(inner),
        }
    }

    /// Create a new [`TracerProvider`] builder.
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Span processors associated with this provider
    pub(crate) fn span_processors(&self) -> &[Box<dyn SpanProcessor>] {
        &self.inner.processors
    }

    /// Config associated with this tracer
    pub(crate) fn config(&self) -> &crate::trace::Config {
        &self.inner.config
    }

    /// true if the provider has been shutdown
    /// Don't start span or export spans when provider is shutdown
    pub(crate) fn is_shutdown(&self) -> bool {
        self.inner.is_shutdown.load(Ordering::Relaxed)
    }

    /// Force flush all remaining spans in span processors and return results.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::global;
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

    /// Shuts down the current `TracerProvider`.
    ///
    /// Note that shut down doesn't means the TracerProvider has dropped
    pub fn shutdown(&self) -> TraceResult<()> {
        if self
            .inner
            .is_shutdown
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            // propagate the shutdown signal to processors
            let errs = self.inner.shutdown();
            if errs.is_empty() {
                Ok(())
            } else {
                Err(TraceError::Other(format!("{errs:?}").into()))
            }
        } else {
            Err(TraceError::AlreadyShutdown)
        }
    }
}

impl opentelemetry::trace::TracerProvider for TracerProvider {
    /// This implementation of `TracerProvider` produces `Tracer` instances.
    type Tracer = Tracer;

    /// Create a new versioned `Tracer` instance.
    fn versioned_tracer(
        &self,
        name: impl Into<Cow<'static, str>>,
        version: Option<impl Into<Cow<'static, str>>>,
        schema_url: Option<impl Into<Cow<'static, str>>>,
        attributes: Option<Vec<opentelemetry::KeyValue>>,
    ) -> Self::Tracer {
        // Use default value if name is invalid empty string
        let name = name.into();
        let component_name = if name.is_empty() {
            Cow::Borrowed(DEFAULT_COMPONENT_NAME)
        } else {
            name
        };

        let mut builder = self.tracer_builder(component_name);

        if let Some(v) = version {
            builder = builder.with_version(v);
        }
        if let Some(s) = schema_url {
            builder = builder.with_schema_url(s);
        }
        if let Some(a) = attributes {
            builder = builder.with_attributes(a);
        }

        builder.build()
    }

    fn library_tracer(&self, library: Arc<InstrumentationLibrary>) -> Self::Tracer {
        if self.inner.is_shutdown.load(Ordering::Relaxed) {
            return Tracer::new(library, NOOP_TRACER_PROVIDER.clone());
        }
        Tracer::new(library, self.clone())
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
    pub fn with_batch_exporter<T: SpanExporter + 'static, R: RuntimeChannel>(
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

        // Create a new vector to hold the modified processors
        let mut processors = self.processors;

        // Set the resource for each processor
        for p in &mut processors {
            p.set_resource(config.resource.as_ref());
        }

        let is_shutdown = AtomicBool::new(false);
        TracerProvider::new(TracerProviderInner {
            processors,
            config,
            is_shutdown,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::export::trace::SpanData;
    use crate::resource::{
        SERVICE_NAME, TELEMETRY_SDK_LANGUAGE, TELEMETRY_SDK_NAME, TELEMETRY_SDK_VERSION,
    };
    use crate::trace::provider::TracerProviderInner;
    use crate::trace::{Config, Span, SpanProcessor};
    use crate::Resource;
    use opentelemetry::trace::{TraceError, TraceResult, Tracer, TracerProvider};
    use opentelemetry::{Context, Key, KeyValue, Value};
    use std::borrow::Cow;
    use std::env;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::Arc;

    // fields below is wrapped with Arc so we can assert it
    #[derive(Default, Debug)]
    struct AssertInfo {
        started_span: AtomicU32,
        is_shutdown: AtomicBool,
    }

    #[derive(Default, Debug, Clone)]
    struct SharedAssertInfo(Arc<AssertInfo>);

    impl SharedAssertInfo {
        fn started_span_count(&self, count: u32) -> bool {
            self.0.started_span.load(Ordering::SeqCst) == count
        }
    }

    #[derive(Debug)]
    struct TestSpanProcessor {
        success: bool,
        assert_info: SharedAssertInfo,
    }

    impl TestSpanProcessor {
        fn new(success: bool) -> TestSpanProcessor {
            TestSpanProcessor {
                success,
                assert_info: SharedAssertInfo::default(),
            }
        }

        // get handle to assert info
        fn assert_info(&self) -> SharedAssertInfo {
            self.assert_info.clone()
        }
    }

    impl SpanProcessor for TestSpanProcessor {
        fn on_start(&self, _span: &mut Span, _cx: &Context) {
            self.assert_info
                .0
                .started_span
                .fetch_add(1, Ordering::SeqCst);
        }

        fn on_end(&self, _span: SpanData) {
            // ignore
        }

        fn force_flush(&self) -> TraceResult<()> {
            if self.success {
                Ok(())
            } else {
                Err(TraceError::from("cannot export"))
            }
        }

        fn shutdown(&self) -> TraceResult<()> {
            if self.assert_info.0.is_shutdown.load(Ordering::SeqCst) {
                Ok(())
            } else {
                let _ = self.assert_info.0.is_shutdown.compare_exchange(
                    false,
                    true,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                );
                self.force_flush()
            }
        }
    }

    #[test]
    fn test_force_flush() {
        let tracer_provider = super::TracerProvider::new(TracerProviderInner {
            processors: vec![
                Box::from(TestSpanProcessor::new(true)),
                Box::from(TestSpanProcessor::new(false)),
            ],
            config: Default::default(),
            is_shutdown: AtomicBool::new(false),
        });

        let results = tracer_provider.force_flush();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_tracer_provider_default_resource() {
        let assert_resource = |provider: &super::TracerProvider,
                               resource_key: &'static str,
                               expect: Option<&'static str>| {
            assert_eq!(
                provider
                    .config()
                    .resource
                    .get(Key::from_static_str(resource_key))
                    .map(|v| v.to_string()),
                expect.map(|s| s.to_string())
            );
        };
        let assert_telemetry_resource = |provider: &super::TracerProvider| {
            assert_eq!(
                provider
                    .config()
                    .resource
                    .get(TELEMETRY_SDK_LANGUAGE.into()),
                Some(Value::from("rust"))
            );
            assert_eq!(
                provider.config().resource.get(TELEMETRY_SDK_NAME.into()),
                Some(Value::from("opentelemetry"))
            );
            assert_eq!(
                provider.config().resource.get(TELEMETRY_SDK_VERSION.into()),
                Some(Value::from(env!("CARGO_PKG_VERSION")))
            );
        };

        // If users didn't provide a resource and there isn't a env var set. Use default one.
        temp_env::with_var_unset("OTEL_RESOURCE_ATTRIBUTES", || {
            let default_config_provider = super::TracerProvider::builder().build();
            assert_resource(
                &default_config_provider,
                SERVICE_NAME,
                Some("unknown_service"),
            );
            assert_telemetry_resource(&default_config_provider);
        });

        // If user provided a resource, use that.
        let custom_config_provider = super::TracerProvider::builder()
            .with_config(Config {
                resource: Cow::Owned(Resource::new(vec![KeyValue::new(
                    SERVICE_NAME,
                    "test_service",
                )])),
                ..Default::default()
            })
            .build();
        assert_resource(&custom_config_provider, SERVICE_NAME, Some("test_service"));
        assert_eq!(custom_config_provider.config().resource.len(), 1);

        // If `OTEL_RESOURCE_ATTRIBUTES` is set, read them automatically
        temp_env::with_var(
            "OTEL_RESOURCE_ATTRIBUTES",
            Some("key1=value1, k2, k3=value2"),
            || {
                let env_resource_provider = super::TracerProvider::builder().build();
                assert_resource(
                    &env_resource_provider,
                    SERVICE_NAME,
                    Some("unknown_service"),
                );
                assert_resource(&env_resource_provider, "key1", Some("value1"));
                assert_resource(&env_resource_provider, "k3", Some("value2"));
                assert_telemetry_resource(&env_resource_provider);
                assert_eq!(env_resource_provider.config().resource.len(), 6);
            },
        );

        // When `OTEL_RESOURCE_ATTRIBUTES` is set and also user provided config
        temp_env::with_var(
            "OTEL_RESOURCE_ATTRIBUTES",
            Some("my-custom-key=env-val,k2=value2"),
            || {
                let user_provided_resource_config_provider = super::TracerProvider::builder()
                    .with_config(Config {
                        resource: Cow::Owned(Resource::default().merge(&mut Resource::new(vec![
                            KeyValue::new("my-custom-key", "my-custom-value"),
                            KeyValue::new("my-custom-key2", "my-custom-value2"),
                        ]))),
                        ..Default::default()
                    })
                    .build();
                assert_resource(
                    &user_provided_resource_config_provider,
                    SERVICE_NAME,
                    Some("unknown_service"),
                );
                assert_resource(
                    &user_provided_resource_config_provider,
                    "my-custom-key",
                    Some("my-custom-value"),
                );
                assert_resource(
                    &user_provided_resource_config_provider,
                    "my-custom-key2",
                    Some("my-custom-value2"),
                );
                assert_resource(
                    &user_provided_resource_config_provider,
                    "k2",
                    Some("value2"),
                );
                assert_telemetry_resource(&user_provided_resource_config_provider);
                assert_eq!(
                    user_provided_resource_config_provider
                        .config()
                        .resource
                        .len(),
                    7
                );
            },
        );

        // If user provided a resource, it takes priority during collision.
        let no_service_name = super::TracerProvider::builder()
            .with_config(Config {
                resource: Cow::Owned(Resource::empty()),
                ..Default::default()
            })
            .build();

        assert_eq!(no_service_name.config().resource.len(), 0)
    }

    #[test]
    fn test_shutdown_noops() {
        let processor = TestSpanProcessor::new(false);
        let assert_handle = processor.assert_info();
        let tracer_provider = super::TracerProvider::new(TracerProviderInner {
            processors: vec![Box::from(processor)],
            config: Default::default(),
            is_shutdown: AtomicBool::new(false),
        });

        let test_tracer_1 = tracer_provider.tracer("test1");
        let _ = test_tracer_1.start("test");

        assert!(assert_handle.started_span_count(1));

        let _ = test_tracer_1.start("test");

        assert!(assert_handle.started_span_count(2));

        let shutdown = |tracer_provider: super::TracerProvider| {
            let _ = tracer_provider.shutdown(); // shutdown once
        };

        // assert tracer provider can be shutdown using on a cloned version
        shutdown(tracer_provider.clone());

        // after shutdown we should get noop tracer
        let noop_tracer = tracer_provider.tracer("noop");

        // noop tracer cannot start anything
        let _ = noop_tracer.start("test");
        assert!(assert_handle.started_span_count(2));
        // noop tracer's tracer provider should be shutdown
        assert!(noop_tracer.provider().is_shutdown());

        // existing tracer becomes noops after shutdown
        let _ = test_tracer_1.start("test");
        assert!(assert_handle.started_span_count(2));
    }

    #[derive(Debug)]
    struct CountingShutdownProcessor {
        shutdown_count: Arc<AtomicU32>,
    }

    impl CountingShutdownProcessor {
        fn new(shutdown_count: Arc<AtomicU32>) -> Self {
            CountingShutdownProcessor { shutdown_count }
        }
    }

    impl SpanProcessor for CountingShutdownProcessor {
        fn on_start(&self, _span: &mut Span, _cx: &Context) {
            // No operation needed for this processor
        }

        fn on_end(&self, _span: SpanData) {
            // No operation needed for this processor
        }

        fn force_flush(&self) -> TraceResult<()> {
            Ok(())
        }

        fn shutdown(&self) -> TraceResult<()> {
            self.shutdown_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn drop_test_with_multiple_providers() {
        let shutdown_count = Arc::new(AtomicU32::new(0));
        let flush_called = Arc::new(AtomicBool::new(false));

        {
            // Create a shared TracerProviderInner and use it across multiple providers
            let shared_inner = Arc::new(TracerProviderInner {
                processors: vec![Box::new(CountingShutdownProcessor::new(
                    shutdown_count.clone(),
                ))],
                config: Config::default(),
                is_shutdown: AtomicBool::new(false),
            });

            {
                let tracer_provider1 = super::TracerProvider {
                    inner: shared_inner.clone(),
                };
                let tracer_provider2 = super::TracerProvider {
                    inner: shared_inner.clone(),
                };

                let tracer1 = tracer_provider1.tracer("test-tracer1");
                let tracer2 = tracer_provider2.tracer("test-tracer2");

                let _span1 = tracer1.start("span1");
                let _span2 = tracer2.start("span2");

                // TracerProviderInner should not be dropped yet, since both providers and `shared_inner`
                // are still holding a reference.
            }
            // At this point, both `tracer_provider1` and `tracer_provider2` are dropped,
            // but `shared_inner` still holds a reference, so `TracerProviderInner` is NOT dropped yet.
            assert_eq!(shutdown_count.load(Ordering::SeqCst), 0);
        }
        // Verify shutdown was called during the drop of the shared TracerProviderInner
        assert_eq!(shutdown_count.load(Ordering::SeqCst), 1);
        // Verify flush was not called during drop
        assert!(!flush_called.load(Ordering::SeqCst));
    }

    #[test]
    fn drop_after_shutdown_test_with_multiple_providers() {
        let shutdown_count = Arc::new(AtomicU32::new(0));

        // Create a shared TracerProviderInner and use it across multiple providers
        let shared_inner = Arc::new(TracerProviderInner {
            processors: vec![Box::new(CountingShutdownProcessor::new(
                shutdown_count.clone(),
            ))],
            config: Config::default(),
            is_shutdown: AtomicBool::new(false),
        });

        // Create a scope to test behavior when providers are dropped
        {
            let tracer_provider1 = super::TracerProvider {
                inner: shared_inner.clone(),
            };
            let tracer_provider2 = super::TracerProvider {
                inner: shared_inner.clone(),
            };

            // Explicitly shut down the tracer provider
            let shutdown_result = tracer_provider1.shutdown();
            assert!(shutdown_result.is_ok());

            // Verify that shutdown was called exactly once
            assert_eq!(shutdown_count.load(Ordering::SeqCst), 1);

            // TracerProvider2 should observe the shutdown state but not trigger another shutdown
            let shutdown_result2 = tracer_provider2.shutdown();
            assert!(shutdown_result2.is_err());
            assert_eq!(shutdown_count.load(Ordering::SeqCst), 1);

            // Both tracer providers will be dropped at the end of this scope
        }

        // Verify that shutdown was only called once, even after drop
        assert_eq!(shutdown_count.load(Ordering::SeqCst), 1);
    }
}
