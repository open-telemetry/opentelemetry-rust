use core::fmt;
use std::{
    borrow::Cow,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use opentelemetry::{
    metrics::{noop::NoopMeterCore, InstrumentProvider, Meter as ApiMeter, MetricsError, Result},
    Context, KeyValue,
};

use crate::{instrumentation::Scope, Resource};

use super::{meter::Meter as SdkMeter, pipeline::Pipelines, reader::MetricReader, view::View};

/// Handles the creation and coordination of [Meter]s.
///
/// All `Meter`s created by a `MeterProvider` will be associated with the same
/// [Resource], have the same [View]s applied to them, and have their produced
/// metric telemetry passed to the configured [MetricReader]s.
///
/// [Meter]: crate::metrics::Meter
#[derive(Clone, Debug)]
pub struct MeterProvider {
    pipes: Arc<Pipelines>,
    is_shutdown: Arc<AtomicBool>,
}

impl Default for MeterProvider {
    fn default() -> Self {
        MeterProvider::builder().build()
    }
}

impl MeterProvider {
    /// Flushes all pending telemetry.
    ///
    /// There is no guaranteed that all telemetry be flushed or all resources have
    /// been released on error.
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
    /// use opentelemetry_sdk::metrics::MeterProvider;
    ///
    /// fn init_metrics() -> MeterProvider {
    ///     let provider = MeterProvider::default();
    ///
    ///     // Set provider to be used as global meter provider
    ///     let _ = global::set_meter_provider(provider.clone());
    ///
    ///     // Setup metric pipelines with readers + views
    ///
    ///     provider
    /// }
    ///
    /// fn main() {
    ///     let provider = init_metrics();
    ///     let cx = Context::new();
    ///
    ///     // create instruments + record measurements
    ///
    ///     // force all instruments to flush
    ///     provider.force_flush(&cx).unwrap();
    ///
    ///     // record more measurements..
    ///
    ///     // dropping provider and shutting down global provider ensure all
    ///     // remaining metrics data are exported
    ///     drop(provider);
    ///     global::shutdown_meter_provider();
    /// }
    /// ```
    pub fn force_flush(&self, cx: &Context) -> Result<()> {
        self.pipes.force_flush(cx)
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

impl opentelemetry::metrics::MeterProvider for MeterProvider {
    fn versioned_meter(
        &self,
        name: impl Into<Cow<'static, str>>,
        version: Option<impl Into<Cow<'static, str>>>,
        schema_url: Option<impl Into<Cow<'static, str>>>,
        attributes: Option<Vec<KeyValue>>,
    ) -> ApiMeter {
        let inst_provider: Arc<dyn InstrumentProvider + Send + Sync> =
            if !self.is_shutdown.load(Ordering::Relaxed) {
                let scope = Scope::new(name, version, schema_url, attributes);
                Arc::new(SdkMeter::new(scope, self.pipes.clone()))
            } else {
                Arc::new(NoopMeterCore::new())
            };

        ApiMeter::new(inst_provider)
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
    /// [Meter]: crate::metrics::Meter
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
    pub fn build(self) -> MeterProvider {
        MeterProvider {
            pipes: Arc::new(Pipelines::new(
                self.resource.unwrap_or_default(),
                self.readers,
                self.views,
            )),
            is_shutdown: Arc::new(AtomicBool::new(false)),
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
    use crate::testing::metrics::metric_reader::TestMetricReader;
    use crate::Resource;
    use opentelemetry::Key;
    use opentelemetry::KeyValue;
    use std::env;

    #[test]
    fn test_meter_provider_resource() {
        // If users didn't provide a resource and there isn't a env var set. Use default one.
        let assert_service_name = |provider: super::MeterProvider, expect: Option<&'static str>| {
            assert_eq!(
                provider.pipes.0[0]
                    .resource
                    .get(Key::from_static_str("service.name"))
                    .map(|v| v.to_string()),
                expect.map(|s| s.to_string())
            );
        };
        let reader = TestMetricReader {};
        let default_meter_provider = super::MeterProvider::builder().with_reader(reader).build();
        assert_service_name(default_meter_provider, Some("unknown_service"));

        // If user provided a resource, use that.
        let reader2 = TestMetricReader {};
        let custom_meter_provider = super::MeterProvider::builder()
            .with_reader(reader2)
            .with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                "test_service",
            )]))
            .build();
        assert_service_name(custom_meter_provider, Some("test_service"));

        // If `OTEL_RESOURCE_ATTRIBUTES` is set, read them automatically
        let reader3 = TestMetricReader {};
        env::set_var("OTEL_RESOURCE_ATTRIBUTES", "key1=value1, k2, k3=value2");
        let env_resource_provider = super::MeterProvider::builder().with_reader(reader3).build();
        assert_eq!(
            env_resource_provider.pipes.0[0].resource,
            Resource::new(vec![
                KeyValue::new("telemetry.sdk.name", "opentelemetry"),
                KeyValue::new("telemetry.sdk.version", env!("CARGO_PKG_VERSION")),
                KeyValue::new("telemetry.sdk.language", "rust"),
                KeyValue::new("key1", "value1"),
                KeyValue::new("k3", "value2"),
                KeyValue::new("service.name", "unknown_service"),
            ])
        );

        // When `OTEL_RESOURCE_ATTRIBUTES` is set and also user provided config
        env::set_var(
            "OTEL_RESOURCE_ATTRIBUTES",
            "my-custom-key=env-val,k2=value2",
        );
        let reader4 = TestMetricReader {};
        let user_provided_resource_config_provider = super::MeterProvider::builder()
            .with_reader(reader4)
            .with_resource(
                Resource::default().merge(&mut Resource::new(vec![KeyValue::new(
                    "my-custom-key",
                    "my-custom-value",
                )])),
            )
            .build();
        assert_eq!(
            user_provided_resource_config_provider.pipes.0[0].resource,
            Resource::new(vec![
                KeyValue::new("telemetry.sdk.name", "opentelemetry"),
                KeyValue::new("telemetry.sdk.version", env!("CARGO_PKG_VERSION")),
                KeyValue::new("telemetry.sdk.language", "rust"),
                KeyValue::new("my-custom-key", "my-custom-value"),
                KeyValue::new("k2", "value2"),
                KeyValue::new("service.name", "unknown_service"),
            ])
        );
        env::remove_var("OTEL_RESOURCE_ATTRIBUTES");

        // If user provided a resource, it takes priority during collision.
        let reader5 = TestMetricReader {};
        let no_service_name = super::MeterProvider::builder()
            .with_reader(reader5)
            .with_resource(Resource::empty())
            .build();

        assert_service_name(no_service_name, None);
    }
}
