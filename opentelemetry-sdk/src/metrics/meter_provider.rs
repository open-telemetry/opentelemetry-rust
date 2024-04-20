use core::fmt;
use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use opentelemetry::{
    global,
    metrics::{noop::NoopMeterCore, Meter, MeterProvider, MetricsError, Result},
    KeyValue,
};

use crate::{instrumentation::Scope, Resource};

use super::{meter::SdkMeter, pipeline::Pipelines, reader::MetricReader, view::View};

/// Handles the creation and coordination of [Meter]s.
///
/// All `Meter`s created by a `MeterProvider` will be associated with the same
/// [Resource], have the same [View]s applied to them, and have their produced
/// metric telemetry passed to the configured [MetricReader]s.
///
/// [Meter]: opentelemetry::metrics::Meter
#[derive(Clone, Debug)]
pub struct SdkMeterProvider {
    inner: Arc<SdkMeterProviderInner>,
}

#[derive(Clone, Debug)]
struct SdkMeterProviderInner {
    pipes: Arc<Pipelines>,
    meters: Arc<Mutex<HashMap<Scope, Arc<SdkMeter>>>>,
    is_shutdown: Arc<AtomicBool>,
}

impl Default for SdkMeterProvider {
    fn default() -> Self {
        SdkMeterProvider::builder().build()
    }
}

impl SdkMeterProvider {
    /// Return default [MeterProviderBuilder]
    pub fn builder() -> MeterProviderBuilder {
        MeterProviderBuilder::default()
    }

    /// Flushes all pending telemetry.
    ///
    /// There is no guaranteed that all telemetry be flushed or all resources have
    /// been released on error.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::{global, Context};
    /// use opentelemetry_sdk::metrics::SdkMeterProvider;
    ///
    /// fn init_metrics() -> SdkMeterProvider {
    ///     // Setup metric pipelines with readers + views, default has no
    ///     // readers so nothing is exported.
    ///     let provider = SdkMeterProvider::default();
    ///
    ///     // Set provider to be used as global meter provider
    ///     let _ = global::set_meter_provider(provider.clone());
    ///
    ///     provider
    /// }
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let provider = init_metrics();
    ///
    ///     // create instruments + record measurements
    ///
    ///     // force all instruments to flush
    ///     provider.force_flush()?;
    ///
    ///     // record more measurements..
    ///
    ///     // shutdown ensures any cleanup required by the provider is done,
    ///     // and also invokes shutdown on the readers.
    ///     provider.shutdown()?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn force_flush(&self) -> Result<()> {
        self.inner.force_flush()
    }

    /// Shuts down the meter provider flushing all pending telemetry and releasing
    /// any held computational resources.
    ///
    /// This call is idempotent. The first call will perform all flush and releasing
    /// operations. Subsequent calls will perform no action and will return an error
    /// stating this.
    ///
    /// Measurements made by instruments from meters this MeterProvider created will
    /// not be exported after Shutdown is called.
    ///
    /// There is no guaranteed that all telemetry be flushed or all resources have
    /// been released on error.
    pub fn shutdown(&self) -> Result<()> {
        self.inner.shutdown()
    }
}

impl SdkMeterProviderInner {
    fn force_flush(&self) -> Result<()> {
        self.pipes.force_flush()
    }

    fn shutdown(&self) -> Result<()> {
        if self
            .is_shutdown
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            self.pipes.shutdown()
        } else {
            Err(MetricsError::Other(
                "metrics provider already shut down".into(),
            ))
        }
    }
}

impl Drop for SdkMeterProviderInner {
    fn drop(&mut self) {
        if let Err(err) = self.shutdown() {
            global::handle_error(err);
        }
    }
}
impl MeterProvider for SdkMeterProvider {
    fn versioned_meter(
        &self,
        name: impl Into<Cow<'static, str>>,
        version: Option<impl Into<Cow<'static, str>>>,
        schema_url: Option<impl Into<Cow<'static, str>>>,
        attributes: Option<Vec<KeyValue>>,
    ) -> Meter {
        if self.inner.is_shutdown.load(Ordering::Relaxed) {
            return Meter::new(Arc::new(NoopMeterCore::new()));
        }

        let scope = Scope::new(name, version, schema_url, attributes);

        if let Ok(mut meters) = self.inner.meters.lock() {
            let meter = meters
                .entry(scope)
                .or_insert_with_key(|scope| {
                    Arc::new(SdkMeter::new(scope.clone(), self.inner.pipes.clone()))
                })
                .clone();
            Meter::new(meter)
        } else {
            Meter::new(Arc::new(NoopMeterCore::new()))
        }
    }
}

/// Configuration options for a [MeterProvider].
#[derive(Default)]
pub struct MeterProviderBuilder {
    resource: Option<Resource>,
    readers: Vec<Box<dyn MetricReader>>,
    views: Vec<Arc<dyn View>>,
}

impl MeterProviderBuilder {
    /// Associates a [Resource] with a [MeterProvider].
    ///
    /// This [Resource] represents the entity producing telemetry and is associated
    /// with all [Meter]s the [MeterProvider] will create.
    ///
    /// By default, if this option is not used, the default [Resource] will be used.
    ///
    /// [Meter]: opentelemetry::metrics::Meter
    pub fn with_resource(mut self, resource: Resource) -> Self {
        self.resource = Some(resource);
        self
    }

    /// Associates a [MetricReader] with a [MeterProvider].
    ///
    /// By default, if this option is not used, the [MeterProvider] will perform no
    /// operations; no data will be exported without a [MetricReader].
    pub fn with_reader<T: MetricReader>(mut self, reader: T) -> Self {
        self.readers.push(Box::new(reader));
        self
    }

    /// Associates a [View] with a [MeterProvider].
    ///
    /// [View]s are appended to existing ones in a [MeterProvider] if this option is
    /// used multiple times.
    ///
    /// By default, if this option is not used, the [MeterProvider] will use the
    /// default view.
    pub fn with_view<T: View>(mut self, view: T) -> Self {
        self.views.push(Arc::new(view));
        self
    }

    /// Construct a new [MeterProvider] with this configuration.

    pub fn build(self) -> SdkMeterProvider {
        SdkMeterProvider {
            inner: Arc::new(SdkMeterProviderInner {
                pipes: Arc::new(Pipelines::new(
                    self.resource.unwrap_or_default(),
                    self.readers,
                    self.views,
                )),
                meters: Default::default(),
                is_shutdown: Arc::new(AtomicBool::new(false)),
            }),
        }
    }
}

impl fmt::Debug for MeterProviderBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MeterProviderBuilder")
            .field("resource", &self.resource)
            .field("readers", &self.readers)
            .field("views", &self.views.len())
            .finish()
    }
}
#[cfg(test)]
mod tests {
    use crate::resource::{
        SERVICE_NAME, TELEMETRY_SDK_LANGUAGE, TELEMETRY_SDK_NAME, TELEMETRY_SDK_VERSION,
    };
    use crate::testing::metrics::metric_reader::TestMetricReader;
    use crate::Resource;
    use opentelemetry::global;
    use opentelemetry::{Key, KeyValue, Value};
    use std::env;

    #[test]
    fn test_meter_provider_resource() {
        let assert_resource = |provider: &super::SdkMeterProvider,
                               resource_key: &'static str,
                               expect: Option<&'static str>| {
            assert_eq!(
                provider.inner.pipes.0[0]
                    .resource
                    .get(Key::from_static_str(resource_key))
                    .map(|v| v.to_string()),
                expect.map(|s| s.to_string())
            );
        };
        let assert_telemetry_resource = |provider: &super::SdkMeterProvider| {
            assert_eq!(
                provider.inner.pipes.0[0]
                    .resource
                    .get(TELEMETRY_SDK_LANGUAGE.into()),
                Some(Value::from("rust"))
            );
            assert_eq!(
                provider.inner.pipes.0[0]
                    .resource
                    .get(TELEMETRY_SDK_NAME.into()),
                Some(Value::from("opentelemetry"))
            );
            assert_eq!(
                provider.inner.pipes.0[0]
                    .resource
                    .get(TELEMETRY_SDK_VERSION.into()),
                Some(Value::from(env!("CARGO_PKG_VERSION")))
            );
        };

        // If users didn't provide a resource and there isn't a env var set. Use default one.
        temp_env::with_var_unset("OTEL_RESOURCE_ATTRIBUTES", || {
            let reader = TestMetricReader::new();
            let default_meter_provider = super::SdkMeterProvider::builder()
                .with_reader(reader)
                .build();
            assert_resource(
                &default_meter_provider,
                SERVICE_NAME,
                Some("unknown_service"),
            );
            assert_telemetry_resource(&default_meter_provider);
        });

        // If user provided a resource, use that.
        let reader2 = TestMetricReader::new();
        let custom_meter_provider = super::SdkMeterProvider::builder()
            .with_reader(reader2)
            .with_resource(Resource::new(vec![KeyValue::new(
                SERVICE_NAME,
                "test_service",
            )]))
            .build();
        assert_resource(&custom_meter_provider, SERVICE_NAME, Some("test_service"));
        assert_eq!(custom_meter_provider.inner.pipes.0[0].resource.len(), 1);

        temp_env::with_var(
            "OTEL_RESOURCE_ATTRIBUTES",
            Some("key1=value1, k2, k3=value2"),
            || {
                // If `OTEL_RESOURCE_ATTRIBUTES` is set, read them automatically
                let reader3 = TestMetricReader::new();
                let env_resource_provider = super::SdkMeterProvider::builder()
                    .with_reader(reader3)
                    .build();
                assert_resource(
                    &env_resource_provider,
                    SERVICE_NAME,
                    Some("unknown_service"),
                );
                assert_resource(&env_resource_provider, "key1", Some("value1"));
                assert_resource(&env_resource_provider, "k3", Some("value2"));
                assert_telemetry_resource(&env_resource_provider);
                assert_eq!(env_resource_provider.inner.pipes.0[0].resource.len(), 6);
            },
        );

        // When `OTEL_RESOURCE_ATTRIBUTES` is set and also user provided config
        temp_env::with_var(
            "OTEL_RESOURCE_ATTRIBUTES",
            Some("my-custom-key=env-val,k2=value2"),
            || {
                let reader4 = TestMetricReader::new();
                let user_provided_resource_config_provider = super::SdkMeterProvider::builder()
                    .with_reader(reader4)
                    .with_resource(Resource::default().merge(&mut Resource::new(vec![
                        KeyValue::new("my-custom-key", "my-custom-value"),
                        KeyValue::new("my-custom-key2", "my-custom-value2"),
                    ])))
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
                    user_provided_resource_config_provider.inner.pipes.0[0]
                        .resource
                        .len(),
                    7
                );
            },
        );

        // If user provided a resource, it takes priority during collision.
        let reader5 = TestMetricReader::new();
        let no_service_name = super::SdkMeterProvider::builder()
            .with_reader(reader5)
            .with_resource(Resource::empty())
            .build();

        assert_eq!(no_service_name.inner.pipes.0[0].resource.len(), 0)
    }

    #[test]
    fn test_meter_provider_shutdown() {
        let reader = TestMetricReader::new();
        let provider = super::SdkMeterProvider::builder()
            .with_reader(reader.clone())
            .build();
        global::set_meter_provider(provider);
        assert!(!reader.is_shutdown());
        // create a meter and an instrument
        let meter = global::meter("test");
        let counter = meter.u64_counter("test_counter").init();
        // no need to drop a meter for meter_provider shutdown
        global::shutdown_meter_provider();
        assert!(reader.is_shutdown());
        // TODO Fix: the instrument is still available, and can be used.
        // While the reader is shutdown, and no collect is happening
        counter.add(1, &[]);
    }
    #[test]
    fn test_shutdown_invoked_on_last_drop() {
        let reader = TestMetricReader::new();
        let provider = super::SdkMeterProvider::builder()
            .with_reader(reader.clone())
            .build();
        let clone1 = provider.clone();
        let clone2 = provider.clone();

        // Initially, shutdown should not be called
        assert!(!reader.is_shutdown());

        // Drop the first clone
        drop(clone1);
        assert!(!reader.is_shutdown());

        // Drop the second clone
        drop(clone2);
        assert!(!reader.is_shutdown());

        // Drop the last original provider
        drop(provider);
        // Now the shutdown should be invoked
        assert!(reader.is_shutdown());
    }
}
