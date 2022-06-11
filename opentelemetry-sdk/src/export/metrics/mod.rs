//! Metrics Export

use core::fmt;
use std::{sync::Arc, time::SystemTime};

use opentelemetry_api::{attributes, metrics::Result, Context, InstrumentationLibrary};

use crate::{
    metrics::{aggregators::Aggregator, sdk_api::Descriptor},
    Resource,
};

use self::aggregation::TemporalitySelector;

pub mod aggregation;
mod stdout;

pub use stdout::{stdout, ExportLine, ExportNumeric, StdoutExporter, StdoutExporterBuilder};

/// AggregatorSelector supports selecting the kind of `Aggregator` to use at
/// runtime for a specific metric instrument.
pub trait AggregatorSelector {
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

/// A container for the common elements for exported metric data that are shared
/// by the `Accumulator`->`Processor` and `Processor`->`Exporter` steps.
#[derive(Debug)]
pub struct Metadata<'a> {
    descriptor: &'a Descriptor,
    attributes: &'a attributes::AttributeSet,
}

impl<'a> Metadata<'a> {
    /// Create a new `Metadata` instance.
    pub fn new(descriptor: &'a Descriptor, attributes: &'a attributes::AttributeSet) -> Self {
        {
            Metadata {
                descriptor,
                attributes,
            }
        }
    }

    /// A description of the metric instrument being exported.
    pub fn descriptor(&self) -> &Descriptor {
        self.descriptor
    }

    /// The attributes associated with the instrument and the aggregated data.
    pub fn attributes(&self) -> &attributes::AttributeSet {
        self.attributes
    }
}

/// Allows `Accumulator` implementations to construct new `Accumulation`s to
/// send to `Processor`s. The `Descriptor`, `Attributes`, `Resource`, and
/// `Aggregator` represent aggregate metric events received over a single
/// collection period.
pub fn accumulation<'a>(
    descriptor: &'a Descriptor,
    attributes: &'a attributes::AttributeSet,
    aggregator: &'a Arc<dyn Aggregator + Send + Sync>,
) -> Accumulation<'a> {
    Accumulation::new(descriptor, attributes, aggregator)
}

/// A container for the exported data for a single metric instrument and attribute
/// set, as prepared by an `Accumulator` for the `Processor`.
pub struct Accumulation<'a> {
    metadata: Metadata<'a>,
    aggregator: &'a Arc<dyn Aggregator + Send + Sync>,
}

impl<'a> Accumulation<'a> {
    /// Create a new `Record` instance.
    pub fn new(
        descriptor: &'a Descriptor,
        attributes: &'a attributes::AttributeSet,
        aggregator: &'a Arc<dyn Aggregator + Send + Sync>,
    ) -> Self {
        Accumulation {
            metadata: Metadata::new(descriptor, attributes),
            aggregator,
        }
    }

    /// A description of the metric instrument being exported.
    pub fn descriptor(&self) -> &Descriptor {
        self.metadata.descriptor
    }

    /// The attributes associated with the instrument and the aggregated data.
    pub fn attributes(&self) -> &attributes::AttributeSet {
        self.metadata.attributes
    }

    /// The checkpointed aggregator for this metric.
    pub fn aggregator(&self) -> &Arc<dyn Aggregator + Send + Sync> {
        self.aggregator
    }
}

impl<'a> fmt::Debug for Accumulation<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Accumulation")
            .field("metadata", &self.metadata)
            .finish()
    }
}

/// Metric data processor.
///
/// Locked processors are responsible gathering exported results from the SDK during
/// collection, and deciding over which dimensions to group the exported data.
///
/// The `process` method is called during collection in a single-threaded
/// context from the SDK, after the aggregator is checkpointed, allowing the
/// processor to build the set of metrics currently being exported.
pub trait LockedProcessor {
    /// Process is called by the SDK once per internal record, passing the export
    /// [`Accumulation`] (a Descriptor, the corresponding attributes, and the
    /// checkpointed aggregator).
    ///
    /// This call has no [`Context`] argument because it is expected to perform only
    /// computation. An SDK is not expected to call exporters from with Process, use
    /// a controller for that.
    fn process(&mut self, accumulation: Accumulation<'_>) -> Result<()>;
}

/// A container for the exported data for a single metric instrument and attribute
/// set, as prepared by the `Processor` for the `Exporter`. This includes the
/// effective start and end time for the aggregation.
pub struct Record<'a> {
    metadata: Metadata<'a>,
    aggregator: Option<&'a Arc<dyn Aggregator + Send + Sync>>,
    start: SystemTime,
    end: SystemTime,
}

impl Record<'_> {
    /// A description of the metric instrument being exported.
    pub fn descriptor(&self) -> &Descriptor {
        self.metadata.descriptor
    }

    /// The attributes associated with the instrument and the aggregated data.
    pub fn attributes(&self) -> &attributes::AttributeSet {
        self.metadata.attributes
    }

    /// The aggregator for this metric
    pub fn aggregator(&self) -> Option<&Arc<dyn Aggregator + Send + Sync>> {
        self.aggregator
    }

    /// The start time of the interval covered by this aggregation.
    pub fn start_time(&self) -> &SystemTime {
        &self.start
    }

    /// The end time of the interval covered by this aggregation.
    pub fn end_time(&self) -> &SystemTime {
        &self.end
    }
}

/// Exporter handles presentation of the checkpoint of aggregate
/// metrics.  This is the final stage of a metrics export pipeline,
/// where metric data are formatted for a specific system.
pub trait MetricsExporter: TemporalitySelector {
    /// Export is called immediately after completing a collection
    /// pass in the SDK.
    ///
    /// The Context comes from the controller that initiated
    /// collection.
    ///
    /// The InstrumentationLibraryReader interface refers to the
    /// Processor that just completed collection.
    fn export(
        &self,
        cx: &Context,
        res: &Resource,
        reader: &dyn InstrumentationLibraryReader,
    ) -> Result<()>;
}

/// InstrumentationLibraryReader is an interface for exporters to iterate
/// over one instrumentation library of metric data at a time.
pub trait InstrumentationLibraryReader {
    /// ForEach calls the passed function once per instrumentation library,
    /// allowing the caller to emit metrics grouped by the library that
    /// produced them.
    fn try_for_each(
        &self,
        f: &mut dyn FnMut(&InstrumentationLibrary, &mut dyn Reader) -> Result<()>,
    ) -> Result<()>;
}

/// Reader allows a controller to access a complete checkpoint of
/// aggregated metrics from the Processor for a single library of
/// metric data.  This is passed to the Exporter which may then use
/// ForEach to iterate over the collection of aggregated metrics.
pub trait Reader {
    /// ForEach iterates over aggregated checkpoints for all
    /// metrics that were updated during the last collection
    /// period. Each aggregated checkpoint returned by the
    /// function parameter may return an error.
    ///
    /// The TemporalitySelector argument is used to determine
    /// whether the Record is computed using Delta or Cumulative
    /// aggregation.
    ///
    /// ForEach tolerates ErrNoData silently, as this is
    /// expected from the Meter implementation. Any other kind
    /// of error will immediately halt ForEach and return
    /// the error to the caller.
    fn try_for_each(
        &mut self,
        temp_selector: &dyn TemporalitySelector,
        f: &mut dyn FnMut(&Record<'_>) -> Result<()>,
    ) -> Result<()>;
}

impl fmt::Debug for Record<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Record")
            .field("metadata", &self.metadata)
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

/// The interface used to create checkpoints.
pub trait Checkpointer: Processor {
    /// Synchronizes the checkpoint process and allows a single locked
    /// checkpoint to be accessed at a time.
    fn checkpoint(
        &self,
        f: &mut dyn FnMut(&mut dyn LockedCheckpointer) -> Result<()>,
    ) -> Result<()>;
}

/// The interface used by a controller to coordinate the processor with
/// accumulator(s) and exporter(s).
///
/// The StartCollection() and FinishCollection() methods start and finish a
/// collection interval. Controllers call the Accumulator(s) during collection
/// to process Accumulations.
pub trait LockedCheckpointer {
    /// Processes metric data for export.
    ///
    /// The `process` method is bracketed by `start_collection` and
    /// `finish_collection` calls.
    fn processor(&mut self) -> &mut dyn LockedProcessor;

    /// Reader returns the current data set.
    ///
    /// This may be called before and after collection. The implementation is
    /// required to return the same value throughout its lifetime.
    fn reader(&mut self) -> &mut dyn Reader;

    /// begins a collection interval.
    fn start_collection(&mut self);

    /// ends a collection interval.
    fn finish_collection(&mut self) -> Result<()>;
}

/// An interface for producing configured [`Checkpointer`] instances.
pub trait CheckpointerFactory {
    /// Creates a new configured checkpointer.
    fn checkpointer(&self) -> Arc<dyn Checkpointer + Send + Sync>;
}

/// Allows `Processor` implementations to construct export records. The
/// `Descriptor`, `Attributes`, and `Aggregator` represent aggregate metric events
/// received over a single collection period.
pub fn record<'a>(
    descriptor: &'a Descriptor,
    attributes: &'a attributes::AttributeSet,
    aggregator: Option<&'a Arc<dyn Aggregator + Send + Sync>>,
    start: SystemTime,
    end: SystemTime,
) -> Record<'a> {
    Record {
        metadata: Metadata::new(descriptor, attributes),
        aggregator,
        start,
        end,
    }
}

/// A utility extension to allow upcasting.
///
/// Can be removed once [trait_upcasting] is stablized.
///
/// [trait_upcasting]: https://doc.rust-lang.org/unstable-book/language-features/trait-upcasting.html
pub trait AsDynProcessor {
    /// Create an `Arc<dyn Processor>` from an impl of [`Processor`].
    fn as_dyn_processor<'a>(self: Arc<Self>) -> Arc<dyn Processor + Send + Sync + 'a>
    where
        Self: 'a;
}

impl<T: Processor + Sized + Send + Sync> AsDynProcessor for T {
    fn as_dyn_processor<'a>(self: Arc<Self>) -> Arc<dyn Processor + Send + Sync + 'a>
    where
        Self: 'a,
    {
        self
    }
}

/// Processor is responsible for deciding which kind of aggregation to use (via
/// `aggregation_selector`), gathering exported results from the SDK during
/// collection, and deciding over which dimensions to group the exported data.
///
/// The SDK supports binding only one of these interfaces, as it has the sole
/// responsibility of determining which Aggregator to use for each record.
///
/// The embedded AggregatorSelector interface is called (concurrently) in
/// instrumentation context to select the appropriate Aggregator for an
/// instrument.
pub trait Processor: AsDynProcessor {
    /// AggregatorSelector is responsible for selecting the
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
    fn aggregator_selector(&self) -> &dyn AggregatorSelector;
}
