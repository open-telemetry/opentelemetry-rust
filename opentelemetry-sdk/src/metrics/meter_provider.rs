use core::fmt;
use std::sync::Arc;

use opentelemetry_api::{
    metrics::{Meter as ApiMeter, Result},
    Context,
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
        self.pipes.shutdown()
    }
}

impl opentelemetry_api::metrics::MeterProvider for MeterProvider {
    fn versioned_meter(
        &self,
        name: &'static str,
        version: Option<&'static str>,
        schema_url: Option<&'static str>,
    ) -> ApiMeter {
        let scope = Scope::new(name, version, schema_url);
        ApiMeter::new(
            scope.clone(),
            Arc::new(SdkMeter::new(scope, self.pipes.clone())),
        )
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
