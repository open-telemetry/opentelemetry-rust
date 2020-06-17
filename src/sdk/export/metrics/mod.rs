//! Metrics Export
use crate::api::{
    labels,
    metrics::{Descriptor, Number, Result},
    Context,
};
use crate::sdk::resource::Resource;
use std::any::Any;
use std::fmt;
use std::sync::Arc;

mod aggregation;

pub use aggregation::{
    Buckets, Count, Distribution, Histogram, LastValue, Max, Min, MinMaxSumCount, Points, Quantile,
    Sum,
};

/// Integrator is responsible for deciding which kind of aggregation to use (via
/// `aggregation_selector`), gathering exported results from the SDK during
/// collection, and deciding over which dimensions to group the exported data.
///
/// The SDK supports binding only one of these interfaces, as it has the sole
/// responsibility of determining which Aggregator to use for each record.
///
/// The embedded AggregationSelector interface is called (concurrently) in
/// instrumentation context to select the appropriate Aggregator for an
/// instrument.
pub trait Integrator: fmt::Debug {
    /// AggregationSelector is responsible for selecting the
    /// concrete type of Aggregator used for a metric in the SDK.
    ///
    /// This may be a static decision based on fields of the
    /// Descriptor, or it could use an external configuration
    /// source to customize the treatment of each metric
    /// instrument.
    ///
    /// The result from AggregatorSelector.AggregatorFor should be
    /// the same type for a given Descriptor or else nil.  The same
    /// type should be returned for a given descriptor, because
    /// Aggregators only know how to Merge with their own type.  If
    /// the result is nil, the metric instrument will be disabled.
    ///
    /// Note that the SDK only calls AggregatorFor when new records
    /// require an Aggregator. This does not provide a way to
    /// disable metrics with active records.
    fn aggregation_selector(&self) -> &dyn AggregationSelector;
}

/// A locked integrator.
///
/// The `Process` method is called during collection in a single-threaded
/// context from the SDK, after the aggregator is checkpointed, allowing the
/// integrator to build the set of metrics currently being exported.
pub trait LockedIntegrator {
    /// Process is called by the SDK once per internal record, passing the export
    /// Record (a Descriptor, the corresponding Labels, and the checkpointed
    /// Aggregator).
    ///
    /// The Context argument originates from the controller that orchestrates
    /// collection.
    fn process(&mut self, record: Record) -> Result<()>;

    /// Allows a controller to access a complete checkpoint of aggregated metrics
    /// from the Integrator. This is passed to the Exporter which may then iterate
    /// over the collection of aggregated metrics.
    fn checkpoint_set(&mut self) -> &mut dyn CheckpointSet;

    /// Cleanup logic or other behavior that needs to be run by the integrator after
    /// collection is complete.
    fn finished_collection(&mut self);
}

/// AggregationSelector supports selecting the kind of `Aggregator` to use at
/// runtime for a specific metric instrument.
pub trait AggregationSelector: fmt::Debug {
    /// This allocates a variable number of aggregators of a kind suitable for
    /// the requested export.   
    ///
    /// When the call returns `None`, the metric instrument is explicitly disabled.
    ///
    /// This must return a consistent type to avoid confusion in later stages of
    /// the metrics export process, e.g., when merging or checkpointing
    /// aggregators for a specific instrument.
    ///
    /// This call should not block.
    fn aggregator_for(&self, descriptor: &Descriptor) -> Option<Arc<dyn Aggregator + Send + Sync>>;
}

/// Aggregator implements a specific aggregation behavior, i.e., a behavior to
/// track a sequence of updates to an instrument. Sum-only instruments commonly
/// use a simple Sum aggregator, but for the distribution instruments
/// (ValueRecorder, ValueObserver) there are a number of possible aggregators
/// with different cost and accuracy tradeoffs.
///
/// Note that any Aggregator may be attached to any instrument--this is the
/// result of the OpenTelemetry API/SDK separation. It is possible to attach a
/// Sum aggregator to a ValueRecorder instrument or a MinMaxSumCount aggregator
/// to a Counter instrument.
pub trait Aggregator: fmt::Debug {
    /// Update receives a new measured value and incorporates it into the
    /// aggregation. Update calls may be called concurrently.
    ///
    /// `Descriptor::number_kind` should be consulted to determine whether the
    /// provided number is an `i64`, `u64` or `f64`.
    ///
    /// The current Context could be inspected for a `CorrelationContext` or
    /// `SpanContext`.
    fn update(&self, number: &Number, descriptor: &Descriptor) -> Result<()> {
        self.update_with_context(&Context::current(), number, descriptor)
    }

    /// Update receives a new measured value and incorporates it into the
    /// aggregation. Update calls may be called concurrently.
    ///
    /// `Descriptor::number_kind` should be consulted to determine whether the
    /// provided number is an `i64`, `u64` or `f64`.
    ///
    /// The Context argument comes from user-level code and could be inspected for a
    /// `CorrelationContext` or `SpanContext`.
    fn update_with_context(
        &self,
        cx: &Context,
        number: &Number,
        descriptor: &Descriptor,
    ) -> Result<()>;

    /// This method is called during collection to finish one period of aggregation
    /// by atomically saving the currently-updating state into the argument
    /// Aggregator.
    ///
    /// `synchronized_copy` is called concurrently with `update`. These two methods
    /// must be synchronized with respect to each other, for correctness.
    ///
    /// This method will return an `InconsistentAggregator` error if this
    /// `Aggregator` cannot be copied into the destination due to an incompatible
    /// type.
    ///
    /// This call has no `Context` argument because it is expected to perform only
    /// computation.
    fn synchronized_copy(
        &self,
        destination: &Arc<dyn Aggregator + Send + Sync>,
        descriptor: &Descriptor,
    ) -> Result<()>;

    /// This combines the checkpointed state from the argument `Aggregator` into this
    /// `Aggregator`. `merge` is not synchronized with respect to `update` or
    /// `synchronized_copy`.
    ///
    /// The owner of an `Aggregator` being merged is responsible for synchronization
    /// of both `Aggregator` states.
    fn merge(&self, other: &(dyn Aggregator + Send + Sync), descriptor: &Descriptor) -> Result<()>;

    /// Returns the implementing aggregator as `Any` for downcasting.
    fn as_any(&self) -> &dyn Any;
}

/// Exporter handles presentation of the checkpoint of aggregate metrics. This
/// is the final stage of a metrics export pipeline, where metric data are
/// formatted for a specific system.
pub trait Exporter: fmt::Debug {
    /// Export is called immediately after completing a collection pass in the SDK.
    ///
    /// The CheckpointSet interface refers to the Integrator that just completed
    /// collection.
    fn export(&self, checkpoint_set: &mut dyn CheckpointSet) -> Result<()>;
}

/// CheckpointSet allows a controller to access a complete checkpoint of
/// aggregated metrics from the Integrator. This is passed to the `Exporter`
/// which may then use `try_for_each` to iterate over the collection of
/// aggregated metrics.
pub trait CheckpointSet: fmt::Debug {
    /// This iterates over aggregated checkpoints for all metrics that were updated
    /// during the last collection period. Each aggregated checkpoint returned by
    /// the function parameter may return an error. ForEach tolerates ErrNoData
    /// silently, as this is expected from the Meter implementation. Any other kind
    /// of error will immediately halt ForEach and return the error to the caller.
    fn try_for_each(&mut self, f: &mut dyn FnMut(&Record) -> Result<()>) -> Result<()>;
}

/// Create a new `Record` instance.
pub fn record<'a>(
    descriptor: &'a Descriptor,
    labels: &'a labels::Set,
    resource: &'a Resource,
    aggregator: &'a Arc<dyn Aggregator + Send + Sync>,
) -> Record<'a> {
    Record::new(descriptor, labels, resource, aggregator)
}

/// Record contains the exported data for a single metric instrument and label set.
#[derive(Debug)]
pub struct Record<'a> {
    descriptor: &'a Descriptor,
    labels: &'a labels::Set,
    resource: &'a Resource,
    aggregator: &'a Arc<dyn Aggregator + Send + Sync>,
}

impl<'a> Record<'a> {
    /// Create a new `Record` instance.
    pub fn new(
        descriptor: &'a Descriptor,
        labels: &'a labels::Set,
        resource: &'a Resource,
        aggregator: &'a Arc<dyn Aggregator + Send + Sync>,
    ) -> Self {
        Record {
            descriptor,
            labels,
            resource,
            aggregator,
        }
    }

    /// The descriptor for this metric.
    pub fn descriptor(&self) -> &Descriptor {
        self.descriptor
    }

    /// The labels for this metric.
    pub fn labels(&self) -> &labels::Set {
        self.labels
    }

    /// The resource for this metric.
    pub fn resource(&self) -> &Resource {
        self.resource
    }

    /// The aggregator for this metric.
    pub fn aggregator(&self) -> &Arc<dyn Aggregator + Send + Sync> {
        self.aggregator
    }
}
