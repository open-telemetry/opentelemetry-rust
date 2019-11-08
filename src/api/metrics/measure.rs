use crate::api::metrics;

pub trait Measure<T, LS>: metrics::Instrument<LS>
where
    T: Into<metrics::value::MeasurementValue>,
    LS: metrics::LabelSet,
{
    type Handle: MeasureHandle<T>;
    // Creates a Measurement object to use with batch recording.
    fn measurement(&self, value: T) -> metrics::Measurement<LS>;

    fn acquire_handle(&self, labels: &LS) -> Self::Handle;

    fn record(&self, value: T, label_set: &LS) {
        self.record_one(value.into(), label_set)
    }
}

pub trait MeasureHandle<T>: metrics::InstrumentHandle
where
    T: Into<metrics::value::MeasurementValue>,
{
    fn record(&self, value: T) {
        self.record_one(value.into())
    }
}
