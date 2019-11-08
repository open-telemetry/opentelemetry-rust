use crate::api::metrics;

pub trait Gauge<T, LS>: metrics::Instrument<LS>
where
    T: Into<metrics::value::MeasurementValue>,
    LS: metrics::LabelSet,
{
    type Handle: GaugeHandle<T>;
    // Creates a Measurement object to use with batch recording.
    fn measurement(&self, value: T) -> metrics::Measurement<LS>;

    fn acquire_handle(&self, labels: &LS) -> Self::Handle;

    fn set(&self, value: T, label_set: &LS) {
        self.record_one(value.into(), label_set)
    }
}

pub trait GaugeHandle<T>: metrics::InstrumentHandle
where
    T: Into<metrics::value::MeasurementValue>,
{
    fn set(&self, value: T) {
        self.record_one(value.into())
    }
}
