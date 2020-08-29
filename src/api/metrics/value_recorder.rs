use crate::api::metrics::{
    sync_instrument::{SyncBoundInstrument, SyncInstrument},
    Descriptor, InstrumentKind, Measurement, Meter, Number, NumberKind, Result,
};
use crate::api::KeyValue;
use std::marker;

/// ValueRecorder is a metric that records per-request non-additive values.
#[derive(Debug)]
pub struct ValueRecorder<T>(SyncInstrument<T>);

impl<T> ValueRecorder<T>
where
    T: Into<Number>,
{
    /// Creates a bound instrument for this ValueRecorder. The labels are
    /// associated with values recorded via subsequent calls to record.
    pub fn bind<'a>(&self, labels: &'a [KeyValue]) -> BoundValueRecorder<'a, T> {
        let bound_instrument = self.0.bind(labels);
        BoundValueRecorder {
            labels,
            bound_instrument,
        }
    }

    /// Record a new metric value
    pub fn record(&self, value: T, labels: &[KeyValue]) {
        self.0.direct_record(value.into(), labels)
    }

    /// Creates a `Measurement` object to use with batch recording.
    pub fn measurement(&self, value: T) -> Measurement {
        Measurement::new(value.into(), self.0.instrument().clone())
    }
}

/// BoundValueRecorder is a bound instrument for recording  per-request
/// non-additive values.
///
/// It inherits the Unbind function from syncBoundInstrument.
#[derive(Debug)]
pub struct BoundValueRecorder<'a, T> {
    labels: &'a [KeyValue],
    bound_instrument: SyncBoundInstrument<T>,
}

impl<'a, T> BoundValueRecorder<'a, T>
where
    T: Into<Number>,
{
    /// Adds a new value to the list of ValueRecorder's records. The labels
    /// should contain the keys and values to be associated with this value.
    pub fn record(&self, value: T) {
        self.bound_instrument.direct_record(value.into())
    }
}

/// Initialization configuration for a given `ValueRecorder`.
#[derive(Debug)]
pub struct ValueRecorderBuilder<'a, T> {
    meter: &'a Meter,
    descriptor: Descriptor,
    _marker: marker::PhantomData<T>,
}

impl<'a, T> ValueRecorderBuilder<'a, T> {
    pub(crate) fn new(meter: &'a Meter, name: String, number_kind: NumberKind) -> Self {
        ValueRecorderBuilder {
            meter,
            descriptor: Descriptor::new(
                name,
                meter.instrumentation_name().to_string(),
                InstrumentKind::ValueRecorder,
                number_kind,
            ),
            _marker: marker::PhantomData,
        }
    }

    /// Set the description for this `ValueRecorder`
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.descriptor.set_description(description.into());
        self
    }

    /// Creates a new value recorder.
    pub fn try_init(self) -> Result<ValueRecorder<T>> {
        let instrument = self.meter.new_sync_instrument(self.descriptor)?;
        Ok(ValueRecorder(SyncInstrument::new(instrument)))
    }

    /// Creates a new value recorder.
    ///
    /// # Panics
    ///
    /// This function panics if the instrument cannot be created. Use try_init if you want to
    /// handle errors.
    pub fn init(self) -> ValueRecorder<T> {
        ValueRecorder(SyncInstrument::new(
            self.meter.new_sync_instrument(self.descriptor).unwrap(),
        ))
    }
}
