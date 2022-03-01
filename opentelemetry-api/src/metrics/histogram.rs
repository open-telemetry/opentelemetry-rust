use crate::metrics::{
    sync_instrument::{SyncBoundInstrument, SyncInstrument},
    Descriptor, InstrumentKind, Measurement, Meter, Number, NumberKind, Result, Unit,
};
use crate::KeyValue;
use std::marker;

/// Histogram is a metric that records per-request non-additive values.
#[derive(Clone, Debug)]
pub struct Histogram<T>(SyncInstrument<T>);

impl<T> Histogram<T>
where
    T: Into<Number>,
{
    /// Creates a bound instrument for this Histogram. The attributes are
    /// associated with values recorded via subsequent calls to record.
    pub fn bind(&self, attributes: &[KeyValue]) -> BoundHistogram<T> {
        let bound_instrument = self.0.bind(attributes);
        BoundHistogram { bound_instrument }
    }

    /// Record a new metric value
    pub fn record(&self, value: T, attributes: &[KeyValue]) {
        self.0.direct_record(value.into(), attributes)
    }

    /// Creates a `Measurement` object to use with batch recording.
    pub fn measurement(&self, value: T) -> Measurement {
        Measurement::new(value.into(), self.0.instrument().clone())
    }
}

/// BoundHistogram is a bound instrument for recording  per-request
/// non-additive values.
///
/// It inherits the Unbind function from syncBoundInstrument.
#[derive(Clone, Debug)]
pub struct BoundHistogram<T> {
    bound_instrument: SyncBoundInstrument<T>,
}

impl<T> BoundHistogram<T>
where
    T: Into<Number>,
{
    /// Adds a new value to the list of Histogram's records. The attributes
    /// should contain the keys and values to be associated with this value.
    pub fn record(&self, value: T) {
        self.bound_instrument.direct_record(value.into())
    }
}

/// Initialization configuration for a given `Histogram`.
#[derive(Debug)]
pub struct HistogramBuilder<'a, T> {
    meter: &'a Meter,
    descriptor: Descriptor,
    _marker: marker::PhantomData<T>,
}

impl<'a, T> HistogramBuilder<'a, T> {
    pub(crate) fn new(meter: &'a Meter, name: String, number_kind: NumberKind) -> Self {
        HistogramBuilder {
            meter,
            descriptor: Descriptor::new(
                name,
                meter.instrumentation_library().name,
                meter.instrumentation_library().version,
                meter.instrumentation_library().schema_url,
                InstrumentKind::Histogram,
                number_kind,
            ),
            _marker: marker::PhantomData,
        }
    }

    /// Set the description for this `Histogram`
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.descriptor.set_description(description.into());
        self
    }

    /// Set the unit for this `Histogram`.
    pub fn with_unit(mut self, unit: Unit) -> Self {
        self.descriptor.config.unit = Some(unit);
        self
    }

    /// Tries to create a new `Histogram`.
    pub fn try_init(self) -> Result<Histogram<T>> {
        let instrument = self.meter.new_sync_instrument(self.descriptor)?;
        Ok(Histogram(SyncInstrument::new(instrument)))
    }

    /// Creates a new `Histogram`.
    ///
    /// # Panics
    ///
    /// This function panics if the instrument cannot be created. Use try_init if you want to
    /// handle errors.
    pub fn init(self) -> Histogram<T> {
        Histogram(SyncInstrument::new(
            self.meter.new_sync_instrument(self.descriptor).unwrap(),
        ))
    }
}
