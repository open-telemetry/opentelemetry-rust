use crate::api::metrics;
use std::marker;
use std::sync::Arc;

pub struct NoopMeter {}

impl metrics::Meter for NoopMeter {
    type LabelSet = NoopLabelSet;
    type I64Counter = NoopCounter<i64>;
    type F64Counter = NoopCounter<f64>;
    type I64Gauge = NoopGauge<i64>;
    type F64Gauge = NoopGauge<f64>;
    type I64Measure = NoopMeasure<i64>;
    type F64Measure = NoopMeasure<f64>;

    fn labels(&self, _key_values: Vec<crate::KeyValue>) -> Self::LabelSet {
        NoopLabelSet {}
    }

    fn new_i64_counter<S: Into<String>>(
        &self,
        _name: S,
        _opts: metrics::Options,
    ) -> Self::I64Counter {
        NoopCounter {
            _marker: marker::PhantomData,
        }
    }

    fn new_f64_counter<S: Into<String>>(
        &self,
        _name: S,
        _opts: metrics::Options,
    ) -> Self::F64Counter {
        NoopCounter {
            _marker: marker::PhantomData,
        }
    }

    fn new_i64_gauge<S: Into<String>>(&self, _name: S, _opts: metrics::Options) -> Self::I64Gauge {
        NoopGauge {
            _marker: marker::PhantomData,
        }
    }

    fn new_f64_gauge<S: Into<String>>(&self, _name: S, _opts: metrics::Options) -> Self::F64Gauge {
        NoopGauge {
            _marker: marker::PhantomData,
        }
    }

    fn new_i64_measure<S: Into<String>>(
        &self,
        _name: S,
        _opts: metrics::Options,
    ) -> Self::I64Measure {
        NoopMeasure {
            _marker: marker::PhantomData,
        }
    }

    fn new_f64_measure<S: Into<String>>(
        &self,
        _name: S,
        _opts: metrics::Options,
    ) -> Self::F64Measure {
        NoopMeasure {
            _marker: marker::PhantomData,
        }
    }

    fn record_batch<M: IntoIterator<Item = metrics::Measurement<NoopLabelSet>>>(
        &self,
        _label_set: &NoopLabelSet,
        _measurements: M,
    ) {
        // Ignored
    }
}

pub struct NoopLabelSet {}

impl metrics::LabelSet for NoopLabelSet {}

pub struct NoopHandle<T> {
    _marker: marker::PhantomData<T>,
}

impl<T> metrics::Instrument<NoopLabelSet> for NoopHandle<T> {
    fn record_one(&self, _value: metrics::MeasurementValue, _label_set: &NoopLabelSet) {
        // Ignored
    }
}

impl<T> metrics::counter::CounterHandle<T> for NoopHandle<T> where
    T: Into<metrics::value::MeasurementValue>
{
}

impl<T> metrics::gauge::GaugeHandle<T> for NoopHandle<T> where
    T: Into<metrics::value::MeasurementValue>
{
}

impl<T> metrics::measure::MeasureHandle<T> for NoopHandle<T> where
    T: Into<metrics::value::MeasurementValue>
{
}

pub struct NoopCounter<T> {
    _marker: marker::PhantomData<T>,
}

impl<T: Into<metrics::value::MeasurementValue> + 'static> metrics::Counter<T, NoopLabelSet>
    for NoopCounter<T>
{
    type Handle = NoopHandle<T>;
    fn measurement(&self, value: T) -> metrics::Measurement<NoopLabelSet> {
        let handle = self.acquire_handle(&NoopLabelSet {});
        metrics::Measurement {
            instrument: Arc::new(handle),
            value: metrics::MeasurementValue::from(value.into()),
        }
    }

    fn acquire_handle(&self, _labels: &NoopLabelSet) -> Self::Handle {
        NoopHandle {
            _marker: marker::PhantomData,
        }
    }
}

impl<T> metrics::Instrument<NoopLabelSet> for NoopCounter<T> {
    fn record_one(&self, _value: metrics::MeasurementValue, _labels: &NoopLabelSet) {
        // Ignored
    }
}

pub struct NoopGauge<T> {
    _marker: marker::PhantomData<T>,
}

impl metrics::Gauge<i64, NoopLabelSet> for NoopGauge<i64> {
    type Handle = NoopHandle<i64>;
    fn measurement(&self, value: i64) -> metrics::Measurement<NoopLabelSet> {
        let handle = self.acquire_handle(&NoopLabelSet {});
        metrics::Measurement {
            instrument: Arc::new(handle),
            value: metrics::MeasurementValue::from(value),
        }
    }

    fn acquire_handle(&self, _labels: &NoopLabelSet) -> Self::Handle {
        NoopHandle {
            _marker: marker::PhantomData,
        }
    }
}

impl metrics::Gauge<f64, NoopLabelSet> for NoopGauge<f64> {
    type Handle = NoopHandle<f64>;
    fn measurement(&self, value: f64) -> metrics::Measurement<NoopLabelSet> {
        let handle = self.acquire_handle(&NoopLabelSet {});
        metrics::Measurement {
            instrument: Arc::new(handle),
            value: metrics::MeasurementValue::from(value),
        }
    }

    fn acquire_handle(&self, _labels: &NoopLabelSet) -> Self::Handle {
        NoopHandle {
            _marker: marker::PhantomData,
        }
    }
}

impl<T> metrics::InstrumentHandle for NoopHandle<T> {
    fn record_one(&self, _value: metrics::MeasurementValue) {
        // Ignored
    }
}

impl<T> metrics::Instrument<NoopLabelSet> for NoopGauge<T> {
    fn record_one(&self, _value: metrics::MeasurementValue, _labels: &NoopLabelSet) {
        // Ignored
    }
}

pub struct NoopMeasure<T> {
    _marker: marker::PhantomData<T>,
}

impl metrics::Measure<i64, NoopLabelSet> for NoopMeasure<i64> {
    type Handle = NoopHandle<i64>;

    fn measurement(&self, value: i64) -> metrics::Measurement<NoopLabelSet> {
        let handle = self.acquire_handle(&NoopLabelSet {});

        metrics::Measurement::new(Arc::new(handle), metrics::MeasurementValue::from(value))
    }

    fn acquire_handle(&self, _labels: &NoopLabelSet) -> Self::Handle {
        NoopHandle {
            _marker: marker::PhantomData,
        }
    }
}

impl metrics::Measure<f64, NoopLabelSet> for NoopMeasure<f64> {
    type Handle = NoopHandle<f64>;

    fn measurement(&self, value: f64) -> metrics::Measurement<NoopLabelSet> {
        let handle = self.acquire_handle(&NoopLabelSet {});

        metrics::Measurement::new(Arc::new(handle), metrics::MeasurementValue::from(value))
    }

    fn acquire_handle(&self, _labels: &NoopLabelSet) -> Self::Handle {
        NoopHandle {
            _marker: marker::PhantomData,
        }
    }
}

impl<T> metrics::Instrument<NoopLabelSet> for NoopMeasure<T> {
    fn record_one(&self, _value: metrics::MeasurementValue, _labels: &NoopLabelSet) {
        // Ignored
    }
}
