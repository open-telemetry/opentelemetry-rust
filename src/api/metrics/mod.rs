use std::sync::Arc;

pub mod counter;
pub mod gauge;
pub mod measure;
pub mod noop;
pub mod value;

pub use counter::{Counter, CounterHandle};
pub use gauge::{Gauge, GaugeHandle};
pub use measure::{Measure, MeasureHandle};
pub use value::MeasurementValue;

/// The implementation-level interface to Set/Add/Record individual metrics without precomputed
/// labels.
pub trait Instrument<LS> {
    /// Allows the SDK to observe a single metric event.
    fn record_one(&self, value: MeasurementValue, label_set: &LS);
}

/// The implementation-level interface to Set/Add/Record individual metrics with precomputed labels.
pub trait InstrumentHandle {
    /// Allows the SDK to observe a single metric event.
    fn record_one(&self, value: MeasurementValue);
}

// LabelSet is an implementation-level interface that represents a
// KeyValue for use as pre-defined labels in the metrics API.
pub trait LabelSet {}

// Options contains some options for metrics of any kind.
#[derive(Default)]
pub struct Options {
    // Description is an optional field describing the metric
    // instrument.
    pub description: String,

    // Unit is an optional field describing the metric instrument.
    pub unit: crate::Unit,

    // Keys are recommended keys determined in the handles
    // obtained for the metric.
    pub keys: Vec<crate::Key>,

    // Alternate defines the property of metric value dependent on
    // a metric type.
    //
    // - for Counter, true implies that the metric is an up-down
    //   Counter
    //
    // - for Gauge, true implies that the metric is a
    //   non-descending Gauge
    //
    // - for Measure, true implies that the metric supports
    //   positive and negative values
    pub alternate: bool,
}

impl Options {
    pub fn with_description<S: Into<String>>(self, description: S) -> Self {
        Options {
            description: description.into(),
            ..self
        }
    }

    pub fn with_unit(self, unit: crate::Unit) -> Self {
        Options { unit, ..self }
    }

    pub fn with_keys(self, keys: Vec<crate::Key>) -> Self {
        Options { keys, ..self }
    }

    pub fn with_monotonic(self, _monotonic: bool) -> Self {
        // TODO figure out counter vs gauge issue here.
        unimplemented!()
    }

    pub fn with_absolute(self, absolute: bool) -> Self {
        Options {
            alternate: !absolute,
            ..self
        }
    }
}

pub struct Measurement<LS> {
    instrument: Arc<dyn Instrument<LS>>,
    value: MeasurementValue,
}

impl<LS: LabelSet> Measurement<LS> {
    /// Create a new measurement
    pub fn new(instrument: Arc<dyn Instrument<LS>>, value: MeasurementValue) -> Self {
        Measurement { instrument, value }
    }

    /// Returns an instrument that created this measurement.
    pub fn instrument(&self) -> Arc<dyn Instrument<LS>> {
        self.instrument.clone()
    }

    /// Returns a value recorded in this measurement.
    pub fn into_value(self) -> MeasurementValue {
        self.value
    }
}

/// Meter is an interface to the metrics portion of the OpenTelemetry SDK.
///
/// The Meter interface allows creating of a registered metric instrument using methods specific to
/// each kind of metric. There are six constructors representing the three kinds of instrument
/// taking either floating point or integer inputs, see the detailed design below.
///
/// Binding instruments to a single Meter instance has two benefits:
///
///    1. Instruments can be exported from the zero state, prior to first use, with no explicit
///       Register call
///    2. The component name provided by the named Meter satisfies a namespace requirement
///
/// The recommended practice is to define structures to contain the instruments in use and keep
/// references only to the instruments that are specifically needed.
///
/// We recognize that many existing metric systems support allocating metric instruments statically
/// and providing the Meter interface at the time of use. In this example, typical of statsd
/// clients, existing code may not be structured with a convenient place to store new metric
/// instruments. Where this becomes a burden, it is recommended to use the global meter factory to
/// construct a static named Meter, to construct metric instruments.
///
/// The situation is similar for users of Prometheus clients, where instruments are allocated
/// statically and there is an implicit global. Such code may not have access to the appropriate
/// Meter where instruments are defined. Where this becomes a burden, it is recommended to use the
/// global meter factory to construct a static named Meter, to construct metric instruments.
///
/// Applications are expected to construct long-lived instruments. Instruments are considered
/// permanent for the lifetime of a SDK, there is no method to delete them.
pub trait Meter {
    type LabelSet: LabelSet;
    type I64Counter: Counter<i64, Self::LabelSet>;
    type F64Counter: Counter<f64, Self::LabelSet>;
    type I64Gauge: Gauge<i64, Self::LabelSet>;
    type F64Gauge: Gauge<f64, Self::LabelSet>;
    type I64Measure: Measure<i64, Self::LabelSet>;
    type F64Measure: Measure<f64, Self::LabelSet>;

    // Returns a reference to a set of labels that cannot be read by the application.
    fn labels(&self, key_values: Vec<crate::KeyValue>) -> Self::LabelSet;

    // Creates a new integral counter with a given name and customized with passed options.
    fn new_i64_counter<S: Into<String>>(&self, name: S, opts: Options) -> Self::I64Counter;

    // Creates a new floating point counter with a given name and customized with passed options.
    fn new_f64_counter<S: Into<String>>(&self, name: S, opts: Options) -> Self::F64Counter;

    // Creates a new integral gauge with a given name and customized with passed options.
    fn new_i64_gauge<S: Into<String>>(&self, name: S, opts: Options) -> Self::I64Gauge;

    // Creates a new floating point gauge with a given name and customized with passed options.
    fn new_f64_gauge<S: Into<String>>(&self, name: S, opts: Options) -> Self::F64Gauge;

    // Creates a new integral measure with a given name and customized with passed options.
    fn new_i64_measure<S: Into<String>>(&self, name: S, opts: Options) -> Self::I64Measure;

    // Creates a new floating point measure with a given name and customized with passed options.
    fn new_f64_measure<S: Into<String>>(&self, name: S, opts: Options) -> Self::F64Measure;

    // Atomically records a batch of measurements.
    fn record_batch<M: IntoIterator<Item = Measurement<Self::LabelSet>>>(
        &self,
        label_set: &Self::LabelSet,
        measurements: M,
    );
}
