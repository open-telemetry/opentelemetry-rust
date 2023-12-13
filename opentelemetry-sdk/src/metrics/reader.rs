//! Interfaces for reading and producing metrics
use std::{fmt, sync::Weak};

use opentelemetry::metrics::Result;

use super::{
    aggregation::Aggregation,
    data::{ResourceMetrics, ScopeMetrics, Temporality},
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
pub trait MetricReader:
    AggregationSelector + TemporalitySelector + fmt::Debug + Send + Sync + 'static
{
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

/// Produces metrics for a [MetricReader] from an external source.
pub trait MetricProducer: fmt::Debug + Send + Sync {
    /// Returns aggregated metrics from an external source.
    fn produce(&self) -> Result<ScopeMetrics>;
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

/// An interface for selecting the aggregation and the parameters for an
/// [InstrumentKind].
pub trait AggregationSelector: Send + Sync {
    /// Selects the aggregation and the parameters to use for that aggregation based on
    /// the [InstrumentKind].
    fn aggregation(&self, kind: InstrumentKind) -> Aggregation;
}

impl<T> AggregationSelector for T
where
    T: Fn(InstrumentKind) -> Aggregation + Send + Sync,
{
    fn aggregation(&self, kind: InstrumentKind) -> Aggregation {
        self(kind)
    }
}

/// The default aggregation and parameters for an instrument of [InstrumentKind].
///
/// This [AggregationSelector] uses the following selection mapping per [the spec]:
///
/// * Counter ⇨ Sum
/// * Observable Counter ⇨ Sum
/// * UpDownCounter ⇨ Sum
/// * Observable UpDownCounter ⇨ Sum
/// * Gauge ⇨ LastValue
/// * Observable Gauge ⇨ LastValue
/// * Histogram ⇨ ExplicitBucketHistogram
///
/// [the spec]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.19.0/specification/metrics/sdk.md#default-aggregation
#[derive(Clone, Default, Debug)]
pub struct DefaultAggregationSelector {
    pub(crate) _private: (),
}

impl DefaultAggregationSelector {
    /// Create a new default aggregation selector.
    pub fn new() -> Self {
        Self::default()
    }
}

impl AggregationSelector for DefaultAggregationSelector {
    fn aggregation(&self, kind: InstrumentKind) -> Aggregation {
        match kind {
            InstrumentKind::Counter
            | InstrumentKind::UpDownCounter
            | InstrumentKind::ObservableCounter
            | InstrumentKind::ObservableUpDownCounter => Aggregation::Sum,
            InstrumentKind::Gauge => Aggregation::LastValue,
            InstrumentKind::ObservableGauge => Aggregation::LastValue,
            InstrumentKind::Histogram => Aggregation::ExplicitBucketHistogram {
                boundaries: vec![
                    0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0, 2500.0,
                    5000.0, 7500.0, 10000.0,
                ],
                record_min_max: true,
            },
        }
    }
}
