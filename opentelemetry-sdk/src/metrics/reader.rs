//! Interfaces for reading and producing metrics
use std::{fmt, sync::Weak};

use opentelemetry::metrics::Result;

use super::{
    data::{ResourceMetrics, Temporality},
    instrument::InstrumentKind,
    pipeline::Pipeline,
};

/// The interface used between the SDK and an exporter.
///
/// Control flow is bi-directional through the `MetricReader`, since the SDK
/// initiates `force_flush` and `shutdown` while the reader initiates
/// collection. The `register_pipeline` method here informs the metric reader
/// that it can begin reading, signaling the start of bi-directional control
/// flow.
///
/// Typically, push-based exporters that are periodic will implement
/// `MetricExporter` themselves and construct a `PeriodicReader` to satisfy this
/// interface.
///
/// Pull-based exporters will typically implement `MetricReader` themselves,
/// since they read on demand.
pub trait MetricReader: TemporalitySelector + fmt::Debug + Send + Sync + 'static {
    /// Registers a [MetricReader] with a [Pipeline].
    ///
    /// The pipeline argument allows the `MetricReader` to signal the sdk to collect
    /// and send aggregated metric measurements.
    fn register_pipeline(&self, pipeline: Weak<Pipeline>);

    /// Gathers and returns all metric data related to the [MetricReader] from the
    /// SDK and stores it in the provided [ResourceMetrics] reference.
    ///
    /// An error is returned if this is called after shutdown.
    fn collect(&self, rm: &mut ResourceMetrics) -> Result<()>;

    /// Flushes all metric measurements held in an export pipeline.
    ///
    /// There is no guaranteed that all telemetry be flushed or all resources have
    /// been released on error.
    fn force_flush(&self) -> Result<()>;

    /// Flushes all metric measurements held in an export pipeline and releases any
    /// held computational resources.
    ///
    /// There is no guaranteed that all telemetry be flushed or all resources have
    /// been released on error.
    ///
    /// After `shutdown` is called, calls to `collect` will perform no operation and
    /// instead will return an error indicating the shutdown state.
    fn shutdown(&self) -> Result<()>;
}

/// Produces metrics for a [MetricReader].
pub(crate) trait SdkProducer: fmt::Debug + Send + Sync {
    /// Returns aggregated metrics from a single collection.
    fn produce(&self, rm: &mut ResourceMetrics) -> Result<()>;
}

/// An interface for selecting the temporality for an [InstrumentKind].
pub trait TemporalitySelector: Send + Sync {
    /// Selects the temporality to use based on the [InstrumentKind].
    fn temporality(&self, kind: InstrumentKind) -> Temporality;
}

/// The default temporality used if not specified for a given [InstrumentKind].
///
/// [Temporality::Cumulative] will be used for all instrument kinds if this
/// [TemporalitySelector] is used.
#[derive(Clone, Default, Debug)]
pub struct DefaultTemporalitySelector {
    pub(crate) _private: (),
}

impl DefaultTemporalitySelector {
    /// Create a new default temporality selector.
    pub fn new() -> Self {
        Self::default()
    }
}

impl TemporalitySelector for DefaultTemporalitySelector {
    fn temporality(&self, _kind: InstrumentKind) -> Temporality {
        Temporality::Cumulative
    }
}

/// A temporality selector that returns [`Delta`][Temporality::Delta] for all
/// instruments except `UpDownCounter` and `ObservableUpDownCounter`.
///
/// This temporality selector is equivalent to OTLP Metrics Exporter's
/// `Delta` temporality preference (see [its documentation][exporter-docs]).
///
/// [exporter-docs]: https://github.com/open-telemetry/opentelemetry-specification/blob/a1c13d59bb7d0fb086df2b3e1eaec9df9efef6cc/specification/metrics/sdk_exporters/otlp.md#additional-configuration
#[derive(Clone, Default, Debug)]
pub struct DeltaTemporalitySelector {
    pub(crate) _private: (),
}

impl DeltaTemporalitySelector {
    /// Create a new default temporality selector.
    pub fn new() -> Self {
        Self::default()
    }
}

impl TemporalitySelector for DeltaTemporalitySelector {
    #[rustfmt::skip]
    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        match kind {
            InstrumentKind::Counter
            | InstrumentKind::Histogram
            | InstrumentKind::ObservableCounter
            | InstrumentKind::Gauge
            | InstrumentKind::ObservableGauge => {
                Temporality::Delta
            }
            InstrumentKind::UpDownCounter
            | InstrumentKind::ObservableUpDownCounter => {
                Temporality::Cumulative
            }
        }
    }
}
